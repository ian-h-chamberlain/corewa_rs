//! In this phase, string lines are parsed into their memory representation.

use std::str::FromStr;

use pest::iterators::{Pair, Pairs};

use crate::error::Error;
use crate::load_file;
use crate::parser::grammar;

/// Convert the text input lins into in-memory data structures
pub fn deserialize(lines: Vec<String>) -> Result<load_file::Instructions, Error> {
    let mut instructions = Vec::with_capacity(lines.len());

    for line in lines.into_iter() {
        if let Some(parse_result) = grammar::parse(&line)?.next() {
            match &parse_result.as_rule() {
                grammar::Rule::Instruction => {
                    instructions.push(parse_instruction(parse_result.into_inner()));
                }
                rule => dbgf!("Unexpected rule {:?}", rule),
            }
        }
    }

    Ok(instructions)
}

fn parse_instruction(mut instruction_pairs: Pairs<grammar::Rule>) -> load_file::Instruction {
    let mut operation_pairs = instruction_pairs
        .next()
        .expect("Operation must be first pair after Label in Instruction")
        .into_inner();

    let opcode = parse_opcode(
        &operation_pairs
            .next()
            .expect("Opcode must be first pair in Operation"),
    );

    let maybe_modifier = operation_pairs
        .peek()
        .filter(|pair| pair.as_rule() == grammar::Rule::Modifier)
        .map(|pair| parse_modifier(&pair));

    let field_a = parse_field(
        instruction_pairs
            .next()
            .expect("Field must appear after Opcode"),
    );

    let field_b = instruction_pairs
        .next()
        .filter(|pair| pair.as_rule() == grammar::Rule::Field)
        .map_or_else(load_file::Field::default, parse_field);

    let modifier = maybe_modifier.unwrap_or_else(|| {
        load_file::Modifier::default_88_to_94(opcode, field_a.address_mode, field_b.address_mode)
    });

    load_file::Instruction {
        opcode,
        modifier,
        field_a,
        field_b,
    }
}

fn parse_modifier(modifier_pair: &Pair<grammar::Rule>) -> load_file::Modifier {
    load_file::Modifier::from_str(modifier_pair.as_str().to_uppercase().as_ref()).unwrap()
}

fn parse_opcode(opcode_pair: &Pair<grammar::Rule>) -> load_file::Opcode {
    load_file::Opcode::from_str(opcode_pair.as_str().to_uppercase().as_ref()).unwrap()
}

fn parse_field(field_pair: Pair<grammar::Rule>) -> load_file::Field {
    let mut field_pairs = field_pair.into_inner();

    let address_mode = field_pairs
        .peek()
        .filter(|pair| pair.as_rule() == grammar::Rule::AddressMode)
        .map_or(load_file::AddressMode::default(), |pair| {
            load_file::AddressMode::from_str(pair.as_str()).expect("Invalid AddressMode")
        });

    let value = parse_value(
        field_pairs
            .find(|pair| pair.as_rule() == grammar::Rule::Expr)
            .expect("No Expr in Field"),
    );

    load_file::Field {
        address_mode,
        value,
    }
}

fn parse_value(value_pair: Pair<grammar::Rule>) -> load_file::Value {
    let expr_inner = value_pair
        .into_inner()
        .next()
        .expect("Expr must have inner value");

    match expr_inner.as_rule() {
        grammar::Rule::Number => load_file::Value::Literal(
            i32::from_str_radix(expr_inner.as_str(), 10)
                .expect("Number type must be decimal integer"),
        ),
        grammar::Rule::Label => load_file::Value::Label(expr_inner.as_str().to_owned()),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use load_file::{Field, Instruction, Opcode};

    #[test]
    fn parse_simple_file() {
        let simple_input = [
            "mov 1, 3",
            "mov 100, #12",
            "dat #0, #0",
            "jmp +123, #45",
            "jmp -4",
            "jmp -1",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        let expected_core = vec![
            Instruction::new(Opcode::Mov, Field::direct(1), Field::direct(3)),
            Instruction::new(Opcode::Mov, Field::direct(100), Field::immediate(12)),
            Instruction::new(Opcode::Dat, Field::immediate(0), Field::immediate(0)),
            Instruction::new(Opcode::Jmp, Field::direct(123), Field::immediate(45)),
            Instruction::new(Opcode::Jmp, Field::direct(-4), Field::immediate(0)),
            Instruction::new(Opcode::Jmp, Field::direct(-1), Field::immediate(0)),
        ];

        let parsed = deserialize(simple_input)
            .unwrap_or_else(|err| panic!("Failed to parse simple file: {}", err));

        assert_eq!(parsed, expected_core);
    }
}
