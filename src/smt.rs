use std::{collections::HashMap, io::Result};

use easy_smt::{ContextBuilder, Response, SExpr};

use crate::ir::{
    BitVecArithOp, BitVecCmpOp, BitVecExpr, BitVecExprId, Ctx, RwRule, WidthAssignment,
};

// Extension logics.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum Ext {
    // Zero-extension: extend by prepending zeros.
    ZeroExt,
    // Sign-extension: extend by replicating the most significant bit.
    SignExt,
}

/// Builds `((_ zero/sign_extend n) x)`, or leaves unchanged if `n == 0`.
fn smt_extend(smt: &easy_smt::Context, x: SExpr, n: u32, ext: Ext) -> SExpr {
    if n == 0 {
        return x;
    }
    smt.list(vec![
        smt.list(vec![
            smt.atom("_"),
            smt.atom(match ext {
                Ext::SignExt => "sign_extend",
                Ext::ZeroExt => "zero_extend",
            }),
            smt.numeral(n),
        ]),
        x,
    ])
}

impl Ctx {
    /// Lowers a [`BitVecExpr`] to an SMT-LIB bit-vector term.
    fn to_smt(
        &self,
        smt: &mut easy_smt::Context,
        expr: BitVecExprId,
        assgn: &WidthAssignment,
        cache: &mut HashMap<BitVecExprId, SExpr>,
    ) -> std::io::Result<SExpr> {
        if let Some(s) = cache.get(&expr) {
            return Ok(s.clone());
        }
        let s = match &self[expr] {
            BitVecExpr::Var(w, name) => {
                let bv = smt.bit_vec_sort(smt.numeral(self.eval_width(*w, assgn)));
                smt.declare_const(name, bv)?
            }
            BitVecExpr::Const(w, v) => 
                smt.binary(self.eval_width(*w, assgn) as usize, *v as i64),
            BitVecExpr::Arith(w, op, a, b) => {
                let out_w = self.eval_width(*w, assgn);
                let wa = self.eval_width(self.width(*a), assgn);
                let wb = self.eval_width(self.width(*b), assgn);
                assert!(
                    wa <= out_w && wb <= out_w,
                    "operand wider than output; truncation unsupported"
                );
                let sa = self.to_smt(smt, *a, assgn, cache)?;
                let sb = self.to_smt(smt, *b, assgn, cache)?;
                // let sa = smt_extend(smt, sa, out_w - wa);
                // let sb = smt_extend(smt, sb, out_w - wb);
                    match op {
                        BitVecArithOp::And => smt.bvand(sa, sb),
                        BitVecArithOp::Add => smt.bvadd(sa, sb),
                        BitVecArithOp::Or => smt.bvor(sa, sb),
                    }
            }
            BitVecExpr::Cmp(op, a, b) => {
                let sa = self.to_smt(smt, *a, assgn, cache)?;
                let sb = self.to_smt(smt, *b, assgn, cache)?;
                let cond = match op {
                    BitVecCmpOp::Gt => smt.bvugt(sa, sb),
                    BitVecCmpOp::Eq => smt.eq(sa, sb),
                };
                smt.ite(cond, smt.binary(1, 1), smt.binary(1, 0))
            }
            BitVecExpr::Cat(_, a, b) => {
                let sa = self.to_smt(smt, *a, assgn, cache)?;
                let sb = self.to_smt(smt, *b, assgn, cache)?;
                smt.concat(sa, sb)
            }
            BitVecExpr::Ite(_, c, t, e) => {
                let sc = self.to_smt(smt, *c, assgn, cache)?;
                let st = self.to_smt(smt, *t, assgn, cache)?;
                let se = self.to_smt(smt, *e, assgn, cache)?;
                smt.ite(smt.eq(sc, smt.binary(1, 1)), st, se)
            }
            BitVecExpr::SExt(w, a) => {
                let sa = self.to_smt(smt, *a, assgn, cache)?;
                smt_extend(smt, sa, self.eval_width(*w, assgn), Ext::SignExt)
            }
            BitVecExpr::ZExt(w, a) => {
                let sa = self.to_smt(smt, *a, assgn, cache)?;
                smt_extend(smt, sa, self.eval_width(*w, assgn), Ext::ZeroExt)
            }
        };
        cache.insert(expr, s);
        Ok(s)
    }

    /// Checks whether `rule.lhs` and `rule.rhs` are equivalent at a
    /// concrete width assignment, using Z3.
    pub fn check_rule(
        &self,
        rule: &RwRule,
        assign: &WidthAssignment,
    ) -> Result<Response> {
        let mut smt = ContextBuilder::new().with_z3_defaults().build()?;
        let mut cache = HashMap::new();
        let lhs = self.to_smt(&mut smt, rule.lhs, assign, &mut cache)?;
        let rhs = self.to_smt(&mut smt, rule.rhs, assign, &mut cache)?;
        smt.assert(smt.not(smt.eq(lhs, rhs)))?;
        let rslt = smt.check();
        rslt
    }
}
