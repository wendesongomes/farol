# Contributing to Farol

Thanks for helping! Farol is a small project and contributions of every size are welcome. Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Running locally

Follow [Run from source](README.md#run-from-source) in the README: it lists the prerequisites for each system and the `npm run tauri dev` command.

## The easiest way to contribute: a detection rule

Farol decides whether a server is a front-end or a back-end, and who started it, using the lists in [`rules/detection.toml`](rules/detection.toml). That file is plain data, embedded in the app at build time: supporting a new framework, agent or terminal does not require touching any Rust code.

Every rule should come with a test case in [`rules/detection_tests.toml`](rules/detection_tests.toml), which `cargo test` runs automatically.

### Example PR: detect SvelteKit's dev server as front-end

1. Add the keyword to the front-end list in `rules/detection.toml`:

   ```diff
    [type.front]
    keywords = [
      "next dev",
   +  "svelte-kit dev",
      "next-server",
   ```

2. Add a case that would fail without it to `rules/detection_tests.toml`:

   ```toml
   [[type]]
   command = "node /work/web/node_modules/.bin/svelte-kit dev"
   expect = "front"
   ```

3. Run the tests and open a PR:

   ```bash
   cd src-tauri
   cargo test
   ```

The same applies to agents (`[[origin.agents]]`), terminals (`[origin.terminals]`, one list per system), shells (`[origin.shells]`) and system services (`[system_services.*]`). The comments at the top of each section explain how matching works (case-insensitive, whole words, `.exe` ignored).

There is an issue template, **Detection rule**, if you would rather ask for a rule than write it.

## Where the code lives

```
src/                    React + TypeScript panel
  i18n/en.ts, pt-BR.ts  every user-facing string
src-tauri/src/
  lib.rs                app setup: plugins, tray, global shortcut
  tray.rs               tray / menu bar icon and its menu
  panel.rs              showing, hiding and placing the panel window
  i18n.rs               the few native strings (tray menu)
  process.rs            ProcessInfo: plain data about a process
  scan.rs               joins ports and processes into the list the panel shows
  model.rs              PortProcess, the data sent to the panel (mirrors src/types.ts)
  commands.rs           the functions the panel calls with invoke()
  command.rs            running external programs without flashing a console
  ports.rs              listening TCP sockets and their PIDs
  worktree.rs           git worktree and branch of a folder (cached)
  classify.rs           front-end vs back-end
  origin.rs             who started a process
  actions.rs            kill, open in browser, open terminal
  platform/
    mod.rs              the `Platform` trait and `PlatformError`
    macos.rs, windows.rs, linux.rs
    unix.rs             signals, shared by macOS and Linux
```

The classification logic (`classify.rs`, `origin.rs`) receives data that was already collected, as plain structs, and never calls the operating system. That is why its tests run on any system.

### Platform-specific code

Everything that differs between macOS, Windows and Linux lives in `src-tauri/src/platform/`, behind the `Platform` trait (terminate, force kill, open a terminal, detect a system service). Each file is compiled only on its own system with `#[cfg(target_os = "...")]`.

To work on one system:

- You can only build and run the implementation for the system you are on. `cargo test` and `npm run tauri dev` on your machine exercise your platform file.
- CI builds and tests the three systems on every PR, so you will find out there if you broke another one.
- When you change behavior on a system, describe how you tested it manually in the PR (system version, desktop environment on Linux).

## Running the tests

```bash
# Rust: unit tests, including the detection rules
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test

# Front-end, from the repository root
npm run lint
npm test
```

## Commit messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: show the branch next to the worktree name
fix: deduplicate IPv4 and IPv6 sockets on Windows
docs: explain the GNOME AppIndicator extension
```

Common types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `ci`.

## Scope

Farol aims to stay small, fast and trustworthy.

**In scope:** listing local listening servers, identifying them (worktree, type, origin, uptime) and simple actions on them (stop, open, pin).

**Out of scope for now:** telemetry or any network access other than the probe to `localhost`, and asking for elevated privileges (`sudo`, UAC).

**Open discussions — open an issue before starting:** reading server logs, and integration with agents (for example an MCP server).

## Reporting a bug

Please use the **Bug report** issue template and include:

- your system and its version (e.g. macOS 15.1, Windows 11 23H2, Ubuntu 24.04);
- the architecture (x64 or ARM);
- on Linux, the desktop environment (GNOME, KDE, …);
- the output of your system's native port command, so we can compare it with what Farol shows:
  - macOS: `lsof -nP -iTCP -sTCP:LISTEN`
  - Linux: `ss -ltnp`
  - Windows: `netstat -ano -p TCP`

Feel free to remove anything you consider private (paths, process names) from that output.
