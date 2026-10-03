# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-10-03

First version.

### Added

- Open source project structure: license, README, contributing guide, issue and PR templates.
- Detection rules in `rules/detection.toml`, with test cases in `rules/detection_tests.toml`.
- CI on macOS, Windows and Linux, and a release workflow that builds the installers (universal `.dmg`; `.exe` and `.msi`; `.deb`, `.rpm` and `.AppImage`) into a draft GitHub Release when a `v*` tag is pushed.
- Tray / menu bar icon. On macOS and Windows a click toggles the panel; on Linux the icon opens a menu with "Open panel".
- Global shortcut to toggle the panel: `Ctrl+Alt+P` (`Cmd+Option+P` on macOS).
- Borderless panel window that hides when it loses focus or on `Esc`, placed next to the tray icon (top center of the screen on Linux), with no Dock or taskbar entry.
- `Platform` trait isolating what differs between macOS, Windows and Linux (terminate, force kill, open a terminal, detect system services).
- Lists every process listening on a TCP port (port, PID, name, command), refreshed every 3 seconds. IPv4 and IPv6 sockets of the same server are shown once. Falls back to `lsof`, `ss` or `netstat` if reading sockets directly fails.
- Stop a process from the panel with an inline confirmation: **Stop** asks it to exit (`SIGTERM`, `taskkill`), **Force** ends it (`SIGKILL`, `taskkill /F`). If the port is still open 3 seconds after Stop, the row offers to force it. The stop button is disabled for processes of other users.
- Groups processes by git worktree (name, branch and path), with processes outside a repository at the end. Each folder is resolved with `git` once (cached for a minute, in parallel); without git the list still works, just without groups.
- Shows how long each server has been running ("2h ago", "há 2 h"); after 24 hours it turns amber with "forgotten?".
- Front-end / back-end detection: keywords from `rules/detection.toml` (longest match wins), then a single `GET /` to `localhost` (HTML means front-end), cached per process. Click the label to correct it; the choice is saved and survives server restarts.
- Main action per row: front-ends open `http://localhost:<port>` in the browser, back-ends open a terminal in the process folder (Terminal on macOS; Windows Terminal or the console on Windows; the first installed terminal on Linux).
- Shows who started each server: Claude Code, Cursor, VS Code, a terminal or the system (database services and the like), by walking up the process tree. Agents are highlighted. New agents can be added in `rules/detection.toml` alone.
- Pin processes (star): pinned ones stay at the top and survive restarts.
- Search by port, name, command, worktree, branch or origin.
- Keyboard: arrow keys move between rows, `Esc` closes the panel.
- Dark theme following the system.
- Number of open ports next to the tray icon (macOS menu bar, Linux where supported) and in its tooltip (all systems, the only option on Windows). It updates every 3 seconds with the panel open and every 15 seconds with it closed.
- "Start at login" option in the tray menu.
- The global shortcut can be changed in the preferences file.
- Origin detection skips shells (`cmd`, `powershell`, `pwsh`) and only uses them as a fallback, so agents that start servers through a shell are still recognized.

[Unreleased]: https://github.com/wendesongomes/farol/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/wendesongomes/farol/releases/tag/v0.1.0
