use std::{fs, unimplemented};

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

use crate::ir::{BitVecExpr, BitVecExprId, RwProg, RwRule, Ctx};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct RwParser;

pub fn parse_rw_file(file: &str, ctx: &mut Ctx) -> RwProg {
    let file = fs::read_to_string(file).expect("Could not read file.");
    let rprog = RwParser::parse(Rule::file, &file).expect("Could not parse file.").next().unwrap();
    let prog: Vec<RwRule> = emit_prog(rprog, ctx);
    println!("{:#?}", prog);
    prog
}

fn emit_prog(rprog: Pair<'_, Rule>, ctx: &mut Ctx) -> RwProg {
    let mut prog = RwProg::new();
    for rrule in rprog.into_inner() {
        match rrule.as_rule() {
            Rule::rwrule => {
                prog.push(emit_rule(rrule, ctx));
            }
            Rule::EOI => {}
            unr => {
                println!("Reached unreachable {:#?}", unr)
            }
        }
    }
    prog
}

fn emit_rule(rrule: Pair<'_, Rule>, ctx: &mut Ctx) -> RwRule {
    let mut rule: RwRule = Default::default();
    for inner in rrule.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                rule.name = inner.to_string();
            }
            Rule::params => {
                rule.width_vars = emit_params(inner, ctx);
            }
            Rule::rule_body => {
                let (lhs, rhs) = emit_body(inner, ctx);
                rule.lhs = lhs;
                rule.rhs = rhs;
            }
            Rule::rule_word => {}
            unr => {
                println!("Reached unreachable {:#?}", unr)
            }
        }
    }
    rule
}

fn emit_params(rparams: Pair<'_, Rule>, ctx: &mut Ctx) -> Vec<String> {
    rparams.into_inner().map(|x| x.as_str().to_string()).collect()
}

fn emit_body(rbody: Pair<'_, Rule>, ctx: &mut Ctx) -> (BitVecExprId, BitVecExprId) {
    let (lhs, rhs): (BitVecExprId, BitVecExprId) = Default::default();
    for inner in rbody.into_inner() {
        match inner.as_rule() {
            Rule::lhs => {
                lhs = emit_bvexpr(inner, ctx)
            }
            Rule::rhs => {
                rhs = emit_bvexpr(inner, ctx)
            }
            unr => {
                println!("Reached unreachable {:#?}", unr)
            }
        }
    }
    (lhs, rhs)
}

// bv_primary = {
//     bv_atom ~ width_suffix?
// }

// bv_atom = {
//       num_lit
//     | func_call
//     | identifier
//     | concat_expr
//     | "(" ~ unary_op ~ bv_expr ~ ")"
//     | "(" ~ bv_expr ~ (bin_op ~ bv_expr | ternary_op ~ bv_expr ~ ternary_colon ~ bv_expr) ~ ")"
// }
fn emit_bvexpr(rbvexpr: Pair<'_, Rule>, ctx: &mut Ctx) -> BitVecExprId {
    let bv: BitVecExpr = Default::default();
    for inner in rbvexpr.into_inner() {
        match inner.as_rule() {
            Rule::bv_atom => {
                for inner_atom in inner.into_inner() {
                    match inner_atom.as_rule() {
                        
                    }
                }
            }
            Rule::width_suffix => {
                
            }
            unr => {
                println!("Reached unreachable {:#?}", unr)
            }
        }
    }
    ctx.insert_bv(unimplemented!())
}
