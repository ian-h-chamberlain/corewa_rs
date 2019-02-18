use std::{fmt, string::ToString, vec};

pub const DEFAULT_CORE_SIZE: usize = 8000;

enum_string!(pub Opcode, {
    Dat => "DAT",
    Mov => "MOV",
    Add => "ADD",
    Sub => "SUB",
    Mul => "MUL",
    Div => "DIV",
    Mod => "MOD",
    Jmp => "JMP",
    Jmz => "JMZ",
    Jmn => "JMN",
    Djn => "DJN",
    Cmp => "CMP",
    Seq => "SEQ",
    Sne => "SNE",
    Slt => "SLT",
    Spl => "SPL",
    Nop => "NOP",
});

impl Default for Opcode {
    fn default() -> Opcode {
        Opcode::Dat
    }
}

enum_string!(pub PseudoOpcode, {
    Org => "ORG",
    Equ => "EQU",
    End => "END",
});

enum_string!(pub Modifier, {
    A   => "A",
    B   => "B",
    AB  => "AB",
    BA  => "BA",
    F   => "F",
    X   => "X",
    I   => "I",
});

impl Default for Modifier {
    fn default() -> Modifier {
        Modifier::F
    }
}

impl Modifier {
    pub fn from_context(opcode: Opcode, a_mode: AddressMode, b_mode: AddressMode) -> Modifier {
        /// Implemented based on the ICWS '94 document,
        /// section A.2.1.2: ICWS'88 to ICWS'94 Conversion
        use Opcode::*;

        match opcode {
            Dat => Modifier::F,
            Jmp | Jmz | Jmn | Djn | Spl | Nop => Modifier::B,
            opcode => {
                if a_mode == AddressMode::Immediate {
                    Modifier::AB
                } else if b_mode == AddressMode::Immediate {
                    Modifier::B
                } else {
                    match opcode {
                        Mov | Cmp | Seq | Sne => Modifier::I,
                        Slt => Modifier::B,
                        Add | Sub | Mul | Div | Mod => Modifier::F,
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}

enum_string!(pub AddressMode, {
    Immediate           => "#",
    Direct              => "$",
    IndirectA           => "*",
    IndirectB           => "@",
    PreDecIndirectA     => "{",
    PreDecIndirectB     => "<",
    PostIncIndirectA    => "}",
    PostIncIndirectB    => ">",
});

impl Default for AddressMode {
    fn default() -> AddressMode {
        AddressMode::Direct
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Field {
    pub address_mode: AddressMode,
    pub value: i32,
}

impl Field {
    pub fn direct(value: i32) -> Field {
        Field {
            address_mode: AddressMode::Direct,
            value,
        }
    }

    pub fn immediate(value: i32) -> Field {
        Field {
            address_mode: AddressMode::Immediate,
            value,
        }
    }
}

impl ToString for Field {
    fn to_string(&self) -> String {
        format!("{}{}", self.address_mode.to_string(), self.value)
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Instruction {
    pub opcode: Opcode,
    pub modifier: Modifier,
    pub field_a: Field,
    pub field_b: Field,
}

impl Instruction {
    pub fn new(opcode: Opcode, field_a: Field, field_b: Field) -> Instruction {
        let modifier = Modifier::from_context(opcode, field_a.address_mode, field_b.address_mode);
        Instruction {
            opcode,
            modifier,
            field_a,
            field_b,
        }
    }
}

impl ToString for Instruction {
    fn to_string(&self) -> String {
        format!(
            "{} {}, {}",
            self.opcode.to_string(),
            self.field_a.to_string(),
            self.field_b.to_string(),
        )
    }
}

pub struct Core {
    instructions: vec::Vec<Instruction>,
}

impl Core {
    pub fn new(core_size: usize) -> Core {
        Core {
            instructions: vec![Instruction::default(); core_size],
        }
    }

    pub fn get(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }

    pub fn set(&mut self, index: usize, value: Instruction) {
        self.instructions[index] = value;
    }

    pub fn dump_all(&self) -> String {
        self.instructions
            .iter()
            .fold(String::new(), |result, instruction| {
                result + &instruction.to_string() + "\n"
            })
    }

    pub fn dump(&self) -> String {
        self.instructions
            .iter()
            .filter(|&instruction| *instruction != Instruction::default())
            .fold(String::new(), |result, instruction| {
                result + &instruction.to_string() + "\n"
            })
    }
}

impl Default for Core {
    fn default() -> Core {
        Core::new(DEFAULT_CORE_SIZE)
    }
}

impl fmt::Debug for Core {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.dump_all())
    }
}

impl PartialEq for Core {
    fn eq(&self, rhs: &Self) -> bool {
        for (self_instruction, other_instruction) in
            self.instructions.iter().zip(rhs.instructions.iter())
        {
            if self_instruction != other_instruction {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use itertools::iproduct;
    use lazy_static::lazy_static;

    use std::str::FromStr;

    use super::*;

    lazy_static! {
        static ref all_modes: vec::Vec<AddressMode> = ["#", "$", "@", "<", ">", "*", "{", "}"]
            .iter()
            .map(|mode_str| AddressMode::from_str(mode_str).unwrap())
            .collect();
    }

    #[test]
    fn default_instruction() {
        let expected_instruction = Instruction {
            opcode: Opcode::Dat,
            modifier: Modifier::F,
            field_a: Field {
                address_mode: AddressMode::Direct,
                value: 0,
            },
            field_b: Field {
                address_mode: AddressMode::Direct,
                value: 0,
            },
        };

        assert_eq!(Instruction::default(), expected_instruction)
    }

    #[test]
    fn dat_from_context() {
        for (&a_mode, &b_mode) in iproduct!(all_modes.iter(), all_modes.iter()) {
            assert_eq!(
                Modifier::from_context(Opcode::Dat, a_mode, b_mode),
                Modifier::F
            );
        }
    }

    #[test]
    fn modifier_b_from_context() {
        use Opcode::*;

        let opcodes = [Jmp, Jmz, Jmn, Djn, Spl, Nop];

        for (&a_mode, &b_mode) in iproduct!(all_modes.iter(), all_modes.iter()) {
            for &opcode in opcodes.iter() {
                assert_eq!(Modifier::from_context(opcode, a_mode, b_mode), Modifier::B);
            }
        }

        let a_modes: vec::Vec<AddressMode> = ["$", "@", "<", ">", "*", "{", "}"]
            .iter()
            .map(|mode_str| AddressMode::from_str(mode_str).unwrap())
            .collect();

        for &a_mode in a_modes.iter() {
            for &b_mode in all_modes.iter() {
                assert_eq!(
                    Modifier::from_context(Opcode::Slt, a_mode, b_mode),
                    Modifier::B
                )
            }
        }

        // TODO Mov, Cmp. Seq, Sne -> B
    }

    #[test]
    fn modifier_ab_from_context() {
        use Opcode::*;

        let opcodes = [Mov, Cmp, Seq, Sne, Add, Sub, Mul, Div, Mod, Slt];

        for &b_mode in all_modes.iter() {
            for &opcode in opcodes.iter() {
                assert_eq!(
                    Modifier::from_context(opcode, AddressMode::Immediate, b_mode),
                    Modifier::AB
                );
            }
        }
    }

    // TODO Mode I
}
