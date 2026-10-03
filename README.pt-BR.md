# Farol

**Farol — veja todos os servidores locais rodando na sua máquina, de qual worktree vieram e qual agente de IA os iniciou.**

[English](README.md)

> **Status:** em desenvolvimento inicial. Ainda não há release publicada; por enquanto, [rode a partir do código-fonte](#rodar-a-partir-do-código-fonte).

![Painel do Farol](docs/screenshot.png)

## Por que existe

Agentes de IA (Claude Code, Cursor, …) sobem servidores de desenvolvimento em background, e é fácil perder o controle do que continua rodando. Aí você vai iniciar algo e a porta já está ocupada. Fica pior com várias Git worktrees, cada uma com seu front-end e seu back-end.

O Farol fica na barra de menu (macOS) ou na bandeja do sistema (Windows e Linux) e mostra todos os processos escutando em portas TCP, para você ver de onde vieram e encerrá-los.

## Funcionalidades

- Lista todos os processos escutando em portas TCP, agrupados por Git worktree.
- Diferencia front-end de back-end (e deixa você corrigir com um clique).
- Mostra quem subiu cada servidor: Claude Code, Cursor, VS Code, um terminal ou o sistema.
- Mostra o tempo de execução e destaca servidores rodando há mais de 24 horas ("esquecido?").
- Encerrar (com educação ou à força), abrir no navegador, abrir um terminal na pasta do processo.
- Fixar processos, busca, temas claro e escuro, atalho global, iniciar com o sistema.
- Sem telemetria e sem acesso à rede (veja [Privacidade](#privacidade)).

## Onde funciona

| Sistema | Versões | Observações |
|---|---|---|
| **macOS** | 10.15 (Catalina) ou mais recente, Apple Silicon e Intel | Fica na barra de menu. |
| **Windows** | 10 e 11 (x64) | Precisa do WebView2, que já vem no Windows 11 e no Windows 10 desde a versão 1803. Se faltar, o instalador baixa. |
| **Linux** | x64 com `webkit2gtk-4.1` (ex.: Ubuntu 22.04+, Debian 12+, Fedora recente) | Ícone de bandeja via AppIndicator. No GNOME, exige a extensão [AppIndicator and KStatusNotifierItem Support](https://extensions.gnome.org/extension/615/appindicator-support/) (já ativa no Ubuntu). |

## Instalação

Baixe o instalador do seu sistema na página de [Releases](https://github.com/<usuario>/farol/releases).

### macOS

1. Baixe o `.dmg` e arraste o **Farol** para **Aplicativos**.
2. O Farol ainda não é assinado nem notarizado, então na primeira abertura o macOS bloqueia o app como de "desenvolvedor não identificado". Para liberar, escolha um dos jeitos:
   - abra **Ajustes do Sistema › Privacidade e Segurança** e clique em **Abrir mesmo assim**; ou
   - rode no terminal:
     ```bash
     xattr -dr com.apple.quarantine /Applications/Farol.app
     ```

### Windows

1. Baixe o instalador `.exe` (ou o `.msi`).
2. O Farol ainda não é assinado, então o SmartScreen mostra "O Windows protegeu o computador". Clique em **Mais informações › Executar assim mesmo**.

### Linux

- **`.deb`** (Debian/Ubuntu):
  ```bash
  sudo apt install ./farol_<versão>_amd64.deb
  ```
- **`.rpm`** (Fedora):
  ```bash
  sudo dnf install ./farol-<versão>.x86_64.rpm
  ```
- **`.AppImage`** (qualquer distro):
  ```bash
  chmod +x Farol_<versão>.AppImage
  ./Farol_<versão>.AppImage
  ```

### Gerenciadores de pacote

Homebrew, winget, Scoop e AUR: em breve.

## Primeiro uso

- **Onde fica o ícone:**
  - **macOS:** na barra de menu, no canto superior direito.
  - **Windows:** na bandeja do sistema, perto do relógio. O Windows pode escondê-lo na setinha `^`; para deixá-lo sempre visível, arraste-o de lá para a barra de tarefas (ou ative-o em **Configurações › Personalização › Barra de tarefas › Outros ícones da bandeja do sistema**).
  - **Linux:** na bandeja / painel superior, conforme o ambiente gráfico.
- **Abrir o painel:**
  - **macOS e Windows:** clique no ícone.
  - **Linux:** a maioria dos ambientes não envia cliques para ícones da bandeja, então o ícone abre um menu: escolha **Abrir painel**.
  - **Em todos:** atalho global `Ctrl+Alt+P` (`Cmd+Option+P` no macOS).
- **Iniciar com o sistema:** pela opção **Iniciar com o sistema** no menu do próprio Farol.

## Privacidade

O Farol não coleta dados e não acessa a internet. Ele só lê informações de processos locais e, para diferenciar front-ends de back-ends, faz uma requisição HTTP a `localhost` na porta analisada. As fontes e todos os outros recursos vêm empacotados no app.

O Farol nunca pede elevação de privilégio (`sudo`, UAC). Processos de outros usuários aparecem na lista quando o sistema permite, com as ações desabilitadas.

## Rodar a partir do código-fonte

### Pré-requisitos

Comuns a todos os sistemas:

- **Rust**, instalado pelo [rustup](https://rustup.rs).
- **Node.js** LTS (20 ou mais recente) com npm.

#### macOS

```bash
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Windows

1. Instale o [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) marcando o workload **Desenvolvimento para desktop com C++**.
2. WebView2: já presente no Windows 11 e no Windows 10 (1803+). Caso contrário, instale o "Evergreen Bootstrapper" da [página do WebView2](https://developer.microsoft.com/microsoft-edge/webview2/).
3. Instale o Rust pelo [`rustup-init.exe`](https://www.rust-lang.org/tools/install) e mantenha o toolchain MSVC como padrão (`rustup default stable-msvc`).

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

Essas listas vêm dos [pré-requisitos oficiais do Tauri 2](https://v2.tauri.app/start/prerequisites/); consulte lá para outras distribuições.

### Compilar e rodar

```bash
git clone https://github.com/<usuario>/farol.git
cd farol
npm install
npm run tauri dev      # modo desenvolvimento
npm run tauri build    # gera os instaladores em src-tauri/target/release/bundle
```

Build universal no macOS (Apple Silicon + Intel):

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm run tauri build -- --target universal-apple-darwin
```

## Desinstalar

O identificador do app é `io.github.wendesongomes.farol`.

- **macOS:** arraste o Farol de Aplicativos para o Lixo e apague `~/Library/Application Support/io.github.wendesongomes.farol`.
- **Windows:** **Configurações › Aplicativos › Farol › Desinstalar**. Os dados ficam em `%APPDATA%\io.github.wendesongomes.farol`; apague a pasta para remover tudo.
- **Linux:** remova pelo gerenciador de pacotes (ou apague o AppImage). Os dados ficam em `~/.local/share/io.github.wendesongomes.farol`.

## Contribuir

Contribuições são bem-vindas — adicionar uma regra de detecção para um framework, agente ou terminal novo é uma mudança de uma linha. Veja o [CONTRIBUTING.md](CONTRIBUTING.md).

## Licença

[MIT](LICENSE)
