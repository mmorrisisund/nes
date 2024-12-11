pub struct Instruction {
    pub byte_value: u8,
    pub operand: u16,
    pub op_code: OpCode,
    pub addressing_mode: AddressingMode,
}

impl Instruction {
    pub fn from_byte(byte: u8) -> Self {
        Self {
            byte_value: byte,
            operand: 0,
            op_code: OPCODES[byte as usize],
            addressing_mode: ADDRESSING_MODES[byte as usize],
        }
    }

    pub fn get_instruction_len(&self) -> usize {
        self.addressing_mode.instruction_length()
    }

    pub fn get_base_cycle_time(&self) -> u8 {
        self.addressing_mode.cycle_length()
    }

    pub fn operand_is_address(&self) -> bool {
        self.addressing_mode.operand_is_address()
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

#[derive(Clone, Copy, PartialEq, Eq)]
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

    // Unofficial
    Slo, // Arithmetic shift left, then OR [Asl + Ora]
    Sre, // Logical shift right, then XOR [Lsr + Eor]
    Rla, // Rotate left, then And [Rol + And]
    Rra, // Rotate right, then Add with carry [Ror + Adc]
    Isc, // Increment memory, then Subtract with carry [Inc + Sbc]
    Dcp, // Decrement memory, then Compare A [Dec + Cmp]
    Sax, // Store the value of (A and X)
    Lax, // Load A and X [Lda + Tax]

    Anc, // And #imm, then set Carry = Negative
    Alr, // And #imm, then Lsr A
    Arr, // And #imm, then Ror A
    Atx, // And #imm -> X
    Axs, // ((A and X) - #imm) -> X
    Xaa, // Transfer X to A, then And #imm [Txa + And #imm]

    Axa, // Store the value of (A and X and {High byte of $addr}) into $addr
    Shy, // Store the value of (Y and {High byte of $addr}) into $addr
    Shx, // Store the value of (X and {High byte of $addr}) into $addr

    Tas, // Store the value of (X and A) into (the stack pointer), and store ((the stack pointer) and {High byte of $addr}) into $addr
    Las, // Store the value of ({value in $addr} & (the stack pointer)) into A, X, and (the stack pointer)

    Kil, // Halt the CPU (CRASH)
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
            Atx => "ATX",
            Axs => "AXS",
            Xaa => "XAA",
            Axa => "AXA",
            Shy => "SHY",
            Shx => "SHX",
            Tas => "TAS",
            Las => "LAS",
            Kil => "KIL",
        };
        write!(f, "{code}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
    pub fn can_cross_page(&self) -> bool {
        self == &AddressingMode::AbsoluteX
            || self == &AddressingMode::AbsoluteY
            || self == &AddressingMode::IndirectIndexed
    }

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

    fn cycle_length(&self) -> u8 {
        match self {
            Implied => 2,
            Relative => 2,
            Immediate => 2,
            Accumulator => 2,
            ZeroPage => 3,
            ZeroPageX => 4,
            ZeroPageY => 4,
            Absolute => 4,
            AbsoluteX => 4,
            AbsoluteY => 4,
            Indirect => 5,
            IndexedIndirect => 6,
            IndirectIndexed => 5,
        }
    }

    fn operand_is_address(&self) -> bool {
        !(self == &AddressingMode::Implied
            || self == &AddressingMode::Immediate
            || self == &AddressingMode::Accumulator)
    }
}

use std::fmt::Display;

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
    Nop, Sta, Nop, Axs, Sty, Sta, Stx, Axs, Dey, Nop, Txa, Xaa, Sty, Sta, Stx, Axs, // 80
    Bcc, Sta, Kil, Axa, Sty, Sta, Stx, Axs, Tya, Sta, Txs, Tas, Shy, Sta, Shx, Axa, // 90
    Ldy, Lda, Ldx, Lax, Ldy, Lda, Ldx, Lax, Tay, Lda, Tax, Atx, Ldy, Lda, Ldx, Lax, // A0
    Bcs, Lda, Kil, Lax, Ldy, Lda, Ldx, Lax, Clv, Lda, Tsx, Las, Ldy, Lda, Ldx, Lax, // B0
    Cpy, Cmp, Nop, Dcp, Cpy, Cmp, Dec, Dcp, Iny, Cmp, Dex, Axs, Cpy, Cmp, Cmp, Dcp, // C0
    Bne, Cmp, Kil, Dcp, Nop, Cmp, Dec, Dcp, Cld, Cmp, Nop, Dcp, Nop, Cmp, Cmp, Dcp, // D0
    Cpx, Sbc, Nop, Isc, Cpx, Sbc, Inc, Isc, Inx, Sbc, Nop, Sbc, Cpx, Sbc, Inc, Isc, // E0
    Beq, Sbc, Kil, Isc, Nop, Sbc, Inc, Isc, Sed, Sbc, Nop, Isc, Nop, Sbc, Inc, Isc, // F0
];

use AddressingMode::*;

#[rustfmt::skip]
const ADDRESSING_MODES: [AddressingMode; 256] = [
//  0          1                2            3                4          5          6          7    
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
