# todo-cli
[![Build Status](https://github.com/Agustin-Mediotti/todo-app/workflows/Rust/badge.svg?branch=master)](https://github.com/Agustin-Mediotti/todo-app/actions?query=branch%3Amaster)

A simple TODO app for the terminal written in Rust. This project is designed to help you organize your tasks efficiently through a command-line interface.

## Features

- Create, read, update, and delete tasks.
- Persistent storage of tasks.
- Interactive UI using Ratatui and Crossterm.
- Customizable views.

## Installation

1. Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed on your system.
2. Clone this repository
3. Navigate to the project directory and build the application:

```bash
cd todo-app
cargo build --release
```

4. Run the application:

```bash
./target/release/todo-app
```

## Development

This project uses the following dependencies:

- [Ratataui](https://crates.io/crates/ratatui)
- [Crossterm](https://crates.io/crates/crossterm)
- [Serde](https://crates.io/crates/serde) and [Serde JSON](https://crates.io/crates/serde_json)
- [Color-eyre](https://crates.io/crates/color-eyre)
- [Throbber-widgets-tui](https://crates.io/crates/throbber-widgets-tui)

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more information.

