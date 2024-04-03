use tfhe::{ConfigBuilder, generate_keys, set_server_key, FheUint8};
use tfhe::prelude::*;

use regex::{Captures, Regex};
use std::collections::HashMap;

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

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("uninitialized feed: {0}")]
    UninitializedFeed(usize),
    #[error("unsupported gate type: {0}")]
    UnsupportedGateType(String),
    #[error(transparent)]
    BuilderError(#[from] crate::BuilderError),
}

impl Circuit {
    /// Parses a circuit in Bristol-fashion format from a file.
    ///
    /// See `https://homes.esat.kuleuven.be/~nsmart/MPC/` for more information.
    ///
    /// # Arguments
    ///
    /// * `filename` - The path to the file to parse.
    /// * `inputs` - The types of the inputs to the circuit.
    /// * `outputs` - The types of the outputs to the circuit.
    ///
    /// # Returns
    ///
    /// The parsed circuit.
    pub fn parse(
        filename: &str,
        inputs: &[ValueType],
        outputs: &[ValueType],
    ) -> Result<Self, ParseError> {
        let file = std::fs::read_to_string(filename)?;

        let builder = CircuitBuilder::new();

        let mut feed_ids: Vec<usize> = Vec::new();
        let mut feed_map: HashMap<usize, Node<Feed>> = HashMap::default();

        let mut input_len = 0;
        for input in inputs {
            let input = builder.add_input_by_type(input.clone());
            for (node, old_id) in input.iter().zip(input_len..input_len + input.len()) {
                feed_map.insert(old_id, *node);
            }
            input_len += input.len();
        }

        let mut state = builder.state().borrow_mut();
        let pattern = Regex::new(GATE_PATTERN).unwrap();
        for cap in pattern.captures_iter(&file) {
            let UncheckedGate {
                xref,
                yref,
                zref,
                gate_type,
            } = UncheckedGate::parse(cap)?;
            feed_ids.push(zref);

            match gate_type {
                GateType::Xor => {
                    let new_x = feed_map
                        .get(&xref)
                        .ok_or(ParseError::UninitializedFeed(xref))?;
                    let new_y = feed_map
                        .get(&yref.unwrap())
                        .ok_or(ParseError::UninitializedFeed(yref.unwrap()))?;
                    let new_z = state.add_xor_gate(*new_x, *new_y);
                    feed_map.insert(zref, new_z);
                }
                GateType::And => {
                    let new_x = feed_map
                        .get(&xref)
                        .ok_or(ParseError::UninitializedFeed(xref))?;
                    let new_y = feed_map
                        .get(&yref.unwrap())
                        .ok_or(ParseError::UninitializedFeed(yref.unwrap()))?;
                    let new_z = state.add_and_gate(*new_x, *new_y);
                    feed_map.insert(zref, new_z);
                }
                GateType::Inv => {
                    let new_x = feed_map
                        .get(&xref)
                        .ok_or(ParseError::UninitializedFeed(xref))?;
                    let new_z = state.add_inv_gate(*new_x);
                    feed_map.insert(zref, new_z);
                }
            }
        }
        drop(state);
        feed_ids.sort();

        for output in outputs.iter().rev() {
            let feeds = feed_ids
                .drain(feed_ids.len() - output.len()..)
                .map(|id| {
                    *feed_map
                        .get(&id)
                        .expect("Old feed should be mapped to new feed")
                })
                .collect::<Vec<Node<Feed>>>();

            let output = output.to_bin_repr(&feeds).unwrap();
            builder.add_output(output);
        }

        Ok(builder.build()?)
    }
}

struct UncheckedGate {
    xref: usize,
    yref: Option<usize>,
    zref: usize,
    gate_type: GateType,
}

impl UncheckedGate {
    fn parse(captures: Captures) -> Result<Self, ParseError> {
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
            _ => return Err(ParseError::UnsupportedGateType(gate_type.to_string())),
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
    let config = ConfigBuilder::default().build();

    // Client-side
    let (client_key, server_key) = generate_keys(config);

    let clear_a = 27u8;
    let clear_b = 128u8;

    let a = FheUint8::encrypt(clear_a, &client_key);
    let b = FheUint8::encrypt(clear_b, &client_key);

    //Server-side
    set_server_key(server_key);
    let result = a.gt(b);

    //Client-side
    let decrypted_result: u8 = result.decrypt(&client_key) as u8;

    let clear_result = (clear_a > clear_b) as u8;
    println!("{}", decrypted_result);
    println!("{}", clear_result);

    assert_eq!(decrypted_result, clear_result);
}
