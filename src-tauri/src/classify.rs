//! Front-end or back-end?
//!
//! In order, the first that decides wins:
//! 1. the user's manual choice (applied in `scan.rs`);
//! 2. keywords in the command line (`rules/detection.toml`);
//! 3. an HTTP request to the port: HTML means front-end;
//! 4. back-end.

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use crate::model::ProcessType;
use crate::rules::{contains_keyword, TypeRules};

const PROBE_TIMEOUT: Duration = Duration::from_millis(300);

/// Keyword step. When keywords from both lists match, the longest (most
/// specific) wins; ties go to back-end. `None` when nothing matches.
pub fn from_keywords(command: &str, rules: &TypeRules) -> Option<ProcessType> {
    let longest = |keywords: &[String]| {
        keywords
            .iter()
            .filter(|k| contains_keyword(command, k))
            .map(|k| k.len())
            .max()
    };
    match (
        longest(&rules.front.keywords),
        longest(&rules.back.keywords),
    ) {
        (Some(front), Some(back)) if front > back => Some(ProcessType::Front),
        (Some(_), None) => Some(ProcessType::Front),
        (_, Some(_)) => Some(ProcessType::Back),
        (None, None) => None,
    }
}

/// HTTP step. Tries IPv4 first, then IPv6 (on Windows `localhost` sometimes
/// only answers on `::1`). Anything that isn't an HTML page is back-end.
pub fn from_probe(port: u16) -> ProcessType {
    let addresses = [
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        IpAddr::V6(Ipv6Addr::LOCALHOST),
    ];
    for ip in addresses {
        if let Some(content_type) = probe_content_type(SocketAddr::new(ip, port)) {
            return if is_html(&content_type) {
                ProcessType::Front
            } else {
                ProcessType::Back
            };
        }
    }
    ProcessType::Back
}

pub fn is_html(content_type: &str) -> bool {
    content_type.to_ascii_lowercase().contains("text/html")
}

/// Sends `GET /` and returns the Content-Type header ("" when the response
/// has none). `None` when nothing answered HTTP within the timeout.
fn probe_content_type(address: SocketAddr) -> Option<String> {
    let deadline = Instant::now() + PROBE_TIMEOUT;
    let mut stream = TcpStream::connect_timeout(&address, PROBE_TIMEOUT).ok()?;
    let host = match address.ip() {
        IpAddr::V6(_) => format!("[::1]:{}", address.port()),
        IpAddr::V4(_) => format!("127.0.0.1:{}", address.port()),
    };
    let request = format!(
        "GET / HTTP/1.1\r\nHost: {host}\r\nAccept: text/html,*/*\r\nUser-Agent: Farol\r\nConnection: close\r\n\r\n"
    );
    stream.set_write_timeout(Some(PROBE_TIMEOUT)).ok()?;
    stream.write_all(request.as_bytes()).ok()?;

    // Only the headers matter; stop at the blank line or after 16 KB.
    let mut response = Vec::new();
    let mut buffer = [0u8; 2048];
    while response.len() < 16 * 1024 && !has_header_end(&response) {
        let left = deadline.checked_duration_since(Instant::now())?;
        stream.set_read_timeout(Some(left)).ok()?;
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => response.extend_from_slice(&buffer[..n]),
            Err(_) => break,
        }
    }
    parse_content_type(&String::from_utf8_lossy(&response))
}

fn has_header_end(bytes: &[u8]) -> bool {
    bytes.windows(4).any(|w| w == b"\r\n\r\n") || bytes.windows(2).any(|w| w == b"\n\n")
}

/// Returns the Content-Type of an HTTP response head, `Some("")` for an HTTP
/// response without one, and `None` if it isn't HTTP at all.
pub fn parse_content_type(head: &str) -> Option<String> {
    let mut lines = head.lines();
    if !lines.next()?.starts_with("HTTP/") {
        return None;
    }
    let content_type = lines
        .take_while(|line| !line.trim().is_empty())
        .filter_map(|line| line.split_once(':'))
        .find(|(name, _)| name.trim().eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.trim().to_string())
        .unwrap_or_default();
    Some(content_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::rules;
    use serde::Deserialize;
    use std::net::TcpListener;

    #[derive(Deserialize)]
    struct Cases {
        #[serde(rename = "type")]
        kind: Vec<TypeCase>,
    }

    #[derive(Deserialize)]
    struct TypeCase {
        command: String,
        expect: String,
    }

    /// Runs every `[[type]]` case of rules/detection_tests.toml.
    #[test]
    fn detection_rules_cases() {
        let cases: Cases =
            toml::from_str(include_str!("../../rules/detection_tests.toml")).unwrap();
        assert!(!cases.kind.is_empty());
        for case in cases.kind {
            let got = match from_keywords(&case.command, &rules().kind) {
                Some(ProcessType::Front) => "front",
                Some(ProcessType::Back) => "back",
                None => "none",
            };
            assert_eq!(got, case.expect, "command: {}", case.command);
        }
    }

    #[test]
    fn parses_content_type() {
        let head = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n";
        assert_eq!(
            parse_content_type(head).as_deref(),
            Some("text/html; charset=utf-8")
        );
        let json = "HTTP/1.1 404 Not Found\r\ncontent-type: application/json\r\n\r\n{}";
        assert_eq!(
            parse_content_type(json).as_deref(),
            Some("application/json")
        );
        assert_eq!(
            parse_content_type("HTTP/1.0 204 No Content\r\n\r\n").as_deref(),
            Some("")
        );
        assert_eq!(parse_content_type("-ERR unknown command 'GET'\r\n"), None);
        assert_eq!(parse_content_type(""), None);
    }

    /// Serves one canned response on a random port.
    fn serve_once(response: &'static str) -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buffer = [0u8; 1024];
                let _ = stream.read(&mut buffer);
                let _ = stream.write_all(response.as_bytes());
            }
        });
        port
    }

    #[test]
    fn probe_detects_html_as_front() {
        let port = serve_once("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<!doctype html>");
        assert_eq!(from_probe(port), ProcessType::Front);
    }

    #[test]
    fn probe_detects_json_as_back() {
        let port = serve_once("HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\n\r\n{}");
        assert_eq!(from_probe(port), ProcessType::Back);
    }

    #[test]
    fn probe_without_http_is_back() {
        // Accepts and closes without answering, like a database would.
        let port = serve_once("");
        assert_eq!(from_probe(port), ProcessType::Back);
    }
}
