# Graft

Graft is a focused Git client for macOS on Apple silicon, built with Rust, Tauri, Vue, and the system Git installation. It preserves the dense, keyboard-oriented workflow of IntelliJ's Git tools while keeping application startup, memory use, and long-history navigation predictable.

## Development

Requirements: macOS 26 or newer, Apple silicon, Xcode Command Line Tools, Rust 1.92+, and Node.js 24+.

```sh
npm install
npm run app:dev
```

Run all local checks with `npm run check`. Create the arm64 app and DMG with `npm run app:build`.

The system `git` executable is the semantic source of truth for working-tree changes, hooks, signing, credentials, SSH, and repository operations. Graft does not send telemetry.

## Status

Graft is under active development. The current application includes paged virtualized history, commit details and patches, changelist-style staging and commits, branch/remotes navigation, worktree creation, merge/cherry-pick/revert/reset flows, interactive rebase planning, and a three-way conflict resolver.

## License

Licensed under either of Apache License 2.0 or MIT, at your option.

