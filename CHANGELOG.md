# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Open source project structure: license, README, contributing guide, issue and PR templates.
- Detection rules in `rules/detection.toml`, with test cases in `rules/detection_tests.toml`.
- CI on macOS, Windows and Linux.
- Tray / menu bar icon. On macOS and Windows a click toggles the panel; on Linux the icon opens a menu with "Open panel".
- Global shortcut to toggle the panel: `Ctrl+Alt+P` (`Cmd+Option+P` on macOS).
- Borderless panel window that hides when it loses focus or on `Esc`, placed next to the tray icon (top center of the screen on Linux), with no Dock or taskbar entry.
- `Platform` trait isolating what differs between macOS, Windows and Linux (terminate, force kill, open a terminal, detect system services).
- Lists every process listening on a TCP port (port, PID, name, command), refreshed every 3 seconds. IPv4 and IPv6 sockets of the same server are shown once. Falls back to `lsof`, `ss` or `netstat` if reading sockets directly fails.
- Stop a process from the panel with an inline confirmation: **Stop** asks it to exit (`SIGTERM`, `taskkill`), **Force** ends it (`SIGKILL`, `taskkill /F`). If the port is still open 3 seconds after Stop, the row offers to force it. The stop button is disabled for processes of other users.
- Origin detection skips shells (`cmd`, `powershell`, `pwsh`) and only uses them as a fallback, so agents that start servers through a shell are still recognized.
