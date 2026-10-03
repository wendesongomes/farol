//! Which TCP ports are being listened on, and by which process.
//!
//! `netstat2` reads this straight from the OS on all three systems. If it
//! fails, Farol falls back to the system's own tool (`lsof`, `ss` or
//! `netstat`) and parses its output.

use std::collections::BTreeSet;

use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};

/// One listening port. `pid` is `None` when the system won't say who owns the
/// socket (on Linux, sockets of other users when not running as root).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Listener {
    pub port: u16,
    pub pid: Option<u32>,
}

pub fn listening() -> Vec<Listener> {
    match from_netstat2() {
        Ok(listeners) => listeners,
        Err(error) => {
            eprintln!("farol: netstat2 failed ({error}), using the native command");
            fallback::listening()
        }
    }
}

fn from_netstat2() -> Result<Vec<Listener>, netstat2::error::Error> {
    let sockets = netstat2::get_sockets_info(
        AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
        ProtocolFlags::TCP,
    )?;
    let raw = sockets
        .into_iter()
        .filter_map(|socket| match socket.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp) if tcp.state == TcpState::Listen => {
                Some((tcp.local_port, socket.associated_pids))
            }
            _ => None,
        });
    Ok(dedup(raw))
}

/// Collapses the raw socket list into one entry per (pid, port).
///
/// - The same server usually listens on both IPv4 and IPv6: one entry.
/// - Pre-forking servers (gunicorn, nginx, php-fpm) share one socket among
///   several processes: the lowest PID is kept, which is the parent that
///   started the others and the one to stop.
pub fn dedup(sockets: impl IntoIterator<Item = (u16, Vec<u32>)>) -> Vec<Listener> {
    let mut known: BTreeSet<Listener> = BTreeSet::new();
    for (port, pids) in sockets {
        known.insert(Listener {
            port,
            pid: pids.into_iter().min(),
        });
    }
    // A port seen without a PID is dropped if another socket on the same
    // port does have one: it is the same server seen through a socket whose
    // owner the system didn't report.
    let with_pid: BTreeSet<u16> = known
        .iter()
        .filter(|l| l.pid.is_some())
        .map(|l| l.port)
        .collect();
    known
        .into_iter()
        .filter(|l| l.pid.is_some() || !with_pid.contains(&l.port))
        .collect()
}

/// Parsers for the native commands. They only run when `netstat2` fails, but
/// are compiled and tested everywhere.
pub mod fallback {
    use super::{dedup, Listener};

    pub fn listening() -> Vec<Listener> {
        #[cfg(target_os = "macos")]
        let parsed = crate::command::output("lsof", &["-nP", "-iTCP", "-sTCP:LISTEN", "-F", "pcn"])
            .map(|out| parse_lsof(&out));
        #[cfg(target_os = "linux")]
        let parsed = crate::command::output("ss", &["-ltnpH"]).map(|out| parse_ss(&out));
        #[cfg(windows)]
        let parsed = crate::command::output("netstat", &["-ano", "-p", "TCP"])
            .map(|out| parse_netstat(&out));
        parsed.unwrap_or_default()
    }

    /// The port is whatever follows the last `:` in an address such as
    /// `*:3000`, `127.0.0.1:5432` or `[::1]:8080`.
    fn port_of(address: &str) -> Option<u16> {
        address.rsplit_once(':')?.1.parse().ok()
    }

    /// `lsof -nP -iTCP -sTCP:LISTEN -F pcn`: one field per line, prefixed by
    /// its letter. `p` starts a process, each `n` that follows is one of its
    /// sockets.
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub fn parse_lsof(output: &str) -> Vec<Listener> {
        let mut pid = None;
        let mut sockets = Vec::new();
        for line in output.lines() {
            let (field, value) = line.split_at(line.len().min(1));
            match field {
                "p" => pid = value.parse::<u32>().ok(),
                "n" => {
                    if let Some(port) = port_of(value) {
                        sockets.push((port, pid.into_iter().collect()));
                    }
                }
                _ => {}
            }
        }
        dedup(sockets)
    }

    /// `ss -ltnpH`: `State Recv-Q Send-Q Local Peer Process`. The process
    /// column is missing for sockets of other users.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    pub fn parse_ss(output: &str) -> Vec<Listener> {
        let sockets = output.lines().filter_map(|line| {
            let mut columns = line.split_whitespace();
            let local = columns.nth(3)?;
            let port = port_of(local)?;
            let pids = line
                .split("pid=")
                .skip(1)
                .filter_map(|rest| {
                    rest.split(|c: char| !c.is_ascii_digit())
                        .next()?
                        .parse()
                        .ok()
                })
                .collect();
            Some((port, pids))
        });
        dedup(sockets)
    }

    /// `netstat -ano -p TCP`: `Proto Local Foreign State PID`. The state is
    /// translated on non-English Windows ("ESCUTANDO"), so listening sockets
    /// are recognized by their foreign address, which is always `*:0`.
    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn parse_netstat(output: &str) -> Vec<Listener> {
        let sockets = output.lines().filter_map(|line| {
            let columns: Vec<&str> = line.split_whitespace().collect();
            let [proto, local, foreign, _state, pid] = columns[..] else {
                return None;
            };
            if !proto.eq_ignore_ascii_case("TCP") || !foreign.ends_with(":0") {
                return None;
            }
            Some((port_of(local)?, vec![pid.parse().ok()?]))
        });
        dedup(sockets)
    }
}

#[cfg(test)]
mod tests {
    use super::fallback::*;
    use super::*;

    fn listener(port: u16, pid: Option<u32>) -> Listener {
        Listener { port, pid }
    }

    #[test]
    fn dedup_merges_ipv4_and_ipv6() {
        let raw = vec![(3000, vec![42]), (3000, vec![42]), (5432, vec![7])];
        assert_eq!(
            dedup(raw),
            vec![listener(3000, Some(42)), listener(5432, Some(7))]
        );
    }

    #[test]
    fn dedup_keeps_the_parent_of_preforked_workers() {
        let raw = vec![(8000, vec![120, 118, 119])];
        assert_eq!(dedup(raw), vec![listener(8000, Some(118))]);
    }

    #[test]
    fn dedup_keeps_unknown_owner_only_when_nothing_else_is_known() {
        let raw = vec![(3000, vec![]), (3000, vec![42]), (631, vec![])];
        assert_eq!(
            dedup(raw),
            vec![listener(631, None), listener(3000, Some(42))]
        );
    }

    #[test]
    fn different_processes_on_the_same_port_are_kept() {
        // e.g. one server on 127.0.0.1:8080 and another on [::1]:8080.
        let raw = vec![(8080, vec![10]), (8080, vec![20])];
        assert_eq!(
            dedup(raw),
            vec![listener(8080, Some(10)), listener(8080, Some(20))]
        );
    }

    #[test]
    fn parses_lsof() {
        let output =
            "p512\ncnode\nn*:3000\nn[::1]:3000\np88\ncpostgres\nn127.0.0.1:5432\nn[::1]:5432\n";
        assert_eq!(
            parse_lsof(output),
            vec![listener(3000, Some(512)), listener(5432, Some(88))]
        );
    }

    #[test]
    fn parses_ss() {
        let output = "\
LISTEN 0      511          0.0.0.0:3000      0.0.0.0:*    users:((\"node\",pid=1234,fd=20))
LISTEN 0      4096       127.0.0.1:5432      0.0.0.0:*
LISTEN 0      511             [::]:3000         [::]:*    users:((\"node\",pid=1234,fd=21))
LISTEN 0      128          0.0.0.0:8000      0.0.0.0:*    users:((\"gunicorn\",pid=301,fd=5),(\"gunicorn\",pid=300,fd=5))
";
        assert_eq!(
            parse_ss(output),
            vec![
                listener(3000, Some(1234)),
                listener(5432, None),
                listener(8000, Some(300))
            ]
        );
    }

    #[test]
    fn parses_netstat_in_any_language() {
        let output = "
Conexões ativas

  Proto  Endereço local         Endereço externo       Estado         PID
  TCP    0.0.0.0:135            0.0.0.0:0              ESCUTANDO       1004
  TCP    0.0.0.0:3000           0.0.0.0:0              LISTENING       8812
  TCP    127.0.0.1:3000         127.0.0.1:52144        ESTABLISHED     8812
  TCP    [::]:3000              [::]:0                 LISTENING       8812
  TCP    [::1]:5173             [::]:0                 LISTENING       9100
";
        assert_eq!(
            parse_netstat(output),
            vec![
                listener(135, Some(1004)),
                listener(3000, Some(8812)),
                listener(5173, Some(9100))
            ]
        );
    }

    #[test]
    fn finds_this_test_listening() {
        // Real end-to-end check of the netstat2 path on the current system.
        let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();
        let me = std::process::id();
        assert!(super::listening().contains(&listener(port, Some(me))));
    }
}
