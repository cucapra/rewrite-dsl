use std::{collections::HashMap, io::Result};

use easy_smt::{ContextBuilder, Response, SExpr};

use crate::ir::{
    BitVecArithOp, BitVecCmpOp, BitVecExpr, BitVecExprId, Ctx, Ext, RwRule, WidthAssignment,
};

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
        assign: &WidthAssignment,
        cache: &mut HashMap<BitVecExprId, (SExpr, Ext)>,
    ) -> std::io::Result<(SExpr, Ext)> {
        if let Some(s) = cache.get(&expr) {
            return Ok(s.clone());
        }
        let (s, ext) = match &self[expr] {
            BitVecExpr::Var(w, name, ext) => {
                let bv = smt.bit_vec_sort(smt.numeral(self.eval_width(*w, assign)));
                (smt.declare_const(name, bv)?, ext)
            }
            BitVecExpr::Const(w, v, ext) => (
                smt.binary(self.eval_width(*w, assign) as usize, *v as i64),
                ext,
            ),
            BitVecExpr::Arith(w, op, a, b, ext) => {
                let out_w = self.eval_width(*w, assign);
                let wa = self.eval_width(self.width(*a), assign);
                let wb = self.eval_width(self.width(*b), assign);
                assert!(
                    wa <= out_w && wb <= out_w,
                    "operand wider than output; truncation unsupported"
                );
                let (sa, ea) = self.to_smt(smt, *a, assign, cache)?;
                let (sb, eb) = self.to_smt(smt, *b, assign, cache)?;
                let sa = smt_extend(smt, sa, out_w - wa, ea);
                let sb = smt_extend(smt, sb, out_w - wb, eb);
                (
                    match op {
                        BitVecArithOp::And => smt.bvand(sa, sb),
                        BitVecArithOp::Add => smt.bvadd(sa, sb),
                        BitVecArithOp::Or => smt.bvor(sa, sb),
                    },
                    ext,
                )
            }
            BitVecExpr::Cmp(op, a, b, ext) => {
                let (sa, _) = self.to_smt(smt, *a, assign, cache)?;
                let (sb, _) = self.to_smt(smt, *b, assign, cache)?;
                let cond = match op {
                    BitVecCmpOp::Gt => smt.bvugt(sa, sb),
                    BitVecCmpOp::Eq => smt.eq(sa, sb),
                };
                (smt.ite(cond, smt.binary(1, 1), smt.binary(1, 0)), ext)
            }
            BitVecExpr::Cat(_, a, b, ext) => {
                let (sa, _) = self.to_smt(smt, *a, assign, cache)?;
                let (sb, _) = self.to_smt(smt, *b, assign, cache)?;
                (smt.concat(sa, sb), ext)
            }
            BitVecExpr::Ite(_, c, t, e, ext) => {
                let (sc, _) = self.to_smt(smt, *c, assign, cache)?;
                let (st, _) = self.to_smt(smt, *t, assign, cache)?;
                let (se, _) = self.to_smt(smt, *e, assign, cache)?;
                (smt.ite(smt.eq(sc, smt.binary(1, 1)), st, se), ext)
            }
        };
        cache.insert(expr, (s, ext.clone()));
        Ok((s, ext.clone()))
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
        let (lhs, _) = self.to_smt(&mut smt, rule.lhs, assign, &mut cache)?;
        let (rhs, _) = self.to_smt(&mut smt, rule.rhs, assign, &mut cache)?;
        smt.assert(smt.not(smt.eq(lhs, rhs)))?;
        let rslt = smt.check();
        rslt
    }
}
