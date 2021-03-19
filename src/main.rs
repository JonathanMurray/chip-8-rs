use std::fmt;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::Read;

const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;

fn main() {
    println!("Hello, world!");
    let m = Machine::new();
    println!("Machine: {:?}", m);
}

fn debug(message: &str) {
    println!("{}", message);
}

struct DisplayBuffer([bool; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize]);

impl DisplayBuffer {
    fn new() -> DisplayBuffer {
        DisplayBuffer([false; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize])
    }

    fn flip_pixel(&mut self, x: u8, y: u8) {
        let index = y as usize * SCREEN_HEIGHT as usize + x as usize;
        self.0[index] = !self.0[index];
    }

    fn get_pixel(&mut self, x: u8, y: u8) -> bool {
        let index = y as usize * SCREEN_HEIGHT as usize + x as usize;
        self.0[index]
    }
}

struct Machine {
    memory: [u8; 0x1000],
    registers: [u8; 16],
    address_register: u16,
    program_counter: u16,
    stack: [u16; 16],
    stack_pointer: u8,
    display_buffer: DisplayBuffer,
}

impl Machine {
    fn new() -> Machine {
        Machine {
            memory: [0; 0x1000],
            registers: [0; 16],
            address_register: 0,
            program_counter: 0,
            stack: [0; 16],
            stack_pointer: 0,
            display_buffer: DisplayBuffer::new(),
        }
    }

    fn execute_opcode(&mut self, opcode: u16) -> Result<(), String> {
        match opcode & 0xF000 {
            0x0000 => {
                let address = opcode & 0x0FFF;
                debug(&format!("[{:#06X}] call (machine): {:#05X}", opcode, address));
                self.stack[self.stack_pointer as usize] = self.program_counter;
                //TODO: extract to push method
                self.stack_pointer += 1;
                self.program_counter = address;
                Ok(())
            }
            0x1000 => {
                let address = opcode & 0x0FFF;
                debug(&format!("[{:#06X}] jump: {:#05X}", opcode, address));
                self.program_counter = address;
                Ok(())
            }
            0x2000 => {
                let address = opcode & 0x0FFF;
                debug(&format!("[{:#06X}] call: {:#05X}", opcode, address));
                self.stack[self.stack_pointer as usize] = self.program_counter;
                //TODO: extract to push method
                self.stack_pointer += 1;
                self.program_counter = address;
                Ok(())
            }
            0x3000 => {
                let index = ((opcode & 0x0F00) >> 8) as usize;
                let constant = (opcode & 0x00FF) as u8;
                debug(&format!("[{:#06X}] skip if V{:X} == {:#04X}", opcode, index, constant));
                if self.registers[index] == constant {
                    self.program_counter += 2;
                }
                Ok(())
            }
            0x4000 => {
                let index = ((opcode & 0x0F00) >> 8) as usize;
                let constant = (opcode & 0x00FF) as u8;
                debug(&format!("[{:#06X}] skip if V{:X} != {:#04X}", opcode, index, constant));
                if self.registers[index] != constant {
                    self.program_counter += 2;
                }
                Ok(())
            }
            0x5000 => {
                let first_index = ((opcode & 0x0F00) >> 8) as usize;
                let second_index = ((opcode & 0x00F0) >> 4) as usize;
                debug(&format!("[{:#06X}] skip if V{:X} == V{:X}", opcode, first_index, second_index));
                if self.registers[first_index] == self.registers[second_index] {
                    self.program_counter += 2;
                }
                Ok(())
            }
            0x6000 => {
                let index = ((opcode & 0x0F00) >> 8) as usize;
                let constant = (opcode & 0x00FF) as u8;
                debug(&format!("[{:#06X}] V{:X} = {:#04X}", opcode, index, constant));
                self.registers[index] = constant;
                Ok(())
            }
            0x7000 => {
                let index = ((opcode & 0x0F00) >> 8) as usize;
                let constant = (opcode & 0x00FF) as u8;
                debug(&format!("[{:#06X}] V{:X} += {:#04X}", opcode, index, constant));
                let result = self.registers[index].wrapping_add(constant);
                self.registers[index] = result;
                Ok(())
            }
            0x9000 => {
                let first_index = ((opcode & 0x0F00) >> 8) as usize;
                let second_index = ((opcode & 0x00F0) >> 4) as usize;
                debug(&format!("[{:#06X}] skip if V{:X} != V{:X}", opcode, first_index, second_index));
                if self.registers[first_index] != self.registers[second_index] {
                    self.program_counter += 2;
                }
                Ok(())
            }
            0xA000 => {
                let address = opcode & 0x0FFF;
                debug(&format!("[{:#06X}] I = {:#04X}", opcode, address));
                self.address_register = address;
                Ok(())
            }
            0xD000 => {
                let x = ((opcode & 0x0F00) >> 8) as u8;
                let y = ((opcode & 0x00F0) >> 4) as u8;
                let height = ((opcode & 0x000F) + 1) as u8;
                debug(&format!("[{:#06X}] render({}, {}, {})", opcode, x, y, height));
                let mut any_pixel_flip = false;
                for dy in 0..height {
                    let row_data = self.memory[(self.address_register + dy as u16) as usize];
                    for dx in 0..8 {
                        if row_data & (1 << (7 - dx)) != 0 {
                            self.display_buffer.flip_pixel(x + dx, y + dy);
                            if !self.display_buffer.get_pixel(x + dx, y + dy) {
                                any_pixel_flip = true;
                            }
                        }
                    }    
                }
                self.registers[0xF] = if any_pixel_flip { 1 } else { 0 };
                Ok(())
            }
            0xF000 => match opcode & 0x00FF {
                0x0055 => {
                    let end_index = ((opcode & 0x0F00) >> 8) as usize;
                    debug(&format!("[{:#06X}] dump(V{:X})", opcode, end_index));
                    for i in 0..end_index + 1 {
                        self.memory[self.address_register as usize + i] = self.registers[i];
                    }
                    Ok(())
                }
                _ => Err(format!("Unhandled op-code: {:#06X}", opcode)),
            },
            _ => Err(format!("Unhandled op-code: {:#06X}", opcode)),
        }
    }

    fn step(&mut self) -> Result<(), String> {
        let addr = self.program_counter as usize;
        debug(&format!("{:#05X}", addr));
        let opcode = ((self.memory[addr] as u16) << 8) | self.memory[addr + 1] as u16;
        self.program_counter += 2;
        self.execute_opcode(opcode)
    }
}

impl Debug for Machine {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Machine")
            .field("registers", &self.registers)
            .field("address_register", &self.address_register)
            .field("program_counter", &self.program_counter)
            .finish()
    }
}

#[test]
fn test_0nnn_call() {
    // TODO should this call be handled differently from normal calls?
    let mut m = Machine::new();
    m.program_counter = 0x987;

    // Call machine code routine at 0x234
    m.execute_opcode(0x0234).unwrap();

    assert_eq!(m.program_counter, 0x234);
    assert_eq!(
        m.stack,
        [0x987, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(m.stack_pointer, 1);
}

#[test]
fn test_1nnn_jump() {
    let mut m = Machine::new();

    // Jump to 0x567
    m.execute_opcode(0x1567).unwrap();

    assert_eq!(m.program_counter, 0x567);
}

#[test]
fn test_2nnn_call() {
    let mut m = Machine::new();

    // Call subroutine at 0xA05
    m.execute_opcode(0x2A05).unwrap();

    
    assert_eq!(m.program_counter, 0xA05);
    assert_eq!(
        m.stack,
        [0xA05, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(m.stack_pointer, 1);
}

#[test]
fn test_3xnn_skip_if_eq() {
    let mut m = Machine::new();
    m.program_counter = 5;
    m.registers[5] = 0xFF;

    // Skip if V5 == 0xFF
    m.execute_opcode(0x35FF).unwrap();

    assert_eq!(m.program_counter, 7);
}

#[test]
fn test_4xnn_skip_if_not_eq() {
    let mut m = Machine::new();
    m.program_counter = 5;
    m.registers[5] = 0xEA;

    // Skip if V5 != 0xFF
    m.execute_opcode(0x45FF).unwrap();

    assert_eq!(m.program_counter, 7);
}


#[test]
fn test_5xy0_skip_if_registers_eq() {
    let mut m = Machine::new();
    m.program_counter = 5;
    m.registers[0x2] = 0x99;
    m.registers[0xA] = 0x99;

    // Skip if V2 == VA
    m.execute_opcode(0x52A0).unwrap();

    assert_eq!(m.program_counter, 7);
}

#[test]
fn test_6xnn_set_register() {
    let mut m = Machine::new();

    // V3 = 0xA2
    m.execute_opcode(0x63A2).unwrap();

    assert_eq!(m.registers[3], 0xA2);
}

#[test]
fn test_7xnn_add_to_register() {
    let mut m = Machine::new();
    m.registers[0xB] = 0xF0;

    // VB += 0x05
    m.execute_opcode(0x7B05).unwrap();

    assert_eq!(m.registers[0xB], 0xF5);
}

#[test]
fn test_7xnn_add_to_register_overflow() {
    let mut m = Machine::new();
    m.registers[0xB] = 0xFF;

    // VB += 0x35
    m.execute_opcode(0x7B35).unwrap();

    assert_eq!(m.registers[0xB], 0x34);
}


#[test]
fn test_9xy0_skip_if_registers_not_eq() {
    let mut m = Machine::new();
    m.program_counter = 5;
    m.registers[0x2] = 0x75;
    m.registers[0xA] = 0x99;

    // Skip if V2 != VA
    m.execute_opcode(0x92A0).unwrap();

    assert_eq!(m.program_counter, 7);
}

#[test]
fn test_annn_set_address_register() {
    let mut m = Machine::new();

    // I = 0xF38
    m.execute_opcode(0xAF38).unwrap();

    assert_eq!(m.address_register, 0xF38);
}

#[test]
fn test_dxyn_draw_1_row_no_carry() {
    let mut m = Machine::new();
    m.address_register = 100;
    m.memory[m.address_register as usize] = 0b1010_0001;
    m.registers[0xF] = 7;

    // draw(8, 5, 0)
    m.execute_opcode(0xD850).unwrap();

    let expected = [true, false, true, false, false, false, false, true];
    for i in 0..8 {
        assert_eq!(m.display_buffer.get_pixel(8 + i, 5), expected[i as usize]);
    }
    assert_eq!(m.registers[0xF], 0)
}

#[test]
fn test_dxyn_draw_1_row_carry() {
    let mut m = Machine::new();
    m.address_register = 100;
    m.memory[m.address_register as usize] = 0b1010_0001;
    m.registers[0xF] = 7;
    m.display_buffer.flip_pixel(10, 5);

    // draw(8, 5, 0)
    m.execute_opcode(0xD850).unwrap();

    let expected = [true, false, false, false, false, false, false, true];
    for i in 0..8 {
        assert_eq!(m.display_buffer.get_pixel(8 + i, 5), expected[i as usize]);
    }
    assert_eq!(m.registers[0xF], 1)
}

#[test]
fn test_dxyn_draw_2_rows_no_carry() {
    let mut m = Machine::new();
    m.address_register = 100;
    m.memory[m.address_register as usize] = 0b1010_0001;
    m.memory[(m.address_register+1) as usize] = 0b0011_1100;

    // draw(8, 5, 1)
    m.execute_opcode(0xD851).unwrap();

    let expected_first_row = [true, false, true, false, false, false, false, true];
    let expected_second_row = [false, false, true, true, true, true, false, false];
    for i in 0..8 {
        assert_eq!(m.display_buffer.get_pixel(8 + i, 5), expected_first_row[i as usize]);
        assert_eq!(m.display_buffer.get_pixel(8 + i, 6), expected_second_row[i as usize]);
    }
    assert_eq!(m.registers[0xF], 0)
}

#[test]
fn test_fx55_dump_registers_to_memory() {
    let mut m = Machine::new();
    m.registers[0x0] = 0x00;
    m.registers[0x1] = 0x12;
    m.registers[0x2] = 0x34;
    m.registers[0x3] = 0x56;
    m.address_register = 0x0F05;

    // dump V0-2
    m.execute_opcode(0xF255).unwrap();

    assert_eq!(&m.memory[0x0F05..0x0F09], [0x00, 0x12, 0x34, 0x00]);
}

#[test]
fn test_rom() {
    let mut f = File::open("test_opcode.ch8").expect("Open test file");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Read from test file");
    let mut m = Machine::new();
    for (i, b) in buffer.into_iter().enumerate() {
        m.memory[0x200 + i] = b;
    }
    m.program_counter = 0x200;

    for _ in 0..100 {
        m.step().unwrap();
    }
}
