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

Learn about more flags/options:
```bash
cargo run --release --bin emulator -- --help
```

### Disassembler

Disassemble a C8 program to a text file:
```bash
$ cargo run --quiet --bin disassembler programs/c8_test.c8 c8_test_disassembly.txt
Wrote 23 instructions to c8_test_disassembly.txt
$ cat c8_test_disassembly.txt | head -5
200: V0 = 0xFF
202: I = delay_timer(V0)
204: V0 = 0x00
206: V9 = 0x00
208: VE = 0x00
```