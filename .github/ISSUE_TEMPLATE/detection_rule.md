---
name: Detection rule
about: Add support for a framework, agent or terminal
title: "rules: detect "
labels: detection-rule, good first issue
---

<!--
Farol's detection rules live in rules/detection.toml. Adding one is usually
a one-line change plus a test case: see CONTRIBUTING.md if you want to send
the PR yourself.
-->

## What should be detected

- [ ] Framework or server as **front-end**
- [ ] Framework or server as **back-end**
- [ ] AI agent / editor (origin)
- [ ] Terminal (origin)
- [ ] System service

Name: <!-- e.g. SvelteKit, Zed, Ghostty -->

## How it shows up

<!--
For a framework: the full command line of the process listening on the port.
For an agent or terminal: the executable name and command line of the process.

macOS / Linux: ps -o pid,ppid,comm,args -p <pid>
Windows:       Get-CimInstance Win32_Process -Filter "ProcessId=<pid>" | Select ProcessId,ParentProcessId,Name,CommandLine
-->

```

```

## System

<!-- macOS / Windows / Linux, and version -->
