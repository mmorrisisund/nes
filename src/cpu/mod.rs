pub mod instruction;

use instruction::{Instruction, OpCode, Operand};

use super::common::Bus;

const RESET_VECTOR_ADDRESS: u16 = 0xfffc;
const IRQ_VECTOR_ADDRESS: u16 = 0xfffe;

const STACK_PAGE: u16 = 0x0100;
const STACK_START: u8 = 0xfd;

pub enum RunState {
    Waiting,
    NormalExecution,
    InfiniteLoop(u16),
}

// NV_BDIZC
enum StatusFlag {
    Carry = 1 << 0,
    Zero = 1 << 1,
    InterruptDisable = 1 << 2,
    DecimalMode = 1 << 3,
    _BreakCmd = 1 << 4,
    Overflow = 1 << 6,
    Negative = 1 << 7,
}

pub struct CPU<T: Bus> {
    reg_pc: u16,
    reg_sp: u8,
    reg_a: u8,
    reg_x: u8,
    reg_y: u8,
    reg_status: u8,

    next_instruction: Option<Instruction>,

    bus: T,
}

fn on_same_page(addr1: u16, addr2: u16) -> bool {
    (addr1 & 0xff00) == (addr2 & 0xff00)
}

impl<T: Bus> CPU<T> {
    pub fn new(bus: T) -> Self {
        Self {
            reg_pc: 0,
            reg_sp: 0,
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            reg_status: 0,

            next_instruction: None,

            bus,
        }
    }

    pub fn reset(&mut self) {
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.reg_pc = self.bus.read_u16(RESET_VECTOR_ADDRESS);
        self.reg_sp = STACK_START;

        self.reg_status = StatusFlag::InterruptDisable as u8 | 1 << 5; // unused, but always 1

        self.next_instruction = Some(self.fetch_next_instruction());
    }

    pub fn run_next_instruction(&mut self) -> RunState {
        let instruction = self.next_instruction.take().unwrap();
        let state = self.run_instruction(&instruction);
        self.next_instruction = Some(self.fetch_next_instruction());
        state
    }

    pub fn trace(&self) -> String {
        let instruction = self.next_instruction.as_ref().unwrap().clone();

        let pc = self.reg_pc - instruction.get_instruction_len() as u16;
        let op_code = instruction.byte_value;

        let operands = match instruction.get_instruction_len() {
            2 => format!("{:02X}", instruction.operand.raw_value as u8),
            3 => {
                let lo = (instruction.operand.raw_value & 0xff) as u8;
                let hi = (instruction.operand.raw_value >> 8 & 0xff) as u8;
                format!("{:02X} {:02X}", lo, hi)
            }
            _ => String::new(),
        };
        let status = if instruction.is_official { " " } else { "*" };
        let mnemonic = format!("{}", instruction.op_code);

        let decoded_operand = match instruction.addressing_mode {
            instruction::AddressingMode::Implied => String::new(),
            instruction::AddressingMode::Relative => {
                format!("${:04X}", instruction.operand.address)
            }
            instruction::AddressingMode::Immediate => {
                format!("#${:02X}", instruction.operand.raw_value)
            }
            instruction::AddressingMode::Accumulator => "A".to_string(),
            instruction::AddressingMode::ZeroPage => {
                let addr = instruction.operand.address;
                let value = self.bus.read(addr);
                format!("${addr:02X} = {value:02X}")
            }
            instruction::AddressingMode::ZeroPageX => {
                let base = instruction.operand.raw_value;
                let addr = instruction.operand.address;
                let value = self.bus.read(addr);

                format!("${base:02X},X @ {addr:02X} = {value:02X}")
            }
            instruction::AddressingMode::ZeroPageY => {
                let base = instruction.operand.raw_value;
                let addr = instruction.operand.address;
                let value = self.bus.read(addr);

                format!("${base:02X},Y @ {addr:02X} = {value:02X}")
            }
            instruction::AddressingMode::Absolute => {
                let addr = instruction.operand.address;
                let value = self.bus.read(addr);
                match instruction.op_code {
                    OpCode::Jmp | OpCode::Jsr => format!("${addr:04X}"),
                    _ => format!("${addr:04X} = {value:02X}"),
                }
            }
            instruction::AddressingMode::AbsoluteX => {
                let base = instruction.operand.raw_value;
                let addr = instruction.operand.address;
                let value = self.bus.read(addr);

                format!("${base:04X},X @ {addr:04X} = {value:02X}")
            }
            instruction::AddressingMode::AbsoluteY => {
                let base = instruction.operand.raw_value;
                let addr = instruction.operand.address;
                let value = self.bus.read(addr);

                format!("${base:04X},Y @ {addr:04X} = {value:02X}")
            }
            instruction::AddressingMode::Indirect => {
                let base = instruction.operand.raw_value;
                let addr = instruction.operand.address;

                format!("(${base:04X}) = {addr:04X}")
            }
            instruction::AddressingMode::IndexedIndirect => {
                let base = instruction.operand.raw_value as u8;
                let ptr = base.wrapping_add(self.reg_x);
                let addr = instruction.operand.address;
                let value = self.bus.read(addr);

                format!("(${base:02X},X) @ {ptr:02X} = {addr:04X} = {value:02X}")
            }
            instruction::AddressingMode::IndirectIndexed => {
                let base = instruction.operand.raw_value;

                let lo = self.bus.read(base) as u16;
                let hi = self.bus.read((base + 1) & 0xff) as u16;
                let ptr = (hi << 8) | lo;

                let addr = instruction.operand.address;
                let value = self.bus.read(addr);

                format!("(${base:02X}),Y = {ptr:04X} @ {addr:04X} = {value:02X}")
            }
        };

        let asm =
            format!("{pc:04X}  {op_code:02X} {operands:6}{status}{mnemonic} {decoded_operand}");
        let a = self.reg_a;
        let x = self.reg_x;
        let y = self.reg_y;
        let p = self.reg_status;
        let s = self.reg_sp;
        let regs = format!("A:{a:02X} X:{x:02X} Y:{y:02X} P:{p:02X} SP:{s:02X}");

        format!("{asm:47} {regs}")
    }
}

impl<T: Bus> CPU<T> {
    fn set_flag(&mut self, flag: StatusFlag) {
        self.reg_status |= flag as u8;
    }

    fn unset_flag(&mut self, flag: StatusFlag) {
        self.reg_status &= !(flag as u8)
    }

    fn set_status(&mut self, flag: StatusFlag, is_set: bool) {
        if is_set {
            self.set_flag(flag);
        } else {
            self.unset_flag(flag);
        }
    }

    fn is_set(&self, flag: StatusFlag) -> bool {
        self.reg_status & flag as u8 != 0
    }

    fn update_nz_flag(&mut self, result: u8) {
        self.set_status(StatusFlag::Zero, result == 0);
        self.set_status(StatusFlag::Negative, result & 0x80 != 0);
    }

    fn get_operand_value(&self, operand: &Operand) -> u8 {
        if operand.is_address {
            self.bus.read(operand.address)
        } else {
            operand.raw_value as u8
        }
    }

    fn run_bitwise_operation<F>(&mut self, operand: &Operand, operation: F)
    where
        F: Fn(u8, u8) -> u8,
    {
        let operand_value = self.get_operand_value(operand);
        let result = operation(self.reg_a, operand_value);

        self.update_nz_flag(result);
        self.reg_a = result;
    }

    fn run_cmp_operation(&mut self, operand: &Operand, register_value: u8) {
        let operand_value = self.get_operand_value(operand);
        let result = (register_value as u16).wrapping_sub(operand_value as u16);

        self.update_nz_flag(result as u8);
        self.set_status(StatusFlag::Carry, result & 0xff00 == 0);
    }

    // returns (cycle adjustment, run state)
    fn run_branch_condition(&mut self, new_addr: u16, condition: bool) -> (u8, RunState) {
        let mut cycles = 0;
        let instruction_start = self.reg_pc.wrapping_sub(2);

        if condition {
            cycles = if on_same_page(self.reg_pc, new_addr) {
                1
            } else {
                2
            };
            self.reg_pc = new_addr;
        }

        (
            cycles,
            if condition && instruction_start == new_addr {
                RunState::InfiniteLoop(instruction_start)
            } else {
                RunState::NormalExecution
            },
        )
    }

    fn run_load_instruction(&mut self, operand: &Operand) -> u8 {
        let operand_value = self.get_operand_value(operand);
        self.update_nz_flag(operand_value);
        operand_value
    }

    fn push_stack(&mut self, data: u8) {
        self.bus.write(STACK_PAGE | self.reg_sp as u16, data);
        self.reg_sp = self.reg_sp.wrapping_sub(1);
    }

    fn pop_stack(&mut self) -> u8 {
        self.reg_sp = self.reg_sp.wrapping_add(1);
        self.bus.read(STACK_PAGE | self.reg_sp as u16)
    }

    fn fetch_next_instruction(&mut self) -> Instruction {
        let opcode = self.bus.read(self.reg_pc);

        self.reg_pc += 1;

        let mut instruction = Instruction::from_byte(opcode);
        let len = instruction.get_instruction_len();

        let mut operand = 0;

        match len {
            2 => operand |= self.bus.read(self.reg_pc) as u16,
            3 => {
                operand |= self.bus.read(self.reg_pc) as u16;
                operand |= (self.bus.read(self.reg_pc + 1) as u16) << 8;
            }
            _ => {}
        }
        instruction.operand.raw_value = operand;
        instruction.operand.is_address = instruction.operand_is_address();

        self.reg_pc += (len - 1) as u16;

        instruction.operand = self.decode_operand(&instruction);
        instruction
    }

    // TODO: Should this return (Operand, bool) bool=page cross
    fn decode_operand(&self, instruction: &Instruction) -> Operand {
        use instruction::AddressingMode::*;

        let mut result = instruction.operand.clone();
        let raw_value = instruction.operand.raw_value;

        match instruction.addressing_mode {
            // TODO: should Accumulator be separate and actually load Acc?
            Implied | Immediate | Accumulator => {
                assert!(!instruction.operand_is_address());
                result.address = raw_value; // TODO: should this be set if it's not an address?? Option??
                result.cycles = instruction.get_base_cycle_time();
            }
            Relative => {
                let sign_extended_value = if raw_value & 0x80 != 0 {
                    raw_value | 0xff00
                } else {
                    raw_value
                };
                result.address = self.reg_pc.wrapping_add(sign_extended_value);
                result.cycles = instruction.get_base_cycle_time();
            }
            ZeroPage => {
                result.address = raw_value & 0xff;
                result.cycles = instruction.get_base_cycle_time();
            }
            ZeroPageX => {
                result.address = (raw_value + self.reg_x as u16) & 0xff;
                result.cycles = instruction.get_base_cycle_time();
            }
            ZeroPageY => {
                result.address = (raw_value + self.reg_y as u16) & 0xff;
                result.cycles = instruction.get_base_cycle_time();
            }
            Absolute => {
                result.address = raw_value;
                result.cycles = instruction.get_base_cycle_time();
            }
            AbsoluteX => {
                result.address = raw_value.wrapping_add(self.reg_x as u16);
                result.cycles = if on_same_page(self.reg_pc, result.address) {
                    0
                } else {
                    1
                };
                result.cycles += instruction.get_base_cycle_time();
            }
            AbsoluteY => {
                result.address = raw_value.wrapping_add(self.reg_y as u16);
                result.cycles = if on_same_page(self.reg_pc, result.address) {
                    0
                } else {
                    1
                };
                result.cycles += instruction.get_base_cycle_time();
            }
            Indirect => {
                // TODO: See if this can be added to trait
                let lo = self.bus.read(raw_value) as u16;
                let hi = if raw_value & 0xff == 0xff {
                    self.bus.read(raw_value & 0xff00)
                } else {
                    self.bus.read(raw_value + 1)
                } as u16;
                result.address = (hi << 8) | lo;
                result.cycles = instruction.get_base_cycle_time();
            }
            IndexedIndirect => {
                let ptr = raw_value.wrapping_add(self.reg_x as u16) & 0xff;
                let lo = self.bus.read(ptr) as u16;
                let hi = self.bus.read((ptr + 1) & 0xff) as u16;

                result.address = (hi << 8) | lo;
                result.cycles = instruction.get_base_cycle_time();
            }
            IndirectIndexed => {
                let lo = self.bus.read(raw_value) as u16;
                let hi = self.bus.read((raw_value + 1) & 0xff) as u16;
                let ptr = (hi << 8) | lo;

                result.address = ptr.wrapping_add(self.reg_y as u16);
                result.cycles = instruction.get_instruction_len() as u8
                    + if on_same_page(result.address, ptr) {
                        0
                    } else {
                        1
                    };
            }
        }

        result
    }

    fn run_instruction(&mut self, instruction: &Instruction) -> RunState {
        use instruction::OpCode::*;

        let mut state = RunState::NormalExecution;

        // match opcode and perform action (some actions alter remaining cycle time)
        match instruction.op_code {
            // Access
            Lda => self.reg_a = self.run_load_instruction(&instruction.operand),
            Sta => self.bus.write(instruction.operand.address, self.reg_a),
            Ldx => self.reg_x = self.run_load_instruction(&instruction.operand),
            Stx => self.bus.write(instruction.operand.address, self.reg_x),
            Ldy => self.reg_y = self.run_load_instruction(&instruction.operand),
            Sty => self.bus.write(instruction.operand.address, self.reg_y),

            // Transfer
            Tax => {
                self.reg_x = self.reg_a;
                self.update_nz_flag(self.reg_x);
            }
            Txa => {
                self.reg_a = self.reg_x;
                self.update_nz_flag(self.reg_a);
            }
            Tay => {
                self.reg_y = self.reg_a;
                self.update_nz_flag(self.reg_y);
            }
            Tya => {
                self.reg_a = self.reg_y;
                self.update_nz_flag(self.reg_a);
            }

            // Arithmetic
            Adc => self.adc(&instruction.operand),
            Sbc => self.sbc(&instruction.operand),
            Inc => self.inc(instruction.operand.address),
            Dec => self.dec(instruction.operand.address),
            Inx => {
                self.reg_x = self.reg_x.wrapping_add(1);
                self.update_nz_flag(self.reg_x);
            }
            Dex => {
                self.reg_x = self.reg_x.wrapping_sub(1);
                self.update_nz_flag(self.reg_x);
            }
            Iny => {
                self.reg_y = self.reg_y.wrapping_add(1);
                self.update_nz_flag(self.reg_y);
            }
            Dey => {
                self.reg_y = self.reg_y.wrapping_sub(1);
                self.update_nz_flag(self.reg_y);
            }

            // Shift
            Asl => self.asl(&instruction.operand),
            Lsr => self.lsr(&instruction.operand),
            Rol => self.rol(&instruction.operand),
            Ror => self.ror(&instruction.operand),

            // Bitwise
            And => self.and(&instruction.operand),
            Eor => self.eor(&instruction.operand),
            Ora => self.ora(&instruction.operand),
            Bit => {
                let data = self.bus.read(instruction.operand.address);

                self.set_status(StatusFlag::Zero, self.reg_a & data == 0);
                self.set_status(StatusFlag::Overflow, data & StatusFlag::Overflow as u8 != 0);
                self.set_status(StatusFlag::Negative, data & StatusFlag::Negative as u8 != 0);
            }

            // Compare
            Cmp => self.run_cmp_operation(&instruction.operand, self.reg_a),
            Cpx => self.run_cmp_operation(&instruction.operand, self.reg_x),
            Cpy => self.run_cmp_operation(&instruction.operand, self.reg_y),

            // Branch
            Bcc => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand.address,
                    self.reg_status & StatusFlag::Carry as u8 == 0, // TODO: update this to use central function is_set
                );
                // todo: update cycles remaining
                state = run_state
            }
            Bcs => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand.address,
                    self.reg_status & StatusFlag::Carry as u8 != 0,
                );
                // todo: update cycles remaining
                state = run_state
            }
            Beq => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand.address,
                    self.reg_status & StatusFlag::Zero as u8 != 0,
                );
                // todo: update cycles remaining
                state = run_state
            }
            Bne => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand.address,
                    self.reg_status & StatusFlag::Zero as u8 == 0,
                );
                // todo: update cycles remaining
                state = run_state
            }
            Bpl => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand.address,
                    self.reg_status & StatusFlag::Negative as u8 == 0,
                );
                // todo: update cycles remaining
                state = run_state
            }
            Bmi => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand.address,
                    self.reg_status & StatusFlag::Negative as u8 != 0,
                );
                // todo: update cycles remaining
                state = run_state
            }
            Bvc => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand.address,
                    self.reg_status & StatusFlag::Overflow as u8 == 0,
                );
                // todo: update cycles remaining
                state = run_state
            }
            Bvs => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand.address,
                    self.reg_status & StatusFlag::Overflow as u8 != 0,
                );
                // todo: update cycles remaining
                state = run_state
            }

            // Jump
            Jmp => {
                // TODO: handle 6502 jump bug
                // TODO: adjust cycle time
                let instruction_start = self.reg_pc.wrapping_sub(3);
                self.reg_pc = instruction.operand.address;

                if instruction_start == self.reg_pc {
                    state = RunState::InfiniteLoop(instruction_start);
                }
            }
            Jsr => {
                let return_addr =
                    self.reg_pc.wrapping_add(2) - instruction.get_instruction_len() as u16;
                self.push_stack(((return_addr >> 8) & 0xff) as u8);
                self.push_stack((return_addr & 0xff) as u8);
                self.reg_pc = instruction.operand.address;
            }
            Rts => {
                let lo = self.pop_stack() as u16;
                let hi = self.pop_stack() as u16;
                self.reg_pc = ((hi << 8) | lo) + 1;
            }
            Rti => {
                let old_status = self.reg_status & 0x30;
                self.reg_status = self.pop_stack() | old_status;

                let lo = self.pop_stack() as u16;
                let hi = self.pop_stack() as u16;
                self.reg_pc = (hi << 8) | lo;
            }
            Brk => {
                // TODO: this entire thing really needs done
                // TODO: handle bug with nmi
                // TODO: probably work on interrupt stuff
                let return_addr = self.reg_pc + 2 - instruction.get_instruction_len() as u16;
                let lo = (return_addr & 0xff) as u8;
                let hi = (return_addr >> 8) as u8;
                self.push_stack(hi);
                self.push_stack(lo);

                self.push_stack(self.reg_status | 0x30);
                self.set_status(StatusFlag::InterruptDisable, true);

                self.reg_pc = self.bus.read_u16(IRQ_VECTOR_ADDRESS);
            }

            // Stack
            Pha => self.push_stack(self.reg_a),
            Pla => {
                self.reg_a = self.pop_stack();
                self.update_nz_flag(self.reg_a);
            }
            Php => self.push_stack(self.reg_status | 0x30),
            Plp => {
                // ignore bit4 and bit5
                let old_status = self.reg_status & 0x30;
                self.reg_status = (self.pop_stack() & 0xcf) | old_status;
            }
            Tsx => {
                self.reg_x = self.reg_sp;
                self.update_nz_flag(self.reg_x);
            }
            Txs => self.reg_sp = self.reg_x,

            // Flags
            Clc => self.unset_flag(StatusFlag::Carry),
            Sec => self.set_flag(StatusFlag::Carry),
            Cli => self.unset_flag(StatusFlag::InterruptDisable),
            Sei => self.set_flag(StatusFlag::InterruptDisable),
            Cld => self.unset_flag(StatusFlag::DecimalMode),
            Sed => self.set_flag(StatusFlag::DecimalMode),
            Clv => self.unset_flag(StatusFlag::Overflow),

            // Other
            Nop => {}

            // Unofficial
            // TODO: will probably need to track cycles before calls and reset after
            Alr => {
                self.and(&instruction.operand);
                self.lsr(&instruction.operand);
            }
            Anc => {
                self.and(&instruction.operand);
                self.set_status(StatusFlag::Carry, self.is_set(StatusFlag::Negative));
            }
            Arr => {
                self.and(&instruction.operand);

                let mut ror_ins = instruction.clone();
                ror_ins.operand.is_address = false;
                self.ror(&ror_ins.operand);

                self.set_status(StatusFlag::Carry, (self.reg_a >> 6) & 1 != 0);
                self.set_status(
                    StatusFlag::Overflow,
                    (self.reg_a >> 6 & 1) ^ (self.reg_a >> 5 & 1) != 0,
                );
            }
            Axs => {
                let result = self.reg_a & self.reg_x;
                self.bus.write(instruction.operand.address, result);
            }
            Lax => {
                let data = self.bus.read(instruction.operand.address);
                self.reg_a = data;
                self.reg_x = data;
                self.update_nz_flag(data);
            }
            Sax => {
                self.bus
                    .write(instruction.operand.address, self.reg_a & self.reg_x);
            }

            Dcp => {
                self.dec(instruction.operand.address);
                self.run_cmp_operation(&instruction.operand, self.reg_a);
            }
            Isc => {
                self.inc(instruction.operand.address);
                self.sbc(&instruction.operand);
            }
            Rla => {
                self.rol(&instruction.operand);
                self.and(&instruction.operand);
            }
            Rra => {
                self.ror(&instruction.operand);
                self.adc(&instruction.operand);
            }
            Slo => {
                self.asl(&instruction.operand);
                self.ora(&instruction.operand);
            }
            Sre => {
                self.lsr(&instruction.operand);
                self.eor(&instruction.operand);
            }

            Atx => {
                self.and(&instruction.operand);
                self.reg_x = self.reg_a;
            }
            Las => {
                let result = self.bus.read(instruction.operand.address) & self.reg_sp;

                self.update_nz_flag(result);
                self.reg_a = result;
                self.reg_x = result;
                self.reg_sp = result;
            }
            Axa => {
                let hi = (instruction.operand.address >> 8) as u8;
                let value = self.reg_a & self.reg_x & (hi + 1);

                // TODO: on page cross, value becomes hi of the store (hi << 8) | lo
                self.bus.write(instruction.operand.address, value);
            }
            Shx => {
                let hi = (instruction.operand.address >> 8) as u8;
                let value = self.reg_x & hi;

                // TODO: The value to be stored is copied also
                //       to ADDR_HI if page boundary is crossed.

                self.bus.write(instruction.operand.address, value);
            }
            Shy => {
                let hi = (instruction.operand.address >> 8) as u8;
                let value = self.reg_y & hi;

                // TODO: The value to be stored is copied also
                //       to ADDR_HI if page boundary is crossed.

                self.bus.write(instruction.operand.address, value);
            }
            Tas => {
                self.reg_sp = self.reg_a & self.reg_x;

                let hi = (instruction.operand.address >> 8) as u8;
                let value = self.reg_sp & hi;

                // TODO: The value to be stored is copied also
                //       to ADDR_HI if page boundary is crossed.
                self.bus.write(instruction.operand.address, value);
            }
            Xaa => {
                self.reg_a = self.reg_x;
                self.and(&instruction.operand);
            }

            Kil => {
                println!("KIL: {:04X}", self.reg_pc);
                todo!();
            }
        }

        state
    }

    // arithmetic
    fn adc(&mut self, operand: &Operand) {
        let operand_value = self.get_operand_value(operand);
        let carry_value = (self.reg_status & 0x01) as u16; // works b/c carry is bit0

        let result = (self.reg_a as u16)
            .wrapping_add(operand_value as u16)
            .wrapping_add(carry_value);
        let carry = result & 0xff00 != 0;

        let result = result as u8;
        let overflow = (result ^ self.reg_a) & (result ^ operand_value) & 0x80 != 0;

        self.set_status(StatusFlag::Carry, carry);
        self.set_status(StatusFlag::Overflow, overflow);
        self.update_nz_flag(result);
        self.reg_a = result;
    }

    fn sbc(&mut self, operand: &Operand) {
        let operand_value = self.get_operand_value(operand);
        let carry_value = (self.reg_status & 0x01) as u16; // works b/c carry is bit0

        let result = (self.reg_a as u16)
            .wrapping_add(!(operand_value as u16))
            .wrapping_add(carry_value);
        let carry = result & 0xff00 == 0;

        let result = result as u8;
        let overflow = (result ^ self.reg_a) & (result ^ !operand_value) & 0x80 != 0;

        self.set_status(StatusFlag::Carry, carry);
        self.set_status(StatusFlag::Overflow, overflow);
        self.update_nz_flag(result);
        self.reg_a = result;
    }

    fn inc(&mut self, address: u16) {
        let operand_value = self.bus.read(address);
        let new_value = operand_value.wrapping_add(1);

        self.bus.write(address, new_value);
        self.update_nz_flag(new_value);
    }

    fn dec(&mut self, address: u16) {
        let operand_value = self.bus.read(address);
        let new_value = operand_value.wrapping_sub(1);

        self.update_nz_flag(new_value);
        self.bus.write(address, new_value);
    }

    // shifts
    fn asl(&mut self, operand: &Operand) {
        let value = if operand.is_address {
            self.bus.read(operand.address)
        } else {
            self.reg_a
        };

        let new_value = value << 1;

        if operand.is_address {
            self.bus.write(operand.address, new_value)
        } else {
            self.reg_a = new_value
        }
        self.update_nz_flag(new_value);
        self.set_status(StatusFlag::Carry, value & 0x80 != 0);
    }

    fn lsr(&mut self, operand: &Operand) {
        let value = if operand.is_address {
            self.bus.read(operand.address)
        } else {
            self.reg_a
        };

        let new_value = value >> 1;

        if operand.is_address {
            self.bus.write(operand.address, new_value);
        } else {
            self.reg_a = new_value;
        }
        self.set_status(StatusFlag::Carry, value & 0x01 != 0);
        self.update_nz_flag(new_value);
    }

    fn rol(&mut self, operand: &Operand) {
        let value = if operand.is_address {
            self.bus.read(operand.address)
        } else {
            self.reg_a
        };

        let mut new_value = value << 1;
        new_value |= self.reg_status & StatusFlag::Carry as u8; // this works b/c carry flag is bit0

        if operand.is_address {
            self.bus.write(operand.address, new_value);
        } else {
            self.reg_a = new_value;
        }
        self.set_status(StatusFlag::Carry, value & 0x80 != 0);
        self.update_nz_flag(new_value);
    }

    fn ror(&mut self, operand: &Operand) {
        let value = if operand.is_address {
            self.bus.read(operand.address)
        } else {
            self.reg_a
        };

        let mut new_value = value >> 1;
        new_value |= if self.is_set(StatusFlag::Carry) {
            0x80
        } else {
            0x00
        };

        if operand.is_address {
            self.bus.write(operand.address, new_value);
        } else {
            self.reg_a = new_value;
        }
        self.set_status(StatusFlag::Carry, value & 0x01 != 0);
        self.update_nz_flag(new_value);
    }

    // bitwise
    fn and(&mut self, operand: &Operand) {
        self.run_bitwise_operation(operand, |a, b| a & b);
    }

    fn eor(&mut self, operand: &Operand) {
        self.run_bitwise_operation(operand, |a, b| a ^ b);
    }

    fn ora(&mut self, operand: &Operand) {
        self.run_bitwise_operation(operand, |a, b| a | b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBus {
        data: [u8; 0x10000],
    }

    impl Bus for TestBus {
        fn read(&self, address: u16) -> u8 {
            self.data[address as usize]
        }

        fn write(&mut self, address: u16, data: u8) {
            self.data[address as usize] = data;
        }
    }

    #[test]
    fn test_functionality() {
        let file_data = include_bytes!("..\\test roms\\6502_functional_test.bin");
        let mut data = [0; 0x10000];

        data[0xa..file_data.len() + 0xa].clone_from_slice(file_data);

        data[0xfffc] = 0x00;
        data[0xfffd] = 0x04;

        let bus = TestBus { data };
        let mut cpu = CPU::new(bus);

        const SUCCESS_ADDRESS: u16 = 0x336d;

        cpu.reset();
        loop {
            let state = cpu.run_next_instruction();

            // if we stuck in a loop, return error
            if let RunState::InfiniteLoop(pc) = state {
                assert!(
                    pc == SUCCESS_ADDRESS,
                    "Test failed at {:04X}, check the `.lst` file for more info",
                    pc
                );
                break;
            }
        }
    }

    #[test]
    fn test_nestest() {
        let file_data = include_bytes!("..\\test roms\\nestest.nes");
        // The end of this test results file was modified to return 00 instead of FF in the final checks.
        // I believe this can be reverted once memory mapping is in place.
        let test_results =
            std::fs::read_to_string(".\\src\\test roms\\nestest_no_cycle.log").unwrap();

        let mut data = [0; 0x10000];
        let rom_data = &file_data[0x10..0x4010];
        data[0x8000..0xc000].clone_from_slice(rom_data);
        data[0xc000..].clone_from_slice(rom_data);

        data[0xfffc] = 0x00;
        data[0xfffd] = 0xc0;

        let bus = TestBus { data };
        let mut cpu = CPU::new(bus);

        cpu.reset();

        assert_eq!(test_results.lines().next().unwrap(), cpu.trace());

        for line in test_results.lines().skip(1) {
            let state = cpu.run_next_instruction();

            assert_eq!(line, cpu.trace());

            if let RunState::InfiniteLoop(pc) = state {
                panic!("entered infinite loop: {pc:04X}");
            }
        }
    }

    #[ignore]
    #[test]
    fn test_timing() {
        let file_data = include_bytes!("..\\test roms\\instr_timing.nes");
        let mut data = [0; 0x10000];

        data[0..file_data.len()].clone_from_slice(file_data);

        // data[0xfffc] = 0x00;
        // data[0xfffd] = 0x04;

        let bus = TestBus { data };
        let mut cpu = CPU::new(bus);

        const SUCCESS_ADDRESS: u16 = 0x336d;

        cpu.reset();
        loop {
            let state = cpu.run_next_instruction();

            // if we stuck in a loop, return error
            if let RunState::InfiniteLoop(pc) = state {
                assert!(
                    pc == SUCCESS_ADDRESS,
                    "Test failed at {:04X}, check the `.lst` file for more info",
                    pc
                );
                break;
            }
        }
    }
}
