use std::{cell::RefCell, fs, num::NonZeroU32, unimplemented};

use pest::{
    Parser,
    iterators::Pair,
    pratt_parser::{Assoc, Op, PrattParser},
};
use pest_derive::Parser;

use crate::ir::{
    BitVecBinArithOp, BitVecExpr, BitVecExprId, BitVecUnArithOp, BoolExpr, BoolExprId, Ctx, RwProg,
    RwRule, WidthBinOp, WidthExpr, WidthExprId,
};

lazy_static::lazy_static! {
    static ref PRATT: PrattParser<Rule> =
    PrattParser::new()
        .op(Op::infix(Rule::width_add, Assoc::Left) | Op::infix(Rule::width_sub, Assoc::Left));

    static ref BOOL_PRATT: PrattParser<Rule> =
    PrattParser::new()
        .op(Op::infix(Rule::or_op, Assoc::Left))
        .op(Op::infix(Rule::and_op, Assoc::Left));
}

fn parse_num_lit(text: &str) -> Result<u64, std::num::ParseIntError> {
    let (digits, radix) = if let Some(digits) = text.strip_prefix("0b") {
        (digits, 2)
    } else if let Some(digits) = text.strip_prefix("0x") {
        (digits, 16)
    } else if let Some(digits) = text.strip_prefix("0o") {
        (digits, 8)
    } else {
        (text, 10)
    };

    u64::from_str_radix(digits, radix)
}

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct RwParser;

pub fn parse_rw_file(file: &str, ctx: &mut Ctx) -> RwProg {
    let file = fs::read_to_string(file).expect("Could not read file.");
    let rprog = RwParser::parse(Rule::file, &file)
        .expect("Could not parse file.")
        .next()
        .unwrap();
    emit_prog(rprog, ctx)
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
                rule.name = inner.as_str().to_string();
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

fn emit_params(rparams: Pair<'_, Rule>, _ctx: &mut Ctx) -> Vec<String> {
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
    let width_atom = width_suffix.into_inner().next().unwrap();
    let width = emit_watom(width_atom, ctx);
    emit_bvatom(atom, ctx, width)
}

fn emit_wexpr(rwexpr: Pair<'_, Rule>, ctx: &mut Ctx) -> WidthExprId {
    let mut inner = rwexpr.into_inner();
    let rule = inner.next().unwrap();
    match rule.as_rule() {
        Rule::width_ite_atom => {
            let mut inner = rule.into_inner();
            let bexpr = emit_bexpr(inner.next().unwrap(), ctx);
            let lexpr = emit_wexpr(inner.next().unwrap(), ctx);
            let rexpr = emit_wexpr(inner.next().unwrap(), ctx);
            ctx.insert_width(WidthExpr::Ite(bexpr, lexpr, rexpr))
        }
        Rule::width_binop => {
            let inner = rule.into_inner();
            let ctx = RefCell::new(ctx);

            PRATT
                .map_primary(|atom| {
                    let mut ctx = ctx.borrow_mut();
                    match atom.as_rule() {
                        Rule::width_atom => emit_watom(atom, &mut ctx),
                        rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
                    }
                })
                .map_infix(|lhs, op, rhs| {
                    let op = match op.as_rule() {
                        Rule::width_add => WidthBinOp::Add,
                        Rule::width_sub => WidthBinOp::Sub,
                        unr => unreachable!("Reached unreachable {:#?}", unr),
                    };
                    ctx.borrow_mut()
                        .insert_width(WidthExpr::BinOp(op, lhs, rhs))
                })
                .parse(inner)
        }
        unr => {
            println!("Reached unreachable {:#?}", unr);
            unreachable!()
        }
    }
}

fn emit_watom(rwatom: Pair<'_, Rule>, ctx: &mut Ctx) -> WidthExprId {
    let mut inner = rwatom.into_inner();
    let rule = inner.next().unwrap();
    match rule.as_rule() {
        Rule::identifier => ctx.insert_width(WidthExpr::Var(rule.as_str().to_string())),
        Rule::bitwidth => ctx.insert_width(WidthExpr::Const(
            rule.as_str().parse::<NonZeroU32>().unwrap(),
        )),
        Rule::width_expr => emit_wexpr(rule, ctx),
        unr => {
            println!("Reached unreachable {:#?}", unr);
            unreachable!()
        }
    }
}

fn emit_bexpr(rbexpr: Pair<'_, Rule>, ctx: &mut Ctx) -> BoolExprId {
    let rule = match rbexpr.as_rule() {
        Rule::bool_expr => rbexpr.into_inner().next().unwrap(),
        _ => rbexpr,
    };

    match rule.as_rule() {
        Rule::bool_or | Rule::bool_and => {
            let inner = rule.into_inner();
            let ctx = RefCell::new(ctx);

            BOOL_PRATT
                .map_primary(|atom| {
                    let mut ctx = ctx.borrow_mut();
                    emit_bexpr(atom, &mut ctx)
                })
                .map_infix(|lhs, op, rhs| {
                    let expr = match op.as_rule() {
                        Rule::bool_or_op => BoolExpr::Or(lhs, rhs),
                        Rule::bool_and_op => BoolExpr::And(lhs, rhs),
                        unr => unreachable!("Reached unreachable {:#?}", unr),
                    };
                    ctx.borrow_mut().insert_bool(expr)
                })
                .parse(inner)
        }
        Rule::bool_not => {
            let mut inner = rule.into_inner();
            inner.next().unwrap();
            let rhs = emit_bexpr(inner.next().unwrap(), ctx);
            ctx.insert_bool(BoolExpr::Not(rhs))
        }
        Rule::bool_true => ctx.insert_bool(BoolExpr::Const(true)),
        Rule::bool_false => ctx.insert_bool(BoolExpr::Const(false)),
        Rule::width_cmp => {
            let mut inner = rule.into_inner();
            let lhs = emit_wexpr(inner.next().unwrap(), ctx);
            let op = inner.next().unwrap();
            let rhs = emit_wexpr(inner.next().unwrap(), ctx);

            let (expr, negate) = match op.as_rule() {
                Rule::geq => (BoolExpr::Gt(rhs, lhs), true),
                Rule::leq => (BoolExpr::Gt(lhs, rhs), true),
                Rule::eq => (BoolExpr::Eq(lhs, rhs), false),
                Rule::gt => (BoolExpr::Gt(lhs, rhs), false),
                Rule::lt => (BoolExpr::Gt(rhs, lhs), false),
                unr => unreachable!("Reached unreachable {:#?}", unr),
            };
            let expr = ctx.insert_bool(expr);
            if negate {
                ctx.insert_bool(BoolExpr::Not(expr))
            } else {
                expr
            }
        }
        unr => unreachable!("Reached unreachable {:#?}", unr),
    }
}

fn emit_bvatom(rbvatom: Pair<'_, Rule>, ctx: &mut Ctx, w: WidthExprId) -> BitVecExprId {
    let mut inner = rbvatom.into_inner();
    let rule = inner.next().unwrap();
    match rule.as_rule() {
        Rule::num_lit => ctx.insert_bv(BitVecExpr::Const(w, parse_num_lit(rule.as_str()).unwrap())),
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
    let mut inner = rbvcall.into_inner();
    let id = inner.next().unwrap();
    let too_few = || {
        println!("Too few arguments!");
        unreachable!()
    };
    let mut args = inner.map(|arg| emit_bvexpr(arg, ctx));
    match id.as_str() {
        "sext" => {
            let arg = args.next().unwrap_or_else(too_few);
            ctx.insert_bv(BitVecExpr::SExt(w, arg))
        }
        "zext" => {
            let arg = args.next().unwrap_or_else(too_few);
            ctx.insert_bv(BitVecExpr::SExt(w, arg))
        }
        unr => {
            println!("Reached undefined function call {:#?}", unr);
            unreachable!()
        }
    }
}

fn emit_bvcat(rbvcat: Pair<'_, Rule>, ctx: &mut Ctx, w: WidthExprId) -> BitVecExprId {
    let mut inner = rbvcat.into_inner();
    let lexpr = inner.next().unwrap();
    let rexpr = inner.next().unwrap();
    let expr = BitVecExpr::Cat(w, emit_bvexpr(lexpr, ctx), emit_bvexpr(rexpr, ctx));
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
        emit_bvexpr(rexpr, ctx),
    );
    ctx.insert_bv(expr)
}

fn emit_bvary(rbvary: Pair<'_, Rule>, ctx: &mut Ctx, w: WidthExprId) -> BitVecExprId {
    let mut inner = rbvary.into_inner();
    let lexpr = inner.next().unwrap();
    let tail = inner.next().unwrap().into_inner().next().unwrap();
    let expr = match tail.as_rule() {
        Rule::binary_tail => {
            let mut inner = tail.into_inner();
            let binop = inner.next().unwrap();
            let rexpr = inner.next().unwrap();
            BitVecExpr::BinArith(
                w,
                emit_bvbinop(binop),
                emit_bvexpr(lexpr, ctx),
                emit_bvexpr(rexpr, ctx),
            )
        }
        Rule::ternary_tail => {
            let mut inner = tail.into_inner();
            let mexpr = inner.next().unwrap();
            let rexpr = inner.next().unwrap();
            BitVecExpr::Ite(
                w,
                emit_bvexpr(lexpr, ctx),
                emit_bvexpr(mexpr, ctx),
                emit_bvexpr(rexpr, ctx),
            )
        }
        unr => {
            println!("Reached unreachable {:#?}", unr);
            unreachable!()
        }
    };
    ctx.insert_bv(expr)
}

fn emit_bvbinop(rbvbinop: Pair<'_, Rule>) -> BitVecBinArithOp {
    match rbvbinop.as_rule() {
        Rule::add_op => BitVecBinArithOp::Add,
        _ => unimplemented!(),
    }
}
