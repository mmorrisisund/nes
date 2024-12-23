pub mod instruction;

use instruction::{Instruction, OpCode};

use super::common::Bus;

// const NMI_VECTOR_ADDRESS: u16 = 0xfffa;
const RESET_VECTOR_ADDRESS: u16 = 0xfffc;
const IRQ_VECTOR_ADDRESS: u16 = 0xfffe;

const STACK_PAGE: u16 = 0x0100;
const STACK_START: u8 = 0xfd;

#[derive(Debug, PartialEq, Eq)]
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

fn on_same_page(addr1: u16, addr2: u16) -> bool {
    (addr1 & 0xff00) == (addr2 & 0xff00)
}

pub struct CPU<T: Bus> {
    reg_pc: u16,
    reg_sp: u8,
    reg_a: u8,
    reg_x: u8,
    reg_y: u8,
    reg_status: u8,

    next_instruction: Option<Instruction>,
    cycles_left: u8,
    total_cycles: u64,

    bus: T,
}

// Public
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
            cycles_left: 0,
            total_cycles: 0,

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
        self.total_cycles = 7;

        self.fetch_next_instruction();
    }

    pub fn cycle_tick(&mut self) -> RunState {
        self.total_cycles += 1;

        if self.cycles_left == 0 && self.next_instruction.is_none() {
            // if interrupt check

            // else
            // - check for irq change request

            // - fetch
            self.fetch_next_instruction();
            self.cycles_left -= 1;

            RunState::Waiting
        } else if self.cycles_left == 1 && self.next_instruction.is_some() {
            // TODO: can't this just be done in execute?
            let instruction = self.next_instruction.take().unwrap();
            let state = self.execute_instruction(&instruction);
            // adjust cycle time
            self.cycles_left -= 1;
            // return state from execution
            state
        } else {
            self.cycles_left -= 1;
            RunState::Waiting
        }
    }

    pub fn run_next_instruction(&mut self) -> RunState {
        loop {
            let state = self.cycle_tick();
            if state != RunState::Waiting {
                return state;
            }
        }
    }

    pub fn trace(&mut self) -> String {
        if self.next_instruction.is_none() {
            self.fetch_next_instruction();
        }
        let instruction = self.next_instruction.as_ref().unwrap().clone();

        let pc = self.reg_pc - instruction.get_instruction_len() as u16;
        let op_code = instruction.byte_value;

        let operands = match instruction.get_instruction_len() {
            2 => format!("{:02X}", instruction.raw_operand as u8),
            3 => {
                let lo = (instruction.raw_operand & 0xff) as u8;
                let hi = (instruction.raw_operand >> 8 & 0xff) as u8;
                format!("{:02X} {:02X}", lo, hi)
            }
            _ => String::new(),
        };
        let status = if instruction.is_official { " " } else { "*" };
        let mnemonic = format!("{}", instruction.op_code);

        let decoded_operand = match instruction.addressing_mode {
            instruction::AddressingMode::Implied => String::new(),
            instruction::AddressingMode::Relative => {
                format!("${:04X}", instruction.operand)
            }
            instruction::AddressingMode::Immediate => {
                format!("#${:02X}", instruction.operand)
            }
            instruction::AddressingMode::Accumulator => "A".to_string(),
            instruction::AddressingMode::ZeroPage => {
                let addr = instruction.operand;
                let value = self.bus.read(addr);
                format!("${addr:02X} = {value:02X}")
            }
            instruction::AddressingMode::ZeroPageX => {
                let base = instruction.raw_operand;
                let addr = instruction.operand;
                let value = self.bus.read(addr);

                format!("${base:02X},X @ {addr:02X} = {value:02X}")
            }
            instruction::AddressingMode::ZeroPageY => {
                let base = instruction.raw_operand;
                let addr = instruction.operand;
                let value = self.bus.read(addr);

                format!("${base:02X},Y @ {addr:02X} = {value:02X}")
            }
            instruction::AddressingMode::Absolute => {
                let addr = instruction.raw_operand;
                let value = self.bus.read(addr);
                match instruction.op_code {
                    OpCode::Jmp | OpCode::Jsr => format!("${addr:04X}"),
                    _ => format!("${addr:04X} = {value:02X}"),
                }
            }
            instruction::AddressingMode::AbsoluteX => {
                let base = instruction.raw_operand;
                let addr = instruction.operand;
                let value = self.bus.read(addr);

                format!("${base:04X},X @ {addr:04X} = {value:02X}")
            }
            instruction::AddressingMode::AbsoluteY => {
                let base = instruction.raw_operand;
                let addr = instruction.operand;
                let value = self.bus.read(addr);

                format!("${base:04X},Y @ {addr:04X} = {value:02X}")
            }
            instruction::AddressingMode::Indirect => {
                let base = instruction.raw_operand;
                let addr = instruction.operand;

                format!("(${base:04X}) = {addr:04X}")
            }
            instruction::AddressingMode::IndexedIndirect => {
                let base = instruction.raw_operand as u8;
                let ptr = base.wrapping_add(self.reg_x);
                let addr = instruction.operand;
                let value = self.bus.read(addr);

                format!("(${base:02X},X) @ {ptr:02X} = {addr:04X} = {value:02X}")
            }
            instruction::AddressingMode::IndirectIndexed => {
                let base = instruction.raw_operand;

                let lo = self.bus.read(base) as u16;
                let hi = self.bus.read((base + 1) & 0xff) as u16;
                let ptr = (hi << 8) | lo;

                let addr = instruction.operand;
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
        let c = self.total_cycles;
        let regs = format!("A:{a:02X} X:{x:02X} Y:{y:02X} P:{p:02X} SP:{s:02X} CYC:{c}");

        format!("{asm:47} {regs}")
    }

    pub fn bus(&self) -> &T {
        &self.bus
    }
}

// Private
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

    fn get_operand_value(&mut self, instruction: &Instruction) -> u8 {
        if instruction.operand_is_address() {
            self.bus.read(instruction.operand)
        } else {
            instruction.operand as u8
        }
    }

    fn run_bitwise_operation<F>(&mut self, instruction: &Instruction, operation: F)
    where
        F: Fn(u8, u8) -> u8,
    {
        let operand_value = self.get_operand_value(instruction);
        let result = operation(self.reg_a, operand_value);

        self.update_nz_flag(result);
        self.reg_a = result;
    }

    fn run_cmp_operation(&mut self, instruction: &Instruction, register_value: u8) {
        let operand_value = self.get_operand_value(instruction);
        let result = (register_value as u16).wrapping_sub(operand_value as u16);

        self.update_nz_flag(result as u8);
        self.set_status(StatusFlag::Carry, result & 0xff00 == 0);
    }

    // returns (cycle adjustment, run state)
    fn run_branch_condition(&mut self, new_addr: u16, condition: bool) -> (u8, RunState) {
        // TODO: Remove cycles and return value
        let cycles = 0;
        let pc_start = self.reg_pc.wrapping_sub(2);

        if condition {
            if !on_same_page(self.reg_pc, new_addr) {
                self.total_cycles += 1;
            }
            self.reg_pc = new_addr;
            self.total_cycles += 1;
        }

        (
            cycles,
            if condition && pc_start == new_addr {
                RunState::InfiniteLoop(pc_start)
            } else {
                RunState::NormalExecution
            },
        )
    }

    fn run_load_instruction(&mut self, instruction: &Instruction) -> u8 {
        let operand_value = self.get_operand_value(instruction);
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

    fn fetch_next_instruction(&mut self) {
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
        self.reg_pc += (len - 1) as u16;
        self.cycles_left = instruction.cycle_length;

        instruction.operand = operand;
        instruction.raw_operand = operand;
        instruction.operand = self.decode_operand(&mut instruction);

        self.next_instruction = Some(instruction);
    }

    // TODO: Should this return (Operand, bool) bool=page cross
    fn decode_operand(&mut self, instruction: &mut Instruction) -> u16 {
        use instruction::AddressingMode::*;

        let raw_value = instruction.operand;

        match instruction.addressing_mode {
            // TODO: should Accumulator be separate and actually load Acc?
            Implied | Immediate | Accumulator => {
                assert!(!instruction.operand_is_address());
                raw_value
            }
            Relative => {
                let sign_extended_value = if raw_value & 0x80 != 0 {
                    raw_value | 0xff00
                } else {
                    raw_value
                };
                self.reg_pc.wrapping_add(sign_extended_value)
            }
            ZeroPage => raw_value & 0xff,
            ZeroPageX => (raw_value + self.reg_x as u16) & 0xff,
            ZeroPageY => (raw_value + self.reg_y as u16) & 0xff,
            Absolute => raw_value,
            AbsoluteX => {
                let result = raw_value.wrapping_add(self.reg_x as u16);
                if !on_same_page(raw_value, result) && !instruction.has_fixed_cycle_length() {
                    self.cycles_left += 1;
                };

                result
            }
            AbsoluteY => {
                let result = raw_value.wrapping_add(self.reg_y as u16);
                if !on_same_page(raw_value, result) && !instruction.has_fixed_cycle_length() {
                    self.cycles_left += 1;
                };

                result
            }
            Indirect => {
                // TODO: See if this can be added to trait
                let lo = self.bus.read(raw_value) as u16;
                let hi = if raw_value & 0xff == 0xff {
                    self.bus.read(raw_value & 0xff00)
                } else {
                    self.bus.read(raw_value + 1)
                } as u16;
                (hi << 8) | lo
            }
            IndexedIndirect => {
                let ptr = raw_value.wrapping_add(self.reg_x as u16) & 0xff;
                let lo = self.bus.read(ptr) as u16;
                let hi = self.bus.read((ptr + 1) & 0xff) as u16;

                (hi << 8) | lo
            }
            IndirectIndexed => {
                let lo = self.bus.read(raw_value) as u16;
                let hi = self.bus.read((raw_value + 1) & 0xff) as u16;
                let ptr = (hi << 8) | lo;

                let result = ptr.wrapping_add(self.reg_y as u16);
                if !on_same_page(ptr, result) && !instruction.has_fixed_cycle_length() {
                    self.cycles_left += 1;
                };

                result
            }
        }
    }

    fn execute_instruction(&mut self, instruction: &Instruction) -> RunState {
        use instruction::OpCode::*;

        let mut state = RunState::NormalExecution;

        // match opcode and perform action (some actions alter remaining cycle time)
        match instruction.op_code {
            // Access
            Lda => self.reg_a = self.run_load_instruction(instruction),
            Sta => self.bus.write(instruction.operand, self.reg_a),
            Ldx => self.reg_x = self.run_load_instruction(instruction),
            Stx => self.bus.write(instruction.operand, self.reg_x),
            Ldy => self.reg_y = self.run_load_instruction(instruction),
            Sty => self.bus.write(instruction.operand, self.reg_y),

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
            Adc => self.adc(instruction),
            Sbc => self.sbc(instruction),
            Inc => self.inc(instruction.operand),
            Dec => self.dec(instruction.operand),
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
            Asl => self.asl(instruction),
            Lsr => self.lsr(instruction),
            Rol => self.rol(instruction),
            Ror => self.ror(instruction),

            // Bitwise
            And => self.and(instruction),
            Eor => self.eor(instruction),
            Ora => self.ora(instruction),
            Bit => {
                let data = self.bus.read(instruction.operand);

                self.set_status(StatusFlag::Zero, self.reg_a & data == 0);
                self.set_status(StatusFlag::Overflow, data & StatusFlag::Overflow as u8 != 0);
                self.set_status(StatusFlag::Negative, data & StatusFlag::Negative as u8 != 0);
            }

            // Compare
            Cmp => self.run_cmp_operation(instruction, self.reg_a),
            Cpx => self.run_cmp_operation(instruction, self.reg_x),
            Cpy => self.run_cmp_operation(instruction, self.reg_y),

            // Branch
            Bcc => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand,
                    self.reg_status & StatusFlag::Carry as u8 == 0, // TODO: update this to use central function is_set
                );
                // todo: update cycles remaining
                state = run_state
            }
            Bcs => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand,
                    self.reg_status & StatusFlag::Carry as u8 != 0,
                );
                // todo: update cycles remaining
                state = run_state
            }
            Beq => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand,
                    self.reg_status & StatusFlag::Zero as u8 != 0,
                );
                // todo: update cycles remaining
                state = run_state
            }
            Bne => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand,
                    self.reg_status & StatusFlag::Zero as u8 == 0,
                );
                // todo: update cycles remaining
                state = run_state
            }
            Bpl => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand,
                    self.reg_status & StatusFlag::Negative as u8 == 0,
                );
                // todo: update cycles remaining
                state = run_state
            }
            Bmi => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand,
                    self.reg_status & StatusFlag::Negative as u8 != 0,
                );
                // todo: update cycles remaining
                state = run_state
            }
            Bvc => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand,
                    self.reg_status & StatusFlag::Overflow as u8 == 0,
                );
                // todo: update cycles remaining
                state = run_state
            }
            Bvs => {
                let (_cycles, run_state) = self.run_branch_condition(
                    instruction.operand,
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
                self.reg_pc = instruction.operand;

                if instruction_start == self.reg_pc {
                    state = RunState::InfiniteLoop(instruction_start);
                }
            }
            Jsr => {
                let return_addr =
                    self.reg_pc.wrapping_add(2) - instruction.get_instruction_len() as u16;
                self.push_stack(((return_addr >> 8) & 0xff) as u8);
                self.push_stack((return_addr & 0xff) as u8);
                self.reg_pc = instruction.operand;
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
                self.and(instruction);
                self.lsr(instruction);
            }
            Anc => {
                self.and(instruction);
                self.set_status(StatusFlag::Carry, self.is_set(StatusFlag::Negative));
            }
            Arr => {
                self.and(instruction);

                // let mut ror_ins = instruction.clone();
                // ror_ins.operand.is_address = false;  // this might already be false based on addressing mode
                self.ror(instruction);

                self.set_status(StatusFlag::Carry, (self.reg_a >> 6) & 1 != 0);
                self.set_status(
                    StatusFlag::Overflow,
                    (self.reg_a >> 6 & 1) ^ (self.reg_a >> 5 & 1) != 0,
                );
            }
            Axs => {
                let result = self.reg_a & self.reg_x;
                self.bus.write(instruction.operand, result);
            }
            Lax => {
                let data = self.bus.read(instruction.operand);
                self.reg_a = data;
                self.reg_x = data;
                self.update_nz_flag(data);
            }
            Sax => {
                self.bus.write(instruction.operand, self.reg_a & self.reg_x);
            }

            Dcp => {
                self.dec(instruction.operand);
                self.run_cmp_operation(instruction, self.reg_a);
            }
            Isc => {
                self.inc(instruction.operand);
                self.sbc(instruction);
            }
            Rla => {
                self.rol(instruction);
                self.and(instruction);
            }
            Rra => {
                self.ror(instruction);
                self.adc(instruction);
            }
            Slo => {
                self.asl(instruction);
                self.ora(instruction);
            }
            Sre => {
                self.lsr(instruction);
                self.eor(instruction);
            }

            Atx => {
                self.and(instruction);
                self.reg_x = self.reg_a;
            }
            Las => {
                let result = self.bus.read(instruction.operand) & self.reg_sp;

                self.update_nz_flag(result);
                self.reg_a = result;
                self.reg_x = result;
                self.reg_sp = result;
            }
            Axa => {
                let hi = (instruction.operand >> 8) as u8;
                let value = self.reg_a & self.reg_x & (hi + 1);

                // TODO: on page cross, value becomes hi of the store (hi << 8) | lo
                self.bus.write(instruction.operand, value);
            }
            Shx => {
                let hi = (instruction.operand >> 8) as u8;
                let value = self.reg_x & hi;

                // TODO: The value to be stored is copied also
                //       to ADDR_HI if page boundary is crossed.

                self.bus.write(instruction.operand, value);
            }
            Shy => {
                let hi = (instruction.operand >> 8) as u8;
                let value = self.reg_y & hi;

                // TODO: The value to be stored is copied also
                //       to ADDR_HI if page boundary is crossed.

                self.bus.write(instruction.operand, value);
            }
            Tas => {
                self.reg_sp = self.reg_a & self.reg_x;

                let hi = (instruction.operand >> 8) as u8;
                let value = self.reg_sp & hi;

                // TODO: The value to be stored is copied also
                //       to ADDR_HI if page boundary is crossed.
                self.bus.write(instruction.operand, value);
            }
            Xaa => {
                self.reg_a = self.reg_x;
                self.and(instruction);
            }

            Kil => {
                println!("KIL: {:04X}", self.reg_pc);
                todo!();
            }
        }

        state
    }

    // arithmetic
    fn adc(&mut self, instruction: &Instruction) {
        let operand_value = self.get_operand_value(instruction);
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

    fn sbc(&mut self, instruction: &Instruction) {
        let operand_value = self.get_operand_value(instruction);
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
    fn asl(&mut self, instruction: &Instruction) {
        let value = if instruction.operand_is_address() {
            self.bus.read(instruction.operand)
        } else {
            self.reg_a
        };

        let new_value = value << 1;

        if instruction.operand_is_address() {
            self.bus.write(instruction.operand, new_value)
        } else {
            self.reg_a = new_value
        }
        self.update_nz_flag(new_value);
        self.set_status(StatusFlag::Carry, value & 0x80 != 0);
    }

    fn lsr(&mut self, instruction: &Instruction) {
        let value = if instruction.operand_is_address() {
            self.bus.read(instruction.operand)
        } else {
            self.reg_a
        };

        let new_value = value >> 1;

        if instruction.operand_is_address() {
            self.bus.write(instruction.operand, new_value);
        } else {
            self.reg_a = new_value;
        }
        self.set_status(StatusFlag::Carry, value & 0x01 != 0);
        self.update_nz_flag(new_value);
    }

    fn rol(&mut self, instruction: &Instruction) {
        let value = if instruction.operand_is_address() {
            self.bus.read(instruction.operand)
        } else {
            self.reg_a
        };

        let mut new_value = value << 1;
        new_value |= self.reg_status & StatusFlag::Carry as u8; // this works b/c carry flag is bit0

        if instruction.operand_is_address() {
            self.bus.write(instruction.operand, new_value);
        } else {
            self.reg_a = new_value;
        }
        self.set_status(StatusFlag::Carry, value & 0x80 != 0);
        self.update_nz_flag(new_value);
    }

    fn ror(&mut self, instruction: &Instruction) {
        let value = if instruction.operand_is_address() {
            self.bus.read(instruction.operand)
        } else {
            self.reg_a
        };

        let mut new_value = value >> 1;
        new_value |= if self.is_set(StatusFlag::Carry) {
            0x80
        } else {
            0x00
        };

        if instruction.operand_is_address() {
            self.bus.write(instruction.operand, new_value);
        } else {
            self.reg_a = new_value;
        }
        self.set_status(StatusFlag::Carry, value & 0x01 != 0);
        self.update_nz_flag(new_value);
    }

    // bitwise
    fn and(&mut self, instruction: &Instruction) {
        self.run_bitwise_operation(instruction, |a, b| a & b);
    }

    fn eor(&mut self, instruction: &Instruction) {
        self.run_bitwise_operation(instruction, |a, b| a ^ b);
    }

    fn ora(&mut self, instruction: &Instruction) {
        self.run_bitwise_operation(instruction, |a, b| a | b);
    }
}

#[cfg(test)]
mod tests {
    use crate::cartridge::TestCartridge;

    use super::*;

    struct TestBusRom {
        pub cartridge: TestCartridge,
        ram: [u8; 0x800],
    }

    impl Bus for TestBusRom {
        fn read(&self, address: u16) -> u8 {
            match address {
                0x0000..0x2000 => self.ram[(address & 0x7ff) as usize], // ram
                0x2000..0x4000 => 0,                                    // ppu
                0x4000..0x4018 => 0,                                    // apu
                0x4018..0x4020 => 0,                                    // unused
                0x4020.. => self.cartridge.read(address),               // cartridge
            }
        }

        fn write(&mut self, address: u16, data: u8) {
            match address {
                0x0000..0x2000 => self.ram[(address & 0x7ff) as usize] = data, // ram
                0x2000..0x4000 => {}                                           // ppu
                0x4000..0x4018 => {}                                           // apu
                0x4018..0x4020 => {}                                           // unused
                0x4020.. => self.cartridge.write(address, data),               // cartridge
            }
        }
    }

    struct TestBusBin {
        data: [u8; 0x10000],
    }

    impl Bus for TestBusBin {
        fn read(&self, address: u16) -> u8 {
            self.data[address as usize]
        }

        fn write(&mut self, address: u16, data: u8) {
            match address {
                0x0002 | 0x0003 => println!("write: {data:02X}"),
                _ => {}
            }
            self.data[address as usize] = data;
        }
    }

    fn cycle_until_infinite(cpu: &mut CPU<TestBusRom>) {
        loop {
            let state = cpu.cycle_tick();
            if let RunState::InfiniteLoop(_) = state {
                return;
            }
        }
    }

    fn cycle_until_mem_neq(cpu: &mut CPU<TestBusRom>, address: u16, value: u8) {
        loop {
            cpu.cycle_tick();

            if cpu.bus().read(address) != value {
                return;
            }
        }
    }

    fn execute_blargg_test(file_path: &str) {
        let cartridge = TestCartridge::from_file(file_path).unwrap();
        let bus = TestBusRom {
            cartridge,
            ram: [0; 0x800],
        };
        let mut cpu = CPU::new(bus);

        cpu.reset();

        cycle_until_infinite(&mut cpu);
        cycle_until_mem_neq(&mut cpu, 0x6000, 0x80);

        let result = cpu.bus.read(0x6000);
        assert_eq!(result, 0, "\n\ntrace: {}\n", cpu.trace());
    }

    #[test]
    fn functionality() {
        let file_data = include_bytes!("../test_roms/cpu/functional/6502_functional_test.bin");
        let mut data = [0; 0x10000];

        data[0xa..file_data.len() + 0xa].clone_from_slice(file_data);

        data[0xfffc] = 0x00;
        data[0xfffd] = 0x04;

        let bus = TestBusBin { data };
        let mut cpu = CPU::new(bus);

        const SUCCESS_ADDRESS: u16 = 0x336d;

        cpu.reset();
        loop {
            // println!("{}", cpu.trace());
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
    fn nestest() {
        let mut cartridge =
            TestCartridge::from_file("./src/test_roms/nes/nestest/nestest.nes").unwrap();

        // rom needs program code mirrored, default reset vector is wrong
        let mut rom = cartridge.get_prg_rom();
        let mut new_rom = rom.clone();
        new_rom.append(&mut rom);

        cartridge.set_prg_rom(new_rom);
        cartridge.write(0xfffc, 0x00);
        cartridge.write(0xfffd, 0xc0);

        let bus = TestBusRom {
            cartridge,
            ram: [0; 0x800],
        };
        let mut cpu = CPU::new(bus);

        // The end of this test results file was modified to return 00 instead of FF in the final checks.
        // I believe this can be reverted once memory mapping and/or ppu is in place.
        let test_results =
            std::fs::read_to_string("./src/test_roms/nes/nestest/nestest_no_ppu.log").unwrap();

        cpu.reset();
        for line in test_results.lines() {
            assert_eq!(cpu.trace(), line);
            let state = cpu.run_next_instruction();

            if let RunState::InfiniteLoop(pc) = state {
                panic!("entered infinite loop: {pc:04X}");
            }
        }
    }

    #[test]
    fn all_instrs() {
        execute_blargg_test("./src/test_roms/cpu/instr_function/all_instrs.nes");
    }

    #[test]
    fn instr_timing() {
        execute_blargg_test("./src/test_roms/cpu/instr_timing/instr_timing.nes");
    }
}
