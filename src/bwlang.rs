// Copyright 2026 Cornell University
// Copyright 2026 Pedro Pontes García

//! This module provides an way to import bwlang rewrites from
//! https://github.com/luigirinaldi/parametric-bitvector-benchmarks

use crate::ir::RwProg;
use std::path::Path;
use tinyjson;
use tinyjson::JsonValue;

fn convert(bw: Rule) -> RwProg {
    todo!()
}

#[derive(Debug)]
struct Vars {
    width_vars: Vec<String>,
}

fn parse_to_bw(string: &str) -> Rule {
    let info = parse_json(string);
    let name = info.name;
    let lhs = parse_expr(&generic_s_expr(&info.lhs));
    let rhs = parse_expr(&generic_s_expr(&info.rhs));
    let preconditions: Vec<_> = info
        .preconditions
        .iter()
        .map(|s| parse_bool_expr(&generic_s_expr(s)))
        .collect();
    Rule {
        name,
        preconditions,
        lhs,
        rhs,
    }
}

#[derive(Debug, Clone)]
struct Rule {
    name: String,
    preconditions: Vec<BoolExpr>,
    lhs: Expr,
    rhs: Expr,
}

fn parse_width_expr(expr: &SExpr) -> WidthExpr {
    use SExpr::*;
    match expr {
        Leaf(name) => WidthExpr::Var(name.to_string()),
        other => todo!("{other:?} -> WidthExpr"),
    }
}

fn parse_bool_expr(expr: &SExpr) -> BoolExpr {
    use SExpr::*;
    match expr {
        List(items) => match items.as_slice() {
            [Leaf("<"), a, b] => {
                BoolExpr::Less(parse_width_expr(a).into(), parse_width_expr(b).into())
            }
            other => todo!("{other:?}"),
        },
        Leaf(name) => todo!("{name:?} -> BoolExpr"),
    }
}

fn parse_expr(expr: &SExpr) -> Expr {
    use SExpr::*;
    match expr {
        List(items) => match items.as_slice() {
            [Leaf("bw"), width, other] => {
                let mut underlying = parse_expr(other);
                underlying.set_width(parse_width_expr(width));
                underlying
            }
            [Leaf("+"), a, b] => Expr::Add(None, parse_expr(a).into(), parse_expr(b).into()),
            other => todo!("{other:?}"),
        },
        Leaf(name) => Expr::Var(None, name.to_string()),
    }
}

fn generic_s_expr(expr: &str) -> SExpr<'_> {
    use SExpr::*;
    let tokens = tokenize(expr);
    let mut stack = vec![];
    let mut open_count = 0;
    for token in tokens {
        match token {
            ")" => {
                assert!(open_count > 0, "too many closing parens!");
                open_count -= 1;
                let mut list = vec![];
                while let Some(e) = stack.pop()
                    && !matches!(e, Leaf("("))
                {
                    list.push(e);
                }
                list.reverse();
                stack.push(List(list));
            }
            "(" => {
                open_count += 1;
                stack.push(Leaf(token));
            }
            _ => {
                stack.push(Leaf(token));
            }
        }
    }
    assert_eq!(open_count, 0, "missing closing parens");
    assert_eq!(stack.len(), 1, "did not get a single root expression");
    stack.pop().unwrap()
}

fn tokenize(expr: &str) -> Vec<&str> {
    let mut token_start: Option<usize> = None;
    let mut tokens = vec![];
    for (idx, c) in expr.char_indices() {
        let is_paran = matches!(c, '(' | ')');
        if is_paran || c.is_ascii_whitespace() {
            if let Some(start) = token_start {
                tokens.push(&expr[start..idx]);
                token_start = None;
            }
        } else if token_start.is_none() {
            token_start = Some(idx);
        }
        if is_paran {
            tokens.push(&expr[idx..idx + 1]);
        }
    }
    tokens
}

#[derive(Debug, PartialEq)]
enum SExpr<'a> {
    List(Vec<SExpr<'a>>),
    Leaf(&'a str),
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

#[derive(Debug, Clone)]
enum WidthExpr {
    Var(String),
}

#[derive(Debug, Clone)]
enum BoolExpr {
    Less(Box<WidthExpr>, Box<WidthExpr>),
}

#[derive(Debug, Clone)]
enum Expr {
    Add(Option<WidthExpr>, Box<Expr>, Box<Expr>),
    Var(Option<WidthExpr>, String),
}

impl Expr {
    fn width(&self) -> &WidthExpr {
        self.width_option().unwrap()
    }

    fn width_option(&self) -> Option<&WidthExpr> {
        match self {
            Expr::Add(w, _, _) => w.as_ref(),
            Expr::Var(w, _) => w.as_ref(),
        }
    }

    fn set_width(&mut self, width: WidthExpr) {
        let w = match self {
            Expr::Add(w, _, _) => w,
            Expr::Var(w, _) => w,
        };
        *w = Some(width);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        parse_to_bw(ADD_ASSOC_4);
    }

    #[test]
    fn test_tokenize() {
        assert_eq!(
            tokenize("(bw u (+   (bw   p a) (bw      r b)))"),
            vec![
                "(", "bw", "u", "(", "+", "(", "bw", "p", "a", ")", "(", "bw", "r", "b", ")", ")",
                ")"
            ]
        );
    }

    #[test]
    fn test_generic_s_expr() {
        use SExpr::*;
        let expected = List(vec![
            Leaf("bw"),
            Leaf("u"),
            List(vec![
                Leaf("+"),
                List(vec![Leaf("bw"), Leaf("p"), Leaf("a")]),
                List(vec![Leaf("bw"), Leaf("r"), Leaf("b")]),
            ]),
        ]);
        assert_eq!(
            generic_s_expr("(bw u (+   (bw   p a) (bw      r b)))"),
            expected
        );
    }
}
