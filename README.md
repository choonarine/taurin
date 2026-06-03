# Taurin™ (WIP)

> [!WARNING]
> Taurin is still in active development and may contain bugs, missing features, or compatibility issues. If you would like to try Taurin with your project, please test it thoroughly before using it in production.

Taurin is an experimental Tauri-based runtime for RPG Maker projects.

This project aims to improve performance and reduce memory usage by replacing parts of the original NW.js/Node.js runtime and performance-critical JavaScript code with native Rust/Tauri implementations.

The long-term goal is to maintain compatibility with existing RPG Maker projects while gradually moving browser-dependent systems such as asset loading, audio, storage, and rendering into the native runtime.

> [!NOTE]
> Taurin currently supports RPG Maker MV. Support for RPG Maker MZ is currently under development. (*Coming Soon™*)
>
> The `™` in the project name is not intended as a trademark claim.

## Installation

### Manual Installation

1. Open the **Releases** page for this project.
2. Download the Taurin release for your operating system.
3. Place the downloaded runtime executable in the same folder as your RPG Maker project's www directory.
4. Run the Taurin executable. Enjoy the game!

### For Commercial Use

If you sell or publish a game using Taurin, please consider including the project's `LICENSE` file or mentioning `Taurin` in the game's credits. Attribution is appreciated, but not required.

## Development

To run Taurin locally, you will need an RPG Maker project. Place the project's `www` directory next to the development executable.

Example:

```text
src-tauri/
└─ target/
   └─ debug/
      ├─ taurin.exe
      └─ www/
```

### Requirements

* Node.js
* pnpm
* Rust
* Tauri development environment

### Dependencies Installation

```sh
pnpm install
```

### Run in Development Mode

```sh
pnpm tauri dev
```

### Run Frontend Only

```sh
pnpm dev
```

### Build

```sh
pnpm tauri build
```

## Contributing

Taurin is primarily a performance and compatibility focused project.

When introducing new features or runtime changes, priority should be given to:

* Maintaining compatibility with existing RPG Maker MV projects.
* Reducing memory usage and startup overhead.
* Replacing performance-critical JavaScript code with native implementations where appropriate.
* Providing measurable improvements over the original NW.js-based runtime.
* Avoiding unnecessary complexity when a simpler solution is sufficient.

Performance-related changes should ideally be backed by profiling data, benchmarks, or a clearly identified bottleneck.

## Project Structure

* `www/`: RPG Maker MV project data and assets (must be located alongside the Taurin executable)
* `src/`: TypeScript frontend used by the Tauri WebView
* `src-tauri/`: Rust Tauri runtime, custom protocol, bridge commands, and audio handling, etc.

## Roadmap

The current development focus is compatibility with existing RPG Maker MV projects.

In particular, support for the core RPG Maker MV runtime and commonly used Yanfly Engine Plugins (YEP) is a high priority.

* Add support for RPG Maker MZ projects.
* Achieve full compatibility with the Node.js and NW.js APIs commonly used by commercial RPG Maker games.
* Move file system access and project file handling into Rust.
* Continue replacing browser-dependent runtime behavior with native Rust/Tauri services.

## License

This project is licensed under the BSD 3-Clause License. See the [LICENSE](LICENSE) file for details.
