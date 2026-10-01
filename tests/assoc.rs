use easy_smt::*;
use rewrite_dsl::ir::*;
use std::borrow::Cow;

/// Builds associativity-of-addition rule.
fn build_assoc_add_diff_widths(ctx: &mut Ctx) -> RwRule {
    let wa = ctx.width_var(Cow::Owned("wa".to_string()));
    let wb = ctx.width_var(Cow::Owned("wb".to_string()));
    let wc = ctx.width_var(Cow::Owned("wc".to_string()));
    let w_bc = ctx.width_var(Cow::Owned("w_bc".to_string()));
    let w_ab = ctx.width_var(Cow::Owned("w_ab".to_string()));
    let w_out = ctx.width_var(Cow::Owned("w_out".to_string()));

    let a = ctx.insert_bv(BitVecExpr::Var(wa, "a".to_string()));
    let b = ctx.insert_bv(BitVecExpr::Var(wb, "b".to_string()));
    let c = ctx.insert_bv(BitVecExpr::Var(wc, "c".to_string()));

    // lhs: a + (b + c)
    let bc = ctx.insert_bv(BitVecExpr::BinArith(w_bc, BitVecBinArithOp::Add, b, c));
    let lhs = ctx.insert_bv(BitVecExpr::BinArith(w_out, BitVecBinArithOp::Add, a, bc));

    // rhs: (a + b) + c
    let ab = ctx.insert_bv(BitVecExpr::BinArith(w_ab, BitVecBinArithOp::Add, a, b));
    let rhs = ctx.insert_bv(BitVecExpr::BinArith(w_out, BitVecBinArithOp::Add, ab, c));

    RwRule {
        name: "assoc_add_diff_widths".to_string(),
        width_vars: vec![
            "wa".into(),
            "wb".into(),
            "wc".into(),
            "w_bc".into(),
            "w_ab".into(),
            "w_out".into(),
        ],
        lhs,
        rhs,
    }
}

#[test]
fn assoc_add_correct_widths() -> std::io::Result<()> {
    let mut ctx = Ctx::default();
    let rule = build_assoc_add_diff_widths(&mut ctx);

    // wa=4, wb=6, wc=3
    let assign = WidthAssignment::from([
        ("wa".to_string(), 4),
        ("wb".to_string(), 6),
        ("wc".to_string(), 3),
        ("w_bc".to_string(), 7),  // max(6, 3) + 1
        ("w_ab".to_string(), 7),  // max(4, 6) + 1
        ("w_out".to_string(), 8), // max(4, 7) + 1 == max(7, 3) + 1
    ]);
    assert_eq!(ctx.check_rule(&rule, &assign)?, Response::Unsat);
    Ok(())
}

#[test]
#[ignore] // TODO: fix test
fn assoc_add_undersized_intermediate_fails() -> std::io::Result<()> {
    let mut ctx = Ctx::default();
    let rule = build_assoc_add_diff_widths(&mut ctx);

    // w_bc is too small to hold b + c
    let assign = WidthAssignment::from([
        ("wa".to_string(), 4),
        ("wb".to_string(), 6),
        ("wc".to_string(), 3),
        ("w_bc".to_string(), 6),
        ("w_ab".to_string(), 7),
        ("w_out".to_string(), 8),
    ]);
    assert_eq!(ctx.check_rule(&rule, &assign)?, Response::Sat);
    Ok(())
}
