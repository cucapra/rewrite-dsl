// Copyright 2026 Cornell University
// Copyright 2026 Pedro Pontes García

//! This module provides an way to import bwlang rewrites from
//! https://github.com/luigirinaldi/parametric-bitvector-benchmarks

use std::path::Path;
use tinyjson;
use tinyjson::JsonValue;

pub fn parse_file(filename: impl AsRef<Path>) {
    let string = std::fs::read_to_string(filename).unwrap();
    parse(&string)
}

pub fn parse(string: &str) {
    let info = parse_json(string);
    parse_s_expr(&info.lhs);
}

fn parse_s_expr(expr: &str) {
    println!("{expr}");
}

fn parse_json(string: &str) -> JsonInfo {
    let value: JsonValue = string.parse().unwrap();
    if let JsonValue::Object(mut map) = value {
        let Some(JsonValue::String(name)) = map.remove("name") else {
            panic!()
        };
        let Some(JsonValue::String(lhs)) = map.remove("lhs") else {
            panic!()
        };
        let Some(JsonValue::String(rhs)) = map.remove("rhs") else {
            panic!()
        };
        let Some(JsonValue::Array(cond_strs)) = map.remove("preconditions") else {
            panic!()
        };
        let mut preconditions = Vec::with_capacity(cond_strs.len());
        for cond in cond_strs {
            let JsonValue::String(cond) = cond else {
                panic!()
            };
            preconditions.push(cond);
        }
        JsonInfo {
            name,
            lhs,
            rhs,
            preconditions,
        }
    } else {
        panic!("Unexpected JSON format");
    }
}

#[derive(Debug)]
struct JsonInfo {
    name: String,
    lhs: String,
    rhs: String,
    preconditions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use crate::bwlang::parse;

    const ADD_ASSOC_4: &str = r##"
    {
    "name": "add_assoc_4",
    "preconditions": [
        "(< r q)",
        "(< s q)",
        "(< p u)",
        "(< r u)"
    ],
    "lhs": "(bw t ( + (bw u (+ (bw p a) (bw r b))) (bw s c)))",
    "rhs": "(bw t ( + (bw p a) (bw q (+ (bw r b) (bw s c)))))"
}
    "##;

    #[test]
    fn test_parse() {
        parse(ADD_ASSOC_4);
    }
}
