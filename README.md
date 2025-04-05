# HashSafe

A Rust application for calculating and displaying SHA-256 file hashes. This application is compatible with macOS and other operating systems, offering both a command-line interface and a native graphical interface.

## Features

- Fast and efficient SHA-256 hash calculation for files of any size
- Native graphical interface that adapts to each operating system
- Command-line mode for use in scripts or automation
- Asynchronous handling of large files to prevent interface blocking
- Functionality to copy the hash to the clipboard

## Requirements

- Rust 1.54 or higher
- Cargo (the Rust package manager)

## Installation

```bash
# Clone the repository (if applicable)
git clone https://github.com/guillerpsanchez/hashsafe.git
cd hashsafe

# Build the project
cargo build --release
```

## Usage

### Graphical Interface (Default)

To use the graphical interface, simply run:

```bash
cargo run --release
```

Or if you've already compiled the application:

```bash
./target/release/hashsafe
```

### Command Line

To calculate a file's hash from the command line:

```bash
cargo run --release -- --file path/to/file.ext
```

Or with the compiled binary:

```bash
./target/release/hashsafe --file path/to/file.ext
```

You can also force CLI mode even if the application has GUI support:

```bash
./target/release/hashsafe --cli --file path/to/file.ext
```

## Development

### Main Dependencies

- `sha2`: For SHA-256 hash calculation
- `hex`: For converting the hash to hexadecimal format
- `clap`: For processing command-line arguments
- `eframe`: For the native graphical user interface
- `rfd`: For native file selection dialogs

### Building without the graphical interface

If you only need the command-line version:

```bash
cargo build --release --no-default-features --features "cli"
```

## License

[MIT](LICENSE) or whatever applies to your project.