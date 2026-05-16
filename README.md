# WhispHub CLI

[![Crates.io](https://img.shields.io/crates/v/whisphub.svg)](https://crates.io/crates/whisphub)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Push your code projects to [WhispHub](https://whisphub.dev) directly from your terminal.

WhispHub is a code-sharing platform with a social twist — projects, not repositories. The CLI lets you push folders without Git, branches, or "create a repo first" friction.

```bash
$ whisphub push
✓ Uploaded 4 files
✓ Whispered to whisphub.dev/yourname/your-project
```

## Installation

```bash
cargo install whisphub
```

Requires Rust 1.70+.

## Quick start

```bash
# Authenticate (opens browser for one-time login)
whisphub login

# Initialize a new project in current folder
whisphub init

# Push your code
whisphub push
```

That's it. Your project is now live at `whisphub.dev/yourname/your-project`.

## Commands

| Command | Description |
|---|---|
| `whisphub login` | Authenticate via browser (device flow) |
| `whisphub logout` | Remove local credentials |
| `whisphub whoami` | Show currently logged-in user |
| `whisphub init` | Initialize a new project interactively |
| `whisphub push` | Upload current folder to WhispHub |

### `push` options

```bash
whisphub push --yes          # Skip confirmation prompt
whisphub push --slug myproj  # Override project slug
```

## Configuration

Credentials are stored at:

- macOS / Linux: `~/.config/whisphub/auth.json`
- Windows: `%APPDATA%\whisphub\auth.json`

File permissions are set to `0600` on Unix systems.

## What gets uploaded

The CLI uploads the current directory as a ZIP, excluding common build artifacts and metadata:

- `.git/`
- `node_modules/`
- `target/`
- `dist/`, `build/`
- `.DS_Store`
- Hidden files starting with `.` (except `.gitignore`, `.env.example`)

Maximum upload size: 100 MB.

## About WhispHub

WhispHub treats your code as a project, not a repository. No branches to manage, no Git knowledge required. Just push and share.

- Built solo in Rust
- Fast HTTP uploads with multipart streaming
- Token stored locally with restrictive permissions
- Cross-platform: macOS, Linux, Windows

## License

MIT — see [LICENSE](LICENSE).

## Links

- Website: [whisphub.dev](https://whisphub.dev)
- crates.io: [whisphub](https://crates.io/crates/whisphub)
- Issues: [github.com/whispem/whisphub-cli/issues](https://github.com/whispem/whisphub-cli/issues)