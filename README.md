# Farol

**Farol — see every local server running on your machine, which worktree it came from, and which AI agent started it.**

[Português (Brasil)](README.pt-BR.md)

> **Status:** early development. There is no published release yet; for now, [run it from source](#run-from-source).

![Farol panel](docs/screenshot.png)

## Why it exists

AI coding agents (Claude Code, Cursor, …) start dev servers in the background, and it is easy to lose track of what is still running. Then you start something and the port is already taken. It gets worse with several Git worktrees, each with its own front-end and back-end.

Farol lives in your menu bar (macOS) or system tray (Windows and Linux) and shows every process listening on a TCP port, so you can see where it came from and stop it.

## Features

- Lists every process listening on a TCP port, grouped by Git worktree.
- Tells front-end from back-end (and lets you correct it with one click).
- Shows who started each server: Claude Code, Cursor, VS Code, a terminal or the system.
- Shows uptime and flags servers running for more than 24 hours ("forgotten?").
- Stop (gracefully or forcefully), open in the browser, open a terminal in the process folder.
- Pin processes, search, light and dark themes, global shortcut, start with the system.
- No telemetry, no network access (see [Privacy](#privacy)).

## Where it works

| System | Versions | Notes |
|---|---|---|
| **macOS** | 10.15 (Catalina) or newer, Apple Silicon and Intel | Lives in the menu bar. |
| **Windows** | 10 and 11 (x64) | Needs WebView2, which ships with Windows 11 and with Windows 10 since version 1803. If it is missing, the installer downloads it. |
| **Linux** | x64 with `webkit2gtk-4.1` (e.g. Ubuntu 22.04+, Debian 12+, recent Fedora) | Tray icon via AppIndicator. On GNOME, the [AppIndicator and KStatusNotifierItem Support](https://extensions.gnome.org/extension/615/appindicator-support/) extension is required (already enabled on Ubuntu). |

## Installation

Download the installer for your system from the [Releases](https://github.com/<user>/farol/releases) page.

### macOS

1. Download the `.dmg` and drag **Farol** to **Applications**.
2. Farol is not signed and notarized yet, so the first time you open it macOS blocks it as coming from an "unidentified developer". To allow it, either:
   - open **System Settings › Privacy & Security** and click **Open Anyway**; or
   - run in a terminal:
     ```bash
     xattr -dr com.apple.quarantine /Applications/Farol.app
     ```

### Windows

1. Download the `.exe` installer (or the `.msi`).
2. Farol is not signed yet, so SmartScreen shows "Windows protected your PC". Click **More info › Run anyway**.

### Linux

- **`.deb`** (Debian/Ubuntu):
  ```bash
  sudo apt install ./farol_<version>_amd64.deb
  ```
- **`.rpm`** (Fedora):
  ```bash
  sudo dnf install ./farol-<version>.x86_64.rpm
  ```
- **`.AppImage`** (any distro):
  ```bash
  chmod +x Farol_<version>.AppImage
  ./Farol_<version>.AppImage
  ```

### Package managers

Homebrew, winget, Scoop and AUR: coming soon.

## First use

- **Where the icon is:**
  - **macOS:** in the menu bar, at the top right.
  - **Windows:** in the system tray, next to the clock. Windows may hide it behind the `^` arrow; to keep it visible, drag it from there onto the taskbar (or enable it in **Settings › Personalization › Taskbar › Other system tray icons**).
  - **Linux:** in the tray / top panel, depending on your desktop.
- **Opening the panel:**
  - **macOS and Windows:** click the icon.
  - **Linux:** most desktops don't send clicks to tray icons, so the icon opens a menu: choose **Open panel**.
  - **Everywhere:** global shortcut `Ctrl+Alt+P` (`Cmd+Option+P` on macOS).
- **Start with the system:** use the **Start at login** option in Farol's own menu.

## Privacy

Farol collects no data and does not access the internet. It only reads local process information and, to tell front-ends from back-ends, makes one HTTP request to `localhost` on the port being inspected. Fonts and every other asset are bundled with the app.

Farol never asks for elevated privileges (`sudo`, UAC). Processes owned by other users are listed when the system allows it, with their actions disabled.

## Run from source

### Prerequisites

Common to every system:

- **Rust**, installed with [rustup](https://rustup.rs).
- **Node.js** LTS (20 or newer) with npm.

#### macOS

```bash
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Windows

1. Install the [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and check the **Desktop development with C++** workload.
2. WebView2: already present on Windows 11 and Windows 10 (1803+). Otherwise, install the "Evergreen Bootstrapper" from the [WebView2 page](https://developer.microsoft.com/microsoft-edge/webview2/).
3. Install Rust with [`rustup-init.exe`](https://www.rust-lang.org/tools/install) and keep the MSVC toolchain as the default (`rustup default stable-msvc`).

#### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Linux (Fedora)

```bash
sudo dnf check-update
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
  libappindicator-gtk3-devel librsvg2-devel libxdo-devel
sudo dnf group install "c-development"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Linux (Arch)

```bash
sudo pacman -Syu
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl \
  appmenu-gtk-module libappindicator-gtk3 librsvg xdotool
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

These lists come from the [official Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/); check there for other distributions.

### Build and run

```bash
git clone https://github.com/<user>/farol.git
cd farol
npm install
npm run tauri dev      # development mode
npm run tauri build    # installers in src-tauri/target/release/bundle
```

Universal macOS build (Apple Silicon + Intel):

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm run tauri build -- --target universal-apple-darwin
```

## Uninstall

Farol's app identifier is `io.github.wendesongomes.farol`.

- **macOS:** drag Farol from Applications to the Trash and delete `~/Library/Application Support/io.github.wendesongomes.farol`.
- **Windows:** **Settings › Apps › Farol › Uninstall**. Data is kept in `%APPDATA%\io.github.wendesongomes.farol`; delete it to remove everything.
- **Linux:** remove it with your package manager (or delete the AppImage). Data is kept in `~/.local/share/io.github.wendesongomes.farol`.

## Contributing

Contributions are welcome — adding a detection rule for a new framework, agent or terminal is a one-line change. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
