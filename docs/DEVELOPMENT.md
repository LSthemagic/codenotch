# Desenvolvimento e instalação para testes

## Escopo

Nyrva é um aplicativo Rust/Tauri 2 para Windows e Linux **X11**. A interface estática fica em `nyrva/ui`; não é necessário introduzir um build Node.js para este fluxo. Os comandos abaixo partem da raiz do repositório, salvo indicação contrária.

Os arquivos de configuração e scripts do repositório são a referência para o build. O CI usa Rust stable, Tauri CLI 2 e o `Cargo.lock` versionado. Isso descreve uma receita repetível, não uma garantia de binários idênticos bit a bit: runners, toolchain e pacotes do sistema podem mudar.

## Windows

Instale Git, Rust com toolchain MSVC, Visual Studio Build Tools com desenvolvimento C++ para desktop e Windows SDK, além do WebView2 Runtime. Para reproduzir a inspeção do instalador do CI, disponibilize também `7z` no PATH. Consulte os [pré-requisitos oficiais do Tauri](https://v2.tauri.app/start/prerequisites/).

No PowerShell, prepare o helper antes de compilar o aplicativo:

```powershell
./scripts/prepare-windows-sidecar.ps1 debug
cargo check --workspace
cargo test --workspace
cargo install tauri-cli --version "^2.0.0" --locked
cd nyrva
cargo tauri dev
```

Para gerar o instalador, volte à raiz e execute:

```powershell
./scripts/prepare-windows-sidecar.ps1 release
cd nyrva
cargo tauri build
```

O instalador fica em `target/release/bundle/nsis/`. Execute o `.exe` e teste a aplicação **instalada**, não apenas o binário de desenvolvimento. Não desative mecanismos de segurança do sistema para contornar um alerta; verifique origem, hash e procedência do build. O fluxo atual não assina os pacotes.

## Linux X11

A lista abaixo reproduz o job Ubuntu 22.04. Ela também orienta Debian 12, mas não substitui o teste nessa distribuição. Em outras versões, confira os nomes dos pacotes nos repositórios oficiais.

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev \
  librsvg2-dev patchelf libfuse2 xdg-utils
```

Instale Git e Rust stable. A sessão usada para os testes de desktop deve ser X11:

```bash
printf '%s\n' "$XDG_SESSION_TYPE"
```

Uma sessão Wayland com aplicações XWayland não comprova suporte ao desktop X11 definido neste MVP. Selecione uma sessão X11 na tela de login quando disponível.

Prepare e valide o workspace:

```bash
bash scripts/prepare-linux-sidecar.sh debug
cargo check --workspace
cargo test --workspace
git diff --exit-code -- Cargo.lock
cargo install tauri-cli --version '^2.0.0' --locked
cd nyrva
cargo tauri dev
```

Para empacotar, a partir da raiz:

```bash
bash scripts/prepare-linux-sidecar.sh release
cd nyrva
cargo tauri build
```

As saídas ficam em `target/release/bundle/deb/` e `target/release/bundle/appimage/`. Na pasta que contém o pacote baixado, use o nome real do arquivo:

```bash
sudo apt install ./NOME_DO_ARQUIVO.deb
chmod +x ./NOME_DO_ARQUIVO.AppImage
./NOME_DO_ARQUIVO.AppImage
```

Teste `.deb` e AppImage separadamente. Mantenha o AppImage em um local estável durante o teste de inicialização automática e relançamento pelo hook. Não execute Nyrva como root. A validação visual exige sessão gráfica e interação; o runner de CI não substitui esse ambiente.

## Helper do Claude Code

`nyrva-hook` é compilado pelos scripts `prepare-*-sidecar` e disponibilizado ao Tauri com o sufixo do target triple. Os pacotes devem conter o helper; o CI inspeciona os instaladores para verificar sua presença.

Instale e remova a integração pelas opções do próprio aplicativo. Faça backup das configurações do Claude antes do teste, confira se hooks alheios são preservados e teste o relançamento com a Nyrva fechada. A instalação explícita do hook é uma alteração de configuração, não uma autorização para modificar tokens ou bancos dos provedores.

## Condições dos provedores

| Provedor | Pré-condições e comportamento a verificar |
|---|---|
| Claude Code | Instalação/autenticação próprias e hook da Nyrva configurado; conferir atividade e relançamento pelo helper instalado. Claude Desktop no Linux não está no escopo. |
| Codex | Autenticação/estado existentes em `CODEX_HOME` não vazio ou `~/.codex`; uso e atividade devem usar a mesma raiz. Nyrva não deve executar login nem renovar o token. |
| Cursor | No Windows, `%APPDATA%/Cursor/User/globalStorage/state.vscdb`; no Linux, `${XDG_CONFIG_HOME:-~/.config}/Cursor/User/globalStorage/state.vscdb`. O banco deve ser lido sem escrita, inclusive com o Cursor aberto. |
| Antigravity | Prioridade para o language server local em execução. No Linux, acesso ao Secret Service quando disponível; estado/transcrições em `~/.gemini/antigravity` como alternativa. Chaveiro bloqueado ou ausência do serviço não deve derrubar o app. |

APIs, contas e formatos locais dos provedores podem mudar. Transcrições/atividade não são equivalentes a uma quota oficial; dados antigos ou ausentes não devem ser apresentados como uma leitura atual válida. Não copie `auth.json`, tokens, bancos privados ou saídas com credenciais para issues, PRs ou evidências públicas.

## Testes de release

Python 3.11+ é necessário apenas para as ferramentas de release e seus testes, não para executar Nyrva:

```bash
python -m unittest discover -s tests -p 'test_release_tools.py' -v
```

Os testes usam arquivos temporários e não acessam provedores nem publicam releases. O CI também executa a suíte Rust e inspeciona o helper dentro dos pacotes. Execute [SMOKE_TESTS.md](SMOKE_TESTS.md) separadamente antes da publicação.
