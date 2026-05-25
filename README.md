# Taurin

Taurin is an experimental Tauri-based desktop runtime for running RPG Maker MV projects.

It currently loads the RPG Maker MV project in the `www` directory through the custom `rpgmv://` protocol and bridges part of the audio behavior to native Rust/Tauri code. The long-term goal is to keep compatibility with existing MV projects while gradually moving browser-dependent systems such as asset loading, audio, storage, and rendering into the native runtime.

## Installation (for default users)

1. Open the **Releases** tab for this project.
2. Download the runtime file that matches your operating system.
3. Replace the existing runtime file with the downloaded one.
4. Run the replaced runtime next to your RPG Maker MV `www` directory.

## Development

Requirements:

- Node.js
- pnpm
- Rust
- Tauri development environment

Install dependencies:

```sh
pnpm install
```

Run in development mode:

```sh
pnpm tauri dev
```

Run only the frontend:

```sh
pnpm dev
```

Build:

```sh
pnpm tauri build
```

## Project Structure

- `www/`: RPG Maker MV project data and assets
- `src/`: TypeScript frontend used by the Tauri WebView
- `src-tauri/`: Rust Tauri runtime, custom protocol, bridge commands, and audio handling

## TODO

- Add RPG Maker MZ project support.
- Move file system access and project file handling into Rust.
- Continue replacing browser-dependent runtime behavior with native Tauri/Rust services.
