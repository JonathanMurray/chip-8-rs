use std::fmt;
use std::fmt::{Debug, Formatter};

const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;

fn debug(message: &str) {
    println!("{}", message);
}

pub struct DisplayBuffer(pub [bool; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize]);

impl DisplayBuffer {
    fn new() -> DisplayBuffer {
        DisplayBuffer([false; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize])
    }

    fn flip_pixel(&mut self, x: u8, y: u8) {
        let index = y as usize * SCREEN_WIDTH as usize + x as usize;
        self.0[index] = !self.0[index];
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> bool {
        let index = y as usize * SCREEN_WIDTH as usize + x as usize;
        self.0[index]
    }
}

impl Debug for DisplayBuffer {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for y in 0..SCREEN_HEIGHT {
            f.write_str("\n")?;
            for x in 0..SCREEN_WIDTH {
                if self.get_pixel(x, y) {
                    f.write_str("O")?;
                } else {
                    f.write_str(" ")?;
                }
            }
        }
        Ok(())
    }
}

pub struct Machine {
    memory: [u8; 0x1000],
    registers: [u8; 16],
    address_register: u16,
    program_counter: u16,
    stack: [u16; 16],
    stack_pointer: u8,
    pub display_buffer: DisplayBuffer,
}

impl Machine {
    pub fn new(memory: [u8; 0x1000]) -> Machine {
        Machine {
            memory: memory,
            registers: [0; 16],
            address_register: 0,
            program_counter: 0x200,
            stack: [0; 16],
            stack_pointer: 0,
            display_buffer: DisplayBuffer::new(),
        }
    }

    fn execute_opcode(&mut self, opcode: u16) -> Result<(), String> {
        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x00ee => {
                        debug(&format!("[{:#06X}] return", opcode));
                        // TODO extract to pop
                        self.stack_pointer -= 1;
                        self.program_counter = self.stack[self.stack_pointer as usize];
                        Ok(())
                    }
                    0x00e0 => Err(format!("TODO: Implement 00e0")),
                    _ => {
                        let address = opcode & 0x0FFF;
                        debug(&format!(
                            "[{:#06X}] call (machine): {:#05X}",
                            opcode, address
                        ));
                        //TODO: extract to push method
                        self.stack[self.stack_pointer as usize] = self.program_counter;
                        self.stack_pointer += 1;
                        self.program_counter = address;
                        Ok(())
                    }
                }
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
                //TODO: extract to push method
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.stack_pointer += 1;
                self.program_counter = address;
                Ok(())
            }
            0x3000 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let constant = (opcode & 0x00FF) as u8;
                debug(&format!(
                    "[{:#06X}] skip if V{:X} == {:#04X}",
                    opcode, a, constant
                ));
                if self.registers[a] == constant {
                    self.program_counter += 2;
                }
                Ok(())
            }
            0x4000 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let constant = (opcode & 0x00FF) as u8;
                debug(&format!(
                    "[{:#06X}] skip if V{:X} != {:#04X}",
                    opcode, a, constant
                ));
                if self.registers[a] != constant {
                    self.program_counter += 2;
                }
                Ok(())
            }
            0x5000 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let b = ((opcode & 0x00F0) >> 4) as usize;
                debug(&format!("[{:#06X}] skip if V{:X} == V{:X}", opcode, a, b));
                if self.registers[a] == self.registers[b] {
                    self.program_counter += 2;
                }
                Ok(())
            }
            0x6000 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let constant = (opcode & 0x00FF) as u8;
                debug(&format!("[{:#06X}] V{:X} = {:#04X}", opcode, a, constant));
                self.registers[a] = constant;
                Ok(())
            }
            0x7000 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let constant = (opcode & 0x00FF) as u8;
                debug(&format!("[{:#06X}] V{:X} += {:#04X}", opcode, a, constant));
                let result = self.registers[a].wrapping_add(constant);
                self.registers[a] = result;
                Ok(())
            }
            0x8000 => match opcode & 0x000F {
                0x0 => {
                    let a = ((opcode & 0x0F00) >> 8) as usize;
                    let b = ((opcode & 0x00F0) >> 4) as usize;
                    debug(&format!("[{:#06X}] V{:X} = V{:X}", opcode, a, b));
                    self.registers[a] = self.registers[b];
                    Ok(())
                }
                0x1 => {
                    let a = ((opcode & 0x0F00) >> 8) as usize;
                    let b = ((opcode & 0x00F0) >> 4) as usize;
                    debug(&format!("[{:#06X}] V{:X} = V{:X} | V{:X}", opcode, a, a, b));
                    self.registers[a] = self.registers[a] | self.registers[b];
                    Ok(())
                }
                0x2 => {
                    let a = ((opcode & 0x0F00) >> 8) as usize;
                    let b = ((opcode & 0x00F0) >> 4) as usize;
                    debug(&format!("[{:#06X}] V{:X} = V{:X} & V{:X}", opcode, a, a, b));
                    self.registers[a] = self.registers[a] & self.registers[b];
                    Ok(())
                }
                0x3 => {
                    let a = ((opcode & 0x0F00) >> 8) as usize;
                    let b = ((opcode & 0x00F0) >> 4) as usize;
                    debug(&format!("[{:#06X}] V{:X} = V{:X} ^ V{:X}", opcode, a, a, b));
                    self.registers[a] = self.registers[a] ^ self.registers[b];
                    Ok(())
                }
                0x4 => {
                    let a = ((opcode & 0x0F00) >> 8) as usize;
                    let b = ((opcode & 0x00F0) >> 4) as usize;
                    debug(&format!("[{:#06X}] V{:X} = V{:X} + V{:X}", opcode, a, a, b));
                    let result = self.registers[a] as u16 + self.registers[b] as u16;
                    self.registers[a] = (result & 0xFF) as u8;
                    self.registers[0xF] = if result > 0xFF { 1 } else { 0 };
                    Ok(())
                }
                0x5 => {
                    let a = ((opcode & 0x0F00) >> 8) as usize;
                    let b = ((opcode & 0x00F0) >> 4) as usize;
                    debug(&format!("[{:#06X}] V{:X} = V{:X} - V{:X}", opcode, a, a, b));
                    let result = self.registers[a] as i16 - self.registers[b] as i16;
                    self.registers[a] = (result % 0x100i16) as u8;
                    self.registers[0xF] = if result < 0 { 0 } else { 1 };
                    Ok(())
                }
                0x6 => {
                    let a = ((opcode & 0x0F00) >> 8) as usize;
                    debug(&format!("[{:#06X}] V{:X} >>= 1", opcode, a));
                    self.registers[0xF] = if self.registers[a] & 1 == 1 { 1 } else { 0 };
                    self.registers[a] >>= 1;
                    Ok(())
                }
                0x7 => {
                    let a = ((opcode & 0x0F00) >> 8) as usize;
                    let b = ((opcode & 0x00F0) >> 4) as usize;
                    debug(&format!("[{:#06X}] V{:X} = V{:X} - V{:X}", opcode, a, b, a));
                    let result = self.registers[b] as i16 - self.registers[a] as i16;
                    self.registers[a] = (result % 0x100i16) as u8;
                    self.registers[0xF] = if result < 0 { 0 } else { 1 };
                    Ok(())
                }
                0xE => {
                    let a = ((opcode & 0x0F00) >> 8) as usize;
                    debug(&format!("[{:#06X}] V{:X} <<= 1", opcode, a));
                    self.registers[0xF] = if self.registers[a] & 0b1000_0000 == 0b1000_0000 {
                        1
                    } else {
                        0
                    };
                    self.registers[a] <<= 1;
                    Ok(())
                }
                _ => Err(format!("Unhandled op-code: {:#06X}", opcode)),
            },
            0x9000 => {
                let a = ((opcode & 0x0F00) >> 8) as usize;
                let b = ((opcode & 0x00F0) >> 4) as usize;
                debug(&format!("[{:#06X}] skip if V{:X} != V{:X}", opcode, a, b));
                if self.registers[a] != self.registers[b] {
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
                let vx = ((opcode & 0x0F00) >> 8) as usize;
                let vy = ((opcode & 0x00F0) >> 4) as usize;
                let height = (opcode & 0x000F) as u8;
                debug(&format!(
                    "[{:#06X}] render(V{}, V{}, {})",
                    opcode, vx, vy, height
                ));

                let x = self.registers[vx];
                let y = self.registers[vy];

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
                0x55 => {
                    let end_index = ((opcode & 0x0F00) >> 8) as usize;
                    debug(&format!("[{:#06X}] dump(V{:X})", opcode, end_index));
                    for i in 0..end_index + 1 {
                        self.memory[self.address_register as usize + i] = self.registers[i];
                    }
                    Ok(())
                }
                0x65 => {
                    let end_index = ((opcode & 0x0F00) >> 8) as usize;
                    debug(&format!("[{:#06X}] load(V{:X})", opcode, end_index));
                    for i in 0..end_index + 1 {
                        self.registers[i] = self.memory[self.address_register as usize + i];
                    }
                    Ok(())
                }
                _ => Err(format!("Unhandled op-code: {:#06X}", opcode)),
            },
            _ => Err(format!("Unhandled op-code: {:#06X}", opcode)),
        }
    }

    pub fn step(&mut self) -> Result<(), String> {
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
    let mut m = Machine::new([0; 0x1000]);
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
fn test_00ee_return() {
    let mut m = Machine::new([0; 0x1000]);
    m.program_counter = 0x987;
    m.stack[0] = 0x123;
    m.stack_pointer = 1;

    // Return from subroutine
    m.execute_opcode(0x00ee).unwrap();

    assert_eq!(m.program_counter, 0x123);
    assert_eq!(m.stack_pointer, 0);
}

#[test]
fn test_1nnn_jump() {
    let mut m = Machine::new([0; 0x1000]);

    // Jump to 0x567
    m.execute_opcode(0x1567).unwrap();

    assert_eq!(m.program_counter, 0x567);
}

#[test]
fn test_2nnn_call() {
    let mut m = Machine::new([0; 0x1000]);
    m.program_counter = 0x153;

    // Call subroutine at 0xA05
    m.execute_opcode(0x2A05).unwrap();

    assert_eq!(m.program_counter, 0xA05);
    assert_eq!(
        m.stack,
        [0x153, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
    assert_eq!(m.stack_pointer, 1);
}

#[test]
fn test_3xnn_skip_if_eq() {
    let mut m = Machine::new([0; 0x1000]);
    m.program_counter = 5;
    m.registers[5] = 0xFF;

    // Skip if V5 == 0xFF
    m.execute_opcode(0x35FF).unwrap();

    assert_eq!(m.program_counter, 7);
}

#[test]
fn test_4xnn_skip_if_not_eq() {
    let mut m = Machine::new([0; 0x1000]);
    m.program_counter = 5;
    m.registers[5] = 0xEA;

    // Skip if V5 != 0xFF
    m.execute_opcode(0x45FF).unwrap();

    assert_eq!(m.program_counter, 7);
}

#[test]
fn test_5xy0_skip_if_registers_eq() {
    let mut m = Machine::new([0; 0x1000]);
    m.program_counter = 5;
    m.registers[0x2] = 0x99;
    m.registers[0xA] = 0x99;

    // Skip if V2 == VA
    m.execute_opcode(0x52A0).unwrap();

    assert_eq!(m.program_counter, 7);
}

#[test]
fn test_6xnn_set_register() {
    let mut m = Machine::new([0; 0x1000]);

    // V3 = 0xA2
    m.execute_opcode(0x63A2).unwrap();

    assert_eq!(m.registers[3], 0xA2);
}

#[test]
fn test_7xnn_add_to_register() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0xB] = 0xF0;

    // VB += 0x05
    m.execute_opcode(0x7B05).unwrap();

    assert_eq!(m.registers[0xB], 0xF5);
}

#[test]
fn test_7xnn_add_to_register_overflow() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0xB] = 0xFF;

    // VB += 0x35
    m.execute_opcode(0x7B35).unwrap();

    assert_eq!(m.registers[0xB], 0x34);
}

#[test]
fn test_8xy0_set_vx_to_vy() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0x2] = 0x75;
    m.registers[0xA] = 0x99;

    // V2 = VA
    m.execute_opcode(0x82A0).unwrap();

    assert_eq!(m.registers[0x2], 0x99);
    assert_eq!(m.registers[0xA], 0x99);
}

#[test]
fn test_8xy1_set_vx_to_vx_or_vy() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0x2] = 0b0100_1111;
    m.registers[0xA] = 0b0110_0100;

    // V2 = V2 | VA
    m.execute_opcode(0x82A1).unwrap();

    assert_eq!(m.registers[0x2], 0b0110_1111);
    assert_eq!(m.registers[0xA], 0b0110_0100);
}

#[test]
fn test_8xy2_set_vx_to_vx_and_vy() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0x2] = 0b0100_1111;
    m.registers[0xA] = 0b0110_0100;

    // V2 = V2 & VA
    m.execute_opcode(0x82A2).unwrap();

    assert_eq!(m.registers[0x2], 0b0100_0100);
    assert_eq!(m.registers[0xA], 0b0110_0100);
}

#[test]
fn test_8xy3_set_vx_to_vx_xor_vy() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0x2] = 0b0100_1111;
    m.registers[0xA] = 0b0110_0100;

    // V2 = V2 ^ VA
    m.execute_opcode(0x82A3).unwrap();

    assert_eq!(m.registers[0x2], 0b0010_1011);
    assert_eq!(m.registers[0xA], 0b0110_0100);
}

#[test]
fn test_8xy4_add_vy_to_vx() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0xF] = 3;
    m.registers[0x6] = 53;
    m.registers[0x0] = 22;

    // V6 = V6 + V0
    m.execute_opcode(0x8604).unwrap();

    assert_eq!(m.registers[0x6], 75);
    assert_eq!(m.registers[0xF], 0);
}

#[test]
fn test_8xy4_add_vy_to_vx_carry() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0xF] = 3;
    m.registers[0x6] = 0xFF;
    m.registers[0x0] = 22;

    // V6 = V6 + V0
    m.execute_opcode(0x8604).unwrap();

    assert_eq!(m.registers[0x6], 21);
    assert_eq!(m.registers[0xF], 1);
}

#[test]
fn test_8xy5_subtract_vy_from_vx() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0xF] = 3;
    m.registers[0x6] = 110;
    m.registers[0x0] = 60;

    // V6 = V6 - V0
    m.execute_opcode(0x8605).unwrap();

    assert_eq!(m.registers[0x6], 50);
    assert_eq!(m.registers[0xF], 1);
}

#[test]
fn test_8xy5_subtract_vy_from_vx_borrow() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0xF] = 3;
    m.registers[0x6] = 60;
    m.registers[0x0] = 110;

    // V6 = V6 - V0
    m.execute_opcode(0x8605).unwrap();

    assert_eq!(m.registers[0x6], 206);
    assert_eq!(m.registers[0xF], 0);
}

#[test]
fn test_8xy6_shift_right() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0xF] = 3;
    m.registers[0x2] = 0b01011110;

    // V2 >>= 1
    m.execute_opcode(0x8206).unwrap();

    assert_eq!(m.registers[0x2], 0b00101111);
    assert_eq!(m.registers[0xF], 0);
}

#[test]
fn test_8xy6_shift_right_carry() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0xF] = 3;
    m.registers[0x2] = 0b01011101;

    // V2 >>= 1
    m.execute_opcode(0x8206).unwrap();

    assert_eq!(m.registers[0x2], 0b00101110);
    assert_eq!(m.registers[0xF], 1);
}

#[test]
fn test_8xy7_set_vx_to_vy_minus_vx() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0xF] = 3;
    m.registers[0x6] = 60;
    m.registers[0x0] = 110;

    // V6 = V0 - V6
    m.execute_opcode(0x8607).unwrap();

    assert_eq!(m.registers[0x6], 50);
    assert_eq!(m.registers[0xF], 1);
}

#[test]
fn test_8xy7_set_vx_to_vy_minus_vx_borrow() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0xF] = 3;
    m.registers[0x6] = 110;
    m.registers[0x0] = 60;

    // V6 = V0 - V6
    m.execute_opcode(0x8607).unwrap();

    assert_eq!(m.registers[0x6], 206);
    assert_eq!(m.registers[0xF], 0);
}

#[test]
fn test_8xye_shift_left() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0xF] = 3;
    m.registers[0x2] = 0b01011101;

    // V2 <<= 1
    m.execute_opcode(0x820E).unwrap();

    assert_eq!(m.registers[0x2], 0b10111010);
    assert_eq!(m.registers[0xF], 0);
}

#[test]
fn test_8xye_shift_left_carry() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0xF] = 3;
    m.registers[0x2] = 0b10011101;

    // V2 <<= 1
    m.execute_opcode(0x820E).unwrap();

    assert_eq!(m.registers[0x2], 0b00111010);
    assert_eq!(m.registers[0xF], 1);
}

#[test]
fn test_9xy0_skip_if_registers_not_eq() {
    let mut m = Machine::new([0; 0x1000]);
    m.program_counter = 5;
    m.registers[0x2] = 0x75;
    m.registers[0xA] = 0x99;

    // Skip if V2 != VA
    m.execute_opcode(0x92A0).unwrap();

    assert_eq!(m.program_counter, 7);
}

#[test]
fn test_annn_set_address_register() {
    let mut m = Machine::new([0; 0x1000]);

    // I = 0xF38
    m.execute_opcode(0xAF38).unwrap();

    assert_eq!(m.address_register, 0xF38);
}

#[test]
fn test_dxyn_draw_1_row_no_carry() {
    let mut m = Machine::new([0; 0x1000]);
    m.address_register = 100;
    m.memory[m.address_register as usize] = 0b1010_0001;
    m.registers[0xF] = 7;
    m.registers[0x5] = 5;
    m.registers[0x8] = 8;

    // draw(V8, V5, 1)
    m.execute_opcode(0xD851).unwrap();

    let expected = [true, false, true, false, false, false, false, true];
    for i in 0..8 {
        assert_eq!(m.display_buffer.get_pixel(8 + i, 5), expected[i as usize]);
    }
    assert_eq!(m.registers[0xF], 0)
}

#[test]
fn test_dxyn_draw_1_row_carry() {
    let mut m = Machine::new([0; 0x1000]);
    m.address_register = 100;
    m.memory[m.address_register as usize] = 0b1010_0001;
    m.registers[0xF] = 7;
    m.registers[0x5] = 5;
    m.registers[0x8] = 8;
    m.display_buffer.flip_pixel(10, 5);

    // draw(8, 5, 1)
    m.execute_opcode(0xD851).unwrap();

    let expected = [true, false, false, false, false, false, false, true];
    for i in 0..8 {
        assert_eq!(m.display_buffer.get_pixel(8 + i, 5), expected[i as usize]);
    }
    assert_eq!(m.registers[0xF], 1)
}

#[test]
fn test_dxyn_draw_2_rows_no_carry() {
    let mut m = Machine::new([0; 0x1000]);
    m.address_register = 100;
    m.memory[m.address_register as usize] = 0b1010_0001;
    m.memory[(m.address_register + 1) as usize] = 0b0011_1100;
    m.registers[0x5] = 5;
    m.registers[0x8] = 8;

    // draw(8, 5, 2)
    m.execute_opcode(0xD852).unwrap();

    let expected_first_row = [true, false, true, false, false, false, false, true];
    let expected_second_row = [false, false, true, true, true, true, false, false];
    for i in 0..8 {
        assert_eq!(
            m.display_buffer.get_pixel(8 + i, 5),
            expected_first_row[i as usize]
        );
        assert_eq!(
            m.display_buffer.get_pixel(8 + i, 6),
            expected_second_row[i as usize]
        );
    }
    assert_eq!(m.registers[0xF], 0)
}

#[test]
fn test_fx55_dump_registers_to_memory() {
    let mut m = Machine::new([0; 0x1000]);
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
fn test_fx65_load_memory_into_registers() {
    let mut m = Machine::new([0; 0x1000]);
    m.registers[0x0] = 0x77;
    m.registers[0x1] = 0x77;
    m.registers[0x2] = 0x77;
    m.registers[0x3] = 0x77;
    m.address_register = 0x0F05;
    m.memory[0x0F05] = 0x0A;
    m.memory[0x0F06] = 0x0B;
    m.memory[0x0F07] = 0x0C;
    m.memory[0x0F08] = 0x0D;

    // load V0-2
    m.execute_opcode(0xF265).unwrap();

    assert_eq!(&m.registers[0x0..0x4], [0x0A, 0x0B, 0x0C, 0x77]);
}

#[test]
fn test_rom() {
    use std::fs::File;
    use std::io::Read;
    let mut f = File::open("test_opcode.ch8").expect("Open test file");
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).expect("Read from test file");
    let mut m = Machine::new([0; 0x1000]);
    for (i, b) in buffer.into_iter().enumerate() {
        m.memory[0x200 + i] = b;
    }
    m.program_counter = 0x200;

    // TODO Run longer
    for _ in 0..200 {
        m.step().unwrap();
    }
}
