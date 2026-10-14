<div align="center">
  <img src="https://capsule-render.vercel.app/api?type=waving&color=gradient&height=200&section=header&text=Claude%20Manager&fontSize=60&animation=fadeIn" alt="Claude Manager Header" />
</div>

<div align="center">

[![Latest Release](https://img.shields.io/github/v/release/contrerasdaniel142/claude-manager)](https://github.com/contrerasdaniel142/claude-manager/releases/latest)
[![License](https://img.shields.io/github/license/contrerasdaniel142/claude-manager)](LICENSE)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)
[![Total Downloads](https://img.shields.io/github/downloads/contrerasdaniel142/claude-manager/total)](https://github.com/contrerasdaniel142/claude-manager/releases)

</div>

<div align="center">

## Download

[**Download Latest Release**](https://github.com/contrerasdaniel142/claude-manager/releases/latest)

> Each release includes installers for Windows, macOS and Linux, built automatically by CI.

</div>

## Why

Claude Code configuration is spread across JSON, YAML, and Markdown files in project and home directories. Hand-editing them is error-prone, difficult to discover, and cumbersome to maintain. Claude Manager provides a visual desktop interface to inspect, edit, and organize all your Claude Code settings seamlessly.

## Features

<details>
<summary><b>CLAUDE.md editor</b></summary>

- Visual editor for project-scoped (`.claude/`) and global (`~/.claude/`) CLAUDE.md files.
- Easily toggle and edit configuration instructions without raw file manipulation.

</details>

<details>
<summary><b>Rules</b></summary>

- Manage project-scoped and global rules with scope switching.
- Visual interface to add, edit, and organize prompt rules.

</details>

<details>
<summary><b>Hooks</b></summary>

- Configure lifecycle hooks across project and global scopes.
- Easily define execution triggers and scripts without hand-editing configs.

</details>

<details>
<summary><b>Agents</b></summary>

- Manage sub-agent configurations with project and global scope switching.
- Create, inspect, and update agent definitions cleanly.

</details>

<details>
<summary><b>Skills</b></summary>

- Manage skills with project-scoped and global scope switching.
- Built-in skills.sh marketplace browser with debounced search, inline SKILL.md preview fetched from GitHub, and one-click install.
- Talks directly to the skills.sh public search API and GitHub API.

</details>

<details>
<summary><b>Output Styles</b></summary>

- Manage and customize output styles with project and global scope switching.
- Visual controls for formatting and response style preferences.

</details>

<details>
<summary><b>Permissions</b></summary>

- View and adjust permission settings for tools and operations.
- Switch between project and global scopes to ensure appropriate access policies.

</details>

<details>
<summary><b>MCP Servers</b></summary>

- Configure Model Context Protocol (MCP) servers across project and global scopes.
- Supports importing MCP server configurations directly from a remote git repo.

</details>

<details>
<summary><b>Plugins</b></summary>

- Installed-plugins view plus a paginated and searchable marketplace view.
- Supports install, uninstall, enable, disable, and marketplace add/remove operations.
- Shells out to the real `claude plugin` CLI for state Claude Code already owns, with project and global scope switching.

</details>

## Tech stack

<div align="center">

![Tauri](https://img.shields.io/badge/Tauri-24C8DB?style=for-the-badge&logo=tauri&logoColor=white)
![React](https://img.shields.io/badge/React-61DAFB?style=for-the-badge&logo=react&logoColor=black)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=for-the-badge&logo=typescript&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-06B6D4?style=for-the-badge&logo=tailwindcss&logoColor=white)

</div>

Vite is the build tool.

## Development

```bash
npm install
npm run tauri dev
```

Run `cargo test` from the `src-tauri/` directory for Rust tests.

A stable Rust toolchain and Tauri 2 system prerequisites are required. See [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/).

## Releasing a new version

- Bump the version in all three files, which must match: `package.json` (`version` field), `src-tauri/Cargo.toml` (`[package]` version), `src-tauri/tauri.conf.json` (`version` field).
- Commit the change.
- Tag and push:

```bash
git tag vX.Y.Z && git push origin vX.Y.Z
```

- Pushing a tag matching `v*.*.*` triggers the release workflow at `.github/workflows/release.yml`, which builds installers for Windows, macOS (Apple Silicon and Intel) and Linux, and attaches them to a DRAFT GitHub Release for that tag. Publish the draft manually after review.

<!-- TODO: drop in a real screenshot or demo GIF of the app here -->
<!-- ![Claude Manager screenshot](path/to/screenshot.png) -->

## License

Licensed under the MIT License, see [LICENSE](LICENSE). Copyright Daniel Contreras.
