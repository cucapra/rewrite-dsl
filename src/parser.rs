use std::fs;

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

use crate::ir::{RwProg, RwRule};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct RwParser;

pub fn parse_rw_file(file: &str) -> RwProg {
    let file = fs::read_to_string(file).expect("Could not read file.");
    let rprog = RwParser::parse(Rule::file, &file).expect("Could not parse file.").next().unwrap();
    let prog: Vec<RwRule> = emit_prog(rprog);
    println!("{:#?}", prog);
    prog
}

fn emit_prog(rprog: Pair<'_, Rule>) -> RwProg {
    let mut prog = RwProg::new();
    for rrule in rprog.into_inner() {
        match rrule.as_rule() {
            Rule::rwrule => {
                prog.push(emit_rule(rrule));
            }
            Rule::EOI => {}
            unr => {
                println!("Reached unreachable {:#?}", unr)
            }
        }
    }
    prog
}

fn emit_rule(rrule: Pair<'_, Rule>) -> RwRule {
    let mut rule: RwRule = Default::default();
    for inner in rrule.into_inner() {
        match inner.as_rule() {
            Rule::identifier => {
                rule.name = inner.to_string();
            }
            Rule::params => {
                rule.width_vars = emit_params(inner);
            }
            Rule::rule_body => {

            }
            Rule::rule_word => {}
            unr => {
                println!("Reached unreachable {:#?}", unr)
            }
        }
    }
    rule
}

fn emit_params(rparams: Pair<'_, Rule>) -> Vec<String> {
    rparams.into_inner().map(|x| x.as_str().to_string()).collect()
}

fn emit_body(rbody) {

}

fn emit_bvexpr(rbvexpr) -> BitVecExpr
