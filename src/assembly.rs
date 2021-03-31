use std::collections::HashMap;

pub fn disassemble_rom(buffer: Vec<u8>) -> HashMap<usize, String> {
    let mut disassembled = HashMap::new();

    let mut visited = Vec::new();
    let mut pc = 0x200;
    loop {
        if pc < 0x200 || pc - 0x200 + 1 >= buffer.len() {
            break;
        }
        let offset = pc - 0x200;
        let opcode = ((buffer[offset] as u16) << 8) | buffer[offset + 1] as u16;
        let text = match disassemble_opcode(opcode) {
            Ok(s) => format!("{}", &s),
            Err(_err) => format!("DATA[{:#06X}]", opcode),
        };

        disassembled.insert(0x200 + offset, text);

        if opcode & 0xF000 == 0x1000 {
            // We follow the jump instruction (it may point to an unaligned address)
            let destination = (opcode & 0x0FFF) as usize;
            if !visited.contains(&destination) {
                visited.push(destination);
                pc = destination;
            } else {
                pc += 2;
            }
        } else {
            pc += 2;
        }
    }
    disassembled
}

pub fn disassemble_opcode(opcode: u16) -> Result<String, String> {
    let s = match opcode & 0xF000 {
        0x0000 => match opcode {
            0x00ee => "return".to_owned(),
            0x00e0 => "clear screen".to_owned(),
            _ => {
                let address = opcode & 0x0FFF;
                format!("call (machine): {:#05X}", address)
            }
        },
        0x1000 => {
            let address = opcode & 0x0FFF;
            format!("jump: {:#05X}", address)
        }
        0x2000 => {
            let address = opcode & 0x0FFF;
            format!("call: {:#05X}", address)
        }
        0x3000 => {
            let a = ((opcode & 0x0F00) >> 8) as usize;
            let constant = (opcode & 0x00FF) as u8;
            format!("skip if V{:X} == {:#04X}", a, constant)
        }
        0x4000 => {
            let a = ((opcode & 0x0F00) >> 8) as usize;
            let constant = (opcode & 0x00FF) as u8;
            format!("skip if V{:X} != {:#04X}", a, constant)
        }
        0x5000 => {
            let a = ((opcode & 0x0F00) >> 8) as usize;
            let b = ((opcode & 0x00F0) >> 4) as usize;
            format!("skip if V{:X} == V{:X}", a, b)
        }
        0x6000 => {
            let a = ((opcode & 0x0F00) >> 8) as usize;
            let constant = (opcode & 0x00FF) as u8;
            format!("V{:X} = {:#04X}", a, constant)
        }
        0x7000 => {
            let a = ((opcode & 0x0F00) >> 8) as usize;
            let constant = (opcode & 0x00FF) as u8;
            format!("V{:X} += {:#04X}", a, constant)
        }
        0x8000 => match opcode & 0x000F {
            0x0 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let b = ((opcode & 0x00F0) >> 4) as usize;
                format!("V{:X} = V{:X}", a, b)
            }
            0x1 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let b = ((opcode & 0x00F0) >> 4) as usize;
                format!("V{:X} = V{:X} | V{:X}", a, a, b)
            }
            0x2 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let b = ((opcode & 0x00F0) >> 4) as usize;
                format!("V{:X} = V{:X} & V{:X}", a, a, b)
            }
            0x3 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let b = ((opcode & 0x00F0) >> 4) as usize;
                format!("V{:X} = V{:X} ^ V{:X}", a, a, b)
            }
            0x4 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let b = ((opcode & 0x00F0) >> 4) as usize;
                format!("V{:X} = V{:X} + V{:X}", a, a, b)
            }
            0x5 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let b = ((opcode & 0x00F0) >> 4) as usize;
                format!("V{:X} = V{:X} - V{:X}", a, a, b)
            }
            0x6 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                format!("V{:X} >>= 1", a)
            }
            0x7 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let b = ((opcode & 0x00F0) >> 4) as usize;
                format!("V{:X} = V{:X} - V{:X}", a, b, a)
            }
            0xE => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                format!("V{:X} <<= 1", a)
            }
            _ => return Err(format!("Unhandled op-code: {:#06X}", opcode)),
        },
        0x9000 => {
            let a = ((opcode & 0x0F00) >> 8) as usize;
            let b = ((opcode & 0x00F0) >> 4) as usize;
            format!("skip if V{:X} != V{:X}", a, b)
        }
        0xA000 => {
            let address = opcode & 0x0FFF;
            format!("I = {:#04X}", address)
        }
        0xB000 => {
            let address = opcode & 0x0FFF;
            format!("jump to V0 + {:#04X}", address)
        }
        0xC000 => {
            let a = ((opcode & 0x0F00) >> 8) as usize;
            let constant = (opcode & 0x00FF) as u8;
            format!("V{:#04X} = rand() & {:#04X}", a, constant)
        }
        0xD000 => {
            let vx = ((opcode & 0x0F00) >> 8) as usize;
            let vy = ((opcode & 0x00F0) >> 4) as usize;
            let height = (opcode & 0x000F) as u8;
            format!("render(V{}, V{}, {})", vx, vy, height)
        }
        0xE000 => match opcode & 0x00FF {
            0x9E => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                format!("skip if V{:X} pressed", a)
            }
            0xA1 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                format!("skip if V{:X} not pressed", a)
            }
            _ => return Err(format!("Unhandled op-code: {:#06X}", opcode)),
        },
        0xF000 => match opcode & 0x00FF {
            0x07 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                format!("V{:X} = get_delay()", a)
            }
            0x0A => {
                let a = ((opcode & 0x0F00) >> 8) as u8;
                format!("V{:X} = get_key()", a)
            }
            0x15 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                format!("I = delay_timer(V{:X})", a)
            }
            0x18 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                format!("I = sound_timer(V{:X})", a)
            }
            0x1E => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                format!("I += V{:X}", a)
            }
            0x29 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                format!("I = sprite_addr(V{:X})", a)
            }
            0x33 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                format!("BCD(V{:X})", a)
            }
            0x55 => {
                let end_index = ((opcode & 0x0F00) >> 8) as usize;
                format!("dump(V{:X})", end_index)
            }
            0x65 => {
                let end_index = ((opcode & 0x0F00) >> 8) as usize;
                format!("load(V{:X})", end_index)
            }
            _ => return Err(format!("Unhandled op-code: {:#06X}", opcode)),
        },
        _ => return Err(format!("Unhandled op-code: {:#06X}", opcode)),
    };
    Ok(s)
}

#[test]
fn test_disassemble_opcode() {
    assert_eq!(disassemble_opcode(0xF70A).unwrap(), "V7 = get_key()");
}

#[test]
fn test_disassemble_rom_aligned() {
    let rom = vec![
        0xF7, 0x0A, // instruction
        0x83, 0x67, // instruction
    ];

    let result = disassemble_rom(rom);

    let expected: HashMap<usize, String> = [
        (0x200, "V7 = get_key()".to_owned()),
        (0x202, "V3 = V6 - V3".to_owned()),
    ]
    .iter()
    .cloned()
    .collect();
    assert_eq!(result, expected);
}

#[test]
fn test_disassemble_rom_unaligned() {
    let rom = vec![
        0xF7, 0x0A, // instruction
        0x12, 0x05, // jump instruction
        0xFF, // junk
        0xF7, 0x0A, // instruction
        0xFF, // junk
    ];

    let result = disassemble_rom(rom);

    let expected: HashMap<usize, String> = [
        (0x200, "V7 = get_key()".to_owned()),
        (0x202, "jump: 0x205".to_owned()),
        (0x205, "V7 = get_key()".to_owned()),
    ]
    .iter()
    .cloned()
    .collect();
    assert_eq!(result, expected);
}
