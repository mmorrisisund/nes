use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub(super) struct Instruction {
    pub byte_value: u8,
    pub operand: u16,
    pub op_code: OpCode,
    pub addressing_mode: AddressingMode,
    pub cycle_length: u8,
    // TODO: these are only used for tracing.  something better??
    pub(super) is_official: bool,
    pub(super) raw_operand: u16,
}

impl Instruction {
    pub fn from_byte(byte: u8) -> Self {
        Self {
            byte_value: byte,
            operand: 0,
            op_code: OPCODES[byte as usize],
            addressing_mode: ADDRESSING_MODES[byte as usize],
            cycle_length: CYCLE_LENGTH[byte as usize],
            is_official: OFFICIAL_STATUS[byte as usize],
            raw_operand: 0,
        }
    }

    pub fn get_instruction_len(&self) -> usize {
        self.addressing_mode.instruction_length()
    }

    pub fn operand_is_address(&self) -> bool {
        self.addressing_mode.operand_is_address()
    }

    pub fn has_fixed_cycle_length(&self) -> bool {
        self.op_code.has_fixed_cycle_length()
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let addr = match self.addressing_mode {
            Implied => String::new(),
            Relative => format!("${:02X}", self.operand),
            Immediate => format!("#${:02X}", self.operand),
            Accumulator => "A".to_string(),
            ZeroPage => format!("${:02X}", self.operand),
            ZeroPageX => format!("${:02X}, X", self.operand),
            ZeroPageY => format!("${:02X}, Y", self.operand),
            Absolute => format!("${:04X}", self.operand),
            AbsoluteX => format!("${:04X}, X", self.operand),
            AbsoluteY => format!("${:04X}, Y", self.operand),
            Indirect => format!("(${:04X})", self.operand),
            IndexedIndirect => format!("(${:02X}, X)", self.operand),
            IndirectIndexed => format!("(${:02X}), Y", self.operand),
        };
        write!(f, "{} {}", self.op_code, addr)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
    Adc, // Add with carry
    And, // And
    Asl, // Arithmetic shift left
    Bcs, // Branch if carry set
    Bcc, // Branch if carry clear
    Beq, // Branch if equal (zero set)
    Bne, // Branch if not equal (zero clear)
    Bmi, // Branch if minus (negative set)
    Bpl, // Branch if plus (negative clear)
    Bit, // Bit test
    Brk, // Break
    Bvs, // Branch if overflow set
    Bvc, // Branch if overflow clear
    Clc, // Clear carry
    Cld, // Clear decimal
    Cli, // Clear interrupt disable
    Clv, // Clear overflow
    Cmp, // Compare a
    Cpx, // Compare x
    Cpy, // Compare y
    Dec, // Decrement memory
    Dex, // Decrement x
    Dey, // Decrement y
    Eor, // Exclusive Or
    Inc, // Increment memory
    Inx, // Increment x
    Iny, // Increment y
    Jmp, // Jump
    Jsr, // Jump to subroutine
    Lda, // Load a
    Ldx, // Load x
    Ldy, // Load y
    Lsr, // Logical shift right
    Nop, // No operation
    Ora, // Or
    Pha, // Push a
    Php, // Push status
    Pla, // Pull a
    Plp, // Pull status
    Rol, // Rotate left
    Ror, // Rotate right
    Rti, // Return from interrupt
    Rts, // Return from subroutine
    Sbc, // Subtract with carry
    Sec, // Set carry
    Sed, // Set decimal
    Sei, // Set interrupt disable
    Sta, // Store a
    Stx, // Store x
    Sty, // Store y
    Tax, // Transfer a to x
    Tay, // Transfer a to y
    Tsx, // Transfer sp to x
    Txa, // Transfer x to a
    Txs, // Transfer x to sp
    Tya, // Transfer y to a

    // Unofficial [Mnemonic used from https://www.masswerk.at/6502/6502_instruction_set.html#illegals]
    Alr, // And #imm, then Lsr A
    Anc, // And #imm, then set Carry = Negative
    Ane, // Transfer X to A, then And #imm [Txa + And #imm]
    Arr, // And #imm, then Ror A
    Dcp, // Decrement memory, then Compare A [Dec + Cmp]
    Isc, // Increment memory, then Subtract with carry [Inc + Sbc]
    Las, // Store the value of ({value in $addr} & (the stack pointer)) into A, X, and (the stack pointer)
    Lax, // Load A and X [Lda + Tax]
    Lxa, // And #imm -> X
    Rla, // Rotate left, then And [Rol + And]
    Rra, // Rotate right, then Add with carry [Ror + Adc]
    Sax, // Store the value of (A and X)
    Sbx, // ((A and X) - #imm) -> X
    Sha, // Store the value of (A and X and {High byte of $addr}) into $addr
    Shx, // Store the value of (X and {High byte of $addr}) into $addr
    Shy, // Store the value of (Y and {High byte of $addr}) into $addr
    Slo, // Arithmetic shift left, then OR [Asl + Ora]
    Sre, // Logical shift right, then XOR [Lsr + Eor]
    Tas, // Store the value of (X and A) into (the stack pointer), and store ((the stack pointer) and {High byte of $addr}) into $addr

    Kil, // Halt the CPU (CRASH)
}

impl OpCode {
    // These opcodes have addressing modes that can cross a page and add a cycle, however, these codes always have a fixed length
    fn has_fixed_cycle_length(&self) -> bool {
        self == &OpCode::Sta
            || self == &OpCode::Asl // read-modify-write instruction
            || self == &OpCode::Dec // read-modify-write instruction
            || self == &OpCode::Inc // read-modify-write instruction
            || self == &OpCode::Lsr // read-modify-write instruction
            || self == &OpCode::Rol // read-modify-write instruction
            || self == &OpCode::Ror // read-modify-write instruction

            // unofficial
            || self == &OpCode::Sha
            || self == &OpCode::Dcp
            || self == &OpCode::Isc
            || self == &OpCode::Rla
            || self == &OpCode::Rra
            || self == &OpCode::Slo
            || self == &OpCode::Sre
            || self == &OpCode::Shx
            || self == &OpCode::Shy
            || self == &OpCode::Tas
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = match self {
            Adc => "ADC",
            And => "AND",
            Asl => "ASL",
            Bcs => "BCS",
            Bcc => "BCC",
            Beq => "BEQ",
            Bne => "BNE",
            Bmi => "BMI",
            Bpl => "BPL",
            Bit => "BIT",
            Brk => "BRK",
            Bvs => "BVS",
            Bvc => "BVC",
            Clc => "CLC",
            Cld => "CLD",
            Cli => "CLI",
            Clv => "CLV",
            Cmp => "CMP",
            Cpx => "CPX",
            Cpy => "CPY",
            Dec => "DEC",
            Dex => "DEX",
            Dey => "DEY",
            Eor => "EOR",
            Inc => "INC",
            Inx => "INX",
            Iny => "INY",
            Jmp => "JMP",
            Jsr => "JSR",
            Lda => "LDA",
            Ldx => "LDX",
            Ldy => "LDY",
            Lsr => "LSR",
            Nop => "NOP",
            Ora => "ORA",
            Pha => "PHA",
            Php => "PHP",
            Pla => "PLA",
            Plp => "PLP",
            Rol => "ROL",
            Ror => "ROR",
            Rti => "RTI",
            Rts => "RTS",
            Sbc => "SBC",
            Sec => "SEC",
            Sed => "SED",
            Sei => "SEI",
            Sta => "STA",
            Stx => "STX",
            Sty => "STY",
            Tax => "TAX",
            Tay => "TAY",
            Tsx => "TSX",
            Txa => "TXA",
            Txs => "TXS",
            Tya => "TYA",
            Slo => "SLO",
            Sre => "SRE",
            Rla => "RLA",
            Rra => "RRA",
            Isc => "ISC",
            Dcp => "DCP",
            Sax => "SAX",
            Lax => "LAX",
            Anc => "ANC",
            Alr => "ALR",
            Arr => "ARR",
            Lxa => "LXA",
            Sbx => "SBX",
            Ane => "ANE",
            Sha => "SHA",
            Shy => "SHY",
            Shx => "SHX",
            Tas => "TAS",
            Las => "LAS",
            Kil => "KIL",
        };
        write!(f, "{code}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressingMode {
    Implied,
    Relative,
    Immediate,
    Accumulator,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
}

impl AddressingMode {
    fn instruction_length(&self) -> usize {
        match self {
            Implied => 1,
            Relative => 2,
            Immediate => 2,
            Accumulator => 1,
            ZeroPage => 2,
            ZeroPageX => 2,
            ZeroPageY => 2,
            Absolute => 3,
            AbsoluteX => 3,
            AbsoluteY => 3,
            Indirect => 3,
            IndexedIndirect => 2,
            IndirectIndexed => 2,
        }
    }

    fn operand_is_address(&self) -> bool {
        !(self == &AddressingMode::Implied
            || self == &AddressingMode::Immediate
            || self == &AddressingMode::Accumulator)
    }
}

use OpCode::*;

#[rustfmt::skip]
const OPCODES: [OpCode; 256] = [
//  0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F
    Brk, Ora, Kil, Slo, Nop, Ora, Asl, Slo, Php, Ora, Asl, Anc, Nop, Ora, Asl, Slo, // 00
    Bpl, Ora, Kil, Slo, Nop, Ora, Asl, Slo, Clc, Ora, Nop, Slo, Nop, Ora, Asl, Slo, // 10
    Jsr, And, Kil, Rla, Bit, And, Rol, Rla, Plp, And, Rol, Anc, Bit, And, Rol, Rla, // 20
    Bmi, And, Kil, Rla, Nop, And, Rol, Rla, Sec, And, Nop, Rla, Nop, And, Rol, Rla, // 30
    Rti, Eor, Kil, Sre, Nop, Eor, Lsr, Sre, Pha, Eor, Lsr, Alr, Jmp, Eor, Lsr, Sre, // 40
    Bvc, Eor, Kil, Sre, Nop, Eor, Lsr, Sre, Cli, Eor, Nop, Sre, Nop, Eor, Lsr, Sre, // 50
    Rts, Adc, Kil, Rra, Nop, Adc, Ror, Rra, Pla, Adc, Ror, Arr, Jmp, Adc, Ror, Rra, // 60
    Bvs, Adc, Kil, Rra, Nop, Adc, Ror, Rra, Sei, Adc, Nop, Rra, Nop, Adc, Ror, Rra, // 70
    Nop, Sta, Nop, Sax, Sty, Sta, Stx, Sax, Dey, Nop, Txa, Ane, Sty, Sta, Stx, Sax, // 80
    Bcc, Sta, Kil, Sha, Sty, Sta, Stx, Sax, Tya, Sta, Txs, Tas, Shy, Sta, Shx, Sha, // 90
    Ldy, Lda, Ldx, Lax, Ldy, Lda, Ldx, Lax, Tay, Lda, Tax, Lxa, Ldy, Lda, Ldx, Lax, // A0
    Bcs, Lda, Kil, Lax, Ldy, Lda, Ldx, Lax, Clv, Lda, Tsx, Las, Ldy, Lda, Ldx, Lax, // B0
    Cpy, Cmp, Nop, Dcp, Cpy, Cmp, Dec, Dcp, Iny, Cmp, Dex, Sbx, Cpy, Cmp, Dec, Dcp, // C0
    Bne, Cmp, Kil, Dcp, Nop, Cmp, Dec, Dcp, Cld, Cmp, Nop, Dcp, Nop, Cmp, Dec, Dcp, // D0
    Cpx, Sbc, Nop, Isc, Cpx, Sbc, Inc, Isc, Inx, Sbc, Nop, Sbc, Cpx, Sbc, Inc, Isc, // E0
    Beq, Sbc, Kil, Isc, Nop, Sbc, Inc, Isc, Sed, Sbc, Nop, Isc, Nop, Sbc, Inc, Isc, // F0
];

use AddressingMode::*;

#[rustfmt::skip]
const ADDRESSING_MODES: [AddressingMode; 256] = [
//  0,8        1,9              2,A          3,B              4,C        5,D        6,E        7,F    
    Implied,   IndexedIndirect, Implied,     IndexedIndirect, ZeroPage,  ZeroPage,  ZeroPage,  ZeroPage,  // 00
    Implied,   Immediate,       Accumulator, Immediate,       Absolute,  Absolute,  Absolute,  Absolute,  // 08
    Relative,  IndirectIndexed, Implied,     IndirectIndexed, ZeroPageX, ZeroPageX, ZeroPageX, ZeroPageX, // 10
    Implied,   AbsoluteY,       Implied,     AbsoluteY,       AbsoluteX, AbsoluteX, AbsoluteX, AbsoluteX, // 18
    Absolute,  IndexedIndirect, Implied,     IndexedIndirect, ZeroPage,  ZeroPage,  ZeroPage,  ZeroPage,  // 20
    Implied,   Immediate,       Accumulator, Immediate,       Absolute,  Absolute,  Absolute,  Absolute,  // 28
    Relative,  IndirectIndexed, Implied,     IndirectIndexed, ZeroPageX, ZeroPageX, ZeroPageX, ZeroPageX, // 30
    Implied,   AbsoluteY,       Implied,     AbsoluteY,       AbsoluteX, AbsoluteX, AbsoluteX, AbsoluteX, // 38
    Implied,   IndexedIndirect, Implied,     IndexedIndirect, ZeroPage,  ZeroPage,  ZeroPage,  ZeroPage,  // 40
    Implied,   Immediate,       Accumulator, Immediate,       Absolute,  Absolute,  Absolute,  Absolute,  // 48
    Relative,  IndirectIndexed, Implied,     IndirectIndexed, ZeroPageX, ZeroPageX, ZeroPageX, ZeroPageX, // 50
    Implied,   AbsoluteY,       Implied,     AbsoluteY,       AbsoluteX, AbsoluteX, AbsoluteX, AbsoluteX, // 58
    Implied,   IndexedIndirect, Implied,     IndexedIndirect, ZeroPage,  ZeroPage,  ZeroPage,  ZeroPage,  // 60
    Implied,   Immediate,       Accumulator, Immediate,       Indirect,  Absolute,  Absolute,  Absolute,  // 68
    Relative,  IndirectIndexed, Implied,     IndirectIndexed, ZeroPageX, ZeroPageX, ZeroPageX, ZeroPageX, // 70
    Implied,   AbsoluteY,       Implied,     AbsoluteY,       AbsoluteX, AbsoluteX, AbsoluteX, AbsoluteX, // 78
    Immediate, IndexedIndirect, Immediate,   IndexedIndirect, ZeroPage,  ZeroPage,  ZeroPage,  ZeroPage,  // 80
    Implied,   Immediate,       Implied,     Immediate,       Absolute,  Absolute,  Absolute,  Absolute,  // 88
    Relative,  IndirectIndexed, Implied,     IndirectIndexed, ZeroPageX, ZeroPageX, ZeroPageY, ZeroPageY, // 90
    Implied,   AbsoluteY,       Implied,     AbsoluteY,       AbsoluteX, AbsoluteX, AbsoluteY, AbsoluteY, // 98
    Immediate, IndexedIndirect, Immediate,   IndexedIndirect, ZeroPage,  ZeroPage,  ZeroPage,  ZeroPage,  // A0
    Implied,   Immediate,       Implied,     Immediate,       Absolute,  Absolute,  Absolute,  Absolute,  // A8
    Relative,  IndirectIndexed, Implied,     IndirectIndexed, ZeroPageX, ZeroPageX, ZeroPageY, ZeroPageY, // B0
    Implied,   AbsoluteY,       Implied,     AbsoluteY,       AbsoluteX, AbsoluteX, AbsoluteY, AbsoluteY, // B8
    Immediate, IndexedIndirect, Immediate,   IndexedIndirect, ZeroPage,  ZeroPage,  ZeroPage,  ZeroPage,  // C0
    Implied,   Immediate,       Implied,     Immediate,       Absolute,  Absolute,  Absolute,  Absolute,  // C8
    Relative,  IndirectIndexed, Implied,     IndirectIndexed, ZeroPageX, ZeroPageX, ZeroPageX, ZeroPageX, // D0
    Implied,   AbsoluteY,       Implied,     AbsoluteY,       AbsoluteX, AbsoluteX, AbsoluteX, AbsoluteX, // D8
    Immediate, IndexedIndirect, Immediate,   IndexedIndirect, ZeroPage,  ZeroPage,  ZeroPage,  ZeroPage,  // E0
    Implied,   Immediate,       Implied,     Immediate,       Absolute,  Absolute,  Absolute,  Absolute,  // E8
    Relative,  IndirectIndexed, Implied,     IndirectIndexed, ZeroPageX, ZeroPageX, ZeroPageX, ZeroPageX, // F0
    Implied,   AbsoluteY,       Implied,     AbsoluteY,       AbsoluteX, AbsoluteX, AbsoluteX, AbsoluteX, // F8
];

#[rustfmt::skip]
const CYCLE_LENGTH: [u8; 256] = [
//  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    7, 6, 0, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6, // 00
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, // 10
    6, 6, 0, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6, // 20
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, // 30
    6, 6, 0, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6, // 40
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, // 50
    6, 6, 0, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6, // 60
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, // 70
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, // 80
    2, 6, 0, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5, // 90
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, // A0
    2, 5, 0, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4, // B0
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, // C0
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, // D0
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, // E0
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, // F0
];

#[rustfmt::skip]
const OFFICIAL_STATUS: [bool; 256] = [
//  0      1     2      3      4      5     6     7      8     9      A      B      C      D     E      F
    true,  true, false, false, false, true, true, false, true, true,  true,  false, false, true, true,  false, // 00
    true,  true, false, false, false, true, true, false, true, true,  false, false, false, true, true,  false, // 10
    true,  true, false, false, true,  true, true, false, true, true,  true,  false, true,  true, true,  false, // 20
    true,  true, false, false, false, true, true, false, true, true,  false, false, false, true, true,  false, // 30
    true,  true, false, false, false, true, true, false, true, true,  true,  false, true,  true, true,  false, // 40
    true,  true, false, false, false, true, true, false, true, true,  false, false, false, true, true,  false, // 50
    true,  true, false, false, false, true, true, false, true, true,  true,  false, true,  true, true,  false, // 60
    true,  true, false, false, false, true, true, false, true, true,  false, false, false, true, true,  false, // 70
    false, true, false, false, true,  true, true, false, true, false, true,  false, true,  true, true,  false, // 80
    true,  true, false, false, true,  true, true, false, true, true,  true,  false, false, true, false, false, // 90
    true,  true, true,  false, true,  true, true, false, true, true,  true,  false, true,  true, true,  false, // A0
    true,  true, false, false, true,  true, true, false, true, true,  true,  false, true,  true, true,  false, // B0
    true,  true, false, false, true,  true, true, false, true, true,  true,  false, true,  true, true,  false, // C0
    true,  true, false, false, false, true, true, false, true, true,  false, false, false, true, true,  false, // D0
    true,  true, false, false, true,  true, true, false, true, true,  true,  false, true,  true, true,  false, // E0
    true,  true, false, false, false, true, true, false, true, true,  false, false, false, true, true,  false, // F0
];
