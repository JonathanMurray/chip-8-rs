# chip-8-rs

_A Chip-8 emulator/debugger written in Rust_

Run and debug Chip-8 games, see register contents and disassembled instructions:

<img
    src="https://github.com/JonathanMurray/chip-8-rs/blob/master/resources/screenshots/example_screenshot_pong_debug.png"
    height="300"
/>
<br/>

## Usage

This program requires an installation of the Rust language.

Run a Chip-8 program:
```bash
cargo run --release --bin emulator
```

... and with the debugger enabled:
```bash
cargo run --release --bin emulator -- --debug
```

To see all run options:
```bash
cargo run --release --bin emulator -- --help
```