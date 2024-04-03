use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint8};
use tfhe::prelude::*;

use regex::{Captures, Regex};
use std::collections::HashMap;
use std::vec;

static GATE_PATTERN: &str = r"(?P<input_count>\d+)\s(?P<output_count>\d+)\s(?P<xref>\d+)\s(?:(?P<yref>\d+)\s)?(?P<zref>\d+)\s(?P<gate>INV|AND|XOR)";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GateType {
    /// XOR gate.
    Xor,
    /// AND gate.
    And,
    /// Inverter gate.
    Inv,
}

pub fn parse(
    filename: &str,
    inputs: &[i32],
    outputs: &[i32],
) {
    let file = std::fs::read_to_string(filename)?;

    let mut input_len = 0;
    for input in inputs {
        input_len += input.len();
    }

    let pattern = Regex::new(GATE_PATTERN).unwrap();
    for cap in pattern.captures_iter(&file) {
        let UncheckedGate {
            xref,
            yref,
            zref,
            gate_type,
        } = UncheckedGate::parse(cap)?;

        match gate_type {
            GateType::Xor => {
                println!("{} {} {} {}", xref, yref.unwrap(), zref, gate_type);
            }
            GateType::And => {
                println!("{} {} {} {}", xref, yref.unwrap(), zref, gate_type);
            }
            GateType::Inv => {
                println!("{} {} {} {}", xref, yref.unwrap(), zref, gate_type);
            }
        }
    }

    for output in outputs.iter().rev() {
        
    }
}

struct UncheckedGate {
    xref: usize,
    yref: Option<usize>,
    zref: usize,
    gate_type: GateType,
}

impl UncheckedGate {
    fn parse(captures: Captures) -> Self {
        let xref: usize = captures.name("xref").unwrap().as_str().parse()?;
        let yref: Option<usize> = captures
            .name("yref")
            .map(|yref| yref.as_str().parse())
            .transpose()?;
        let zref: usize = captures.name("zref").unwrap().as_str().parse()?;
        let gate_type = captures.name("gate").unwrap().as_str();

        let gate_type = match gate_type {
            "XOR" => GateType::Xor,
            "AND" => GateType::And,
            "INV" => GateType::Inv,
            _ => GateType::Inv,
        };

        Ok(Self {
            xref,
            yref,
            zref,
            gate_type,
        })
    }
}


fn main() {
    parse("adder64_reverse.txt", &vec![], &vec![]);
    // let config = ConfigBuilder::default().build();

    // // Client-side
    // let (client_key, server_key) = generate_keys(config);

    // let clear_a = 27u8;
    // let clear_b = 128u8;

    // let a = FheUint8::encrypt(clear_a, &client_key);
    // let b = FheUint8::encrypt(clear_b, &client_key);

    // //Server-side
    // set_server_key(server_key);
    // let result = a.gt(b);

    // //Client-side
    // let decrypted_result: u8 = result.decrypt(&client_key) as u8;

    // let clear_result = (clear_a > clear_b) as u8;
    // println!("{}", decrypted_result);
    // println!("{}", clear_result);

    // assert_eq!(decrypted_result, clear_result);

}
