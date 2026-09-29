use std::{fs, unimplemented};

use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

use crate::ir::{BitVecExpr, BitVecExprId, BitVecUnArithOp, Ctx, RwProg, RwRule, WidthExprId};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct RwParser;

pub fn parse_rw_file(file: &str, ctx: &mut Ctx) -> RwProg {
    let file = fs::read_to_string(file).expect("Could not read file.");
    let rprog = RwParser::parse(Rule::file, &file)
        .expect("Could not parse file.")
        .next()
        .unwrap();
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
    rparams
        .into_inner()
        .map(|x| x.as_str().to_string())
        .collect()
}

fn emit_body(rbody: Pair<'_, Rule>, ctx: &mut Ctx) -> (BitVecExprId, BitVecExprId) {
    let mut inner = rbody.into_inner();
    let rlhs = inner.next().unwrap();
    let lhs = emit_bvexpr(rlhs, ctx);
    let rrhs = inner.next().unwrap();
    let rhs = emit_bvexpr(rrhs, ctx);
    (lhs, rhs)
}

fn emit_bvexpr(rbvexpr: Pair<'_, Rule>, ctx: &mut Ctx) -> BitVecExprId {
    let mut inner = rbvexpr.into_inner();
    let atom = inner.next().unwrap();
    let width_suffix = inner.next().unwrap();
    let width = emit_wexpr(width_suffix, ctx);
    emit_bvatom(atom, ctx, width)
}

fn emit_wexpr(rwexpr: Pair<'_, Rule>, ctx: &mut Ctx) -> WidthExprId {
    Default::default()
}

fn emit_bvatom(rbvatom: Pair<'_, Rule>, ctx: &mut Ctx, w: WidthExprId) -> BitVecExprId {
    let mut inner = rbvatom.into_inner();
    let rule = inner.next().unwrap();
    match rule.as_rule() {
        Rule::num_lit => ctx.insert_bv(BitVecExpr::Const(w, rule.as_str().parse::<u64>().unwrap())),
        Rule::func_call => emit_bvcall(rule, ctx, w),
        Rule::identifier => ctx.insert_bv(BitVecExpr::Var(w, rule.as_str().to_string())),
        Rule::concat_expr => emit_bvcat(rule, ctx, w),
        Rule::unary_expr => emit_bvunop(rule, ctx, w),
        Rule::ary_expr => emit_bvary(rule, ctx, w),
        unr => {
            println!("Reached unreachable {:#?}", unr);
            unreachable!()
        }
    }
}

fn emit_bvcall(rbvcall: Pair<'_, Rule>, ctx: &mut Ctx, w: WidthExprId) -> BitVecExprId {
    unimplemented!()
}

fn emit_bvcat(rbvcat: Pair<'_, Rule>, ctx: &mut Ctx, w: WidthExprId) -> BitVecExprId {
    let mut inner = rbvcat.into_inner();
    let lexpr = inner.next().unwrap();
    let rexpr = inner.next().unwrap();
    let expr = BitVecExpr::Cat(w, emit_bvatom(lexpr, ctx, w), emit_bvatom(rexpr, ctx, w));
    ctx.insert_bv(expr)
}

fn emit_bvunop(rbvunop: Pair<'_, Rule>, ctx: &mut Ctx, w: WidthExprId) -> BitVecExprId {
    let mut inner = rbvunop.into_inner();
    let op = inner.next().unwrap();
    let rexpr = inner.next().unwrap();
    let expr = BitVecExpr::UnArith(
        w,
        match op.as_rule() {
            Rule::not_op => BitVecUnArithOp::Not,
            Rule::neg_op => BitVecUnArithOp::Neg,
            unr => {
                println!("Reached unreachable {:#?}", unr);
                unreachable!()
            }
        },
        emit_bvatom(rexpr, ctx, w),
    );
    ctx.insert_bv(expr)
}

fn emit_bvary(rbvary: Pair<'_, Rule>, ctx: &mut Ctx, w: WidthExprId) -> BitVecExprId {
    unimplemented!()
}
