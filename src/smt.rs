use std::{collections::HashMap, io::Result};

use easy_smt::{ContextBuilder, Response, SExpr};

use crate::ir::{
    BitVecBinArithOp, BitVecCmpOp, BitVecExpr, BitVecExprId, BitVecUnArithOp, Ctx, RwRule,
    WidthAssignment,
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
        vars: &mut HashMap<String, SExpr>,
    ) -> std::io::Result<SExpr> {
        if let Some(s) = cache.get(&expr) {
            return Ok(*s);
        }
        let s = match &self[expr] {
            BitVecExpr::Var(w, name) => {
                if let Some(s) = vars.get(name) {
                    *s
                } else {
                    let bv = smt.bit_vec_sort(smt.numeral(self.eval_width(*w, assgn)));
                    let var = smt.declare_const(name, bv)?;
                    vars.insert(name.clone(), smt.atom(name));
                    var
                }
            }
            BitVecExpr::Const(w, v) => smt.binary(self.eval_width(*w, assgn) as usize, *v as i64),
            BitVecExpr::BinArith(w, op, a, b) => {
                let out_w = self.eval_width(*w, assgn);
                let wa = self.eval_width(self.width(*a), assgn);
                let wb = self.eval_width(self.width(*b), assgn);
                assert!(
                    wa <= out_w && wb <= out_w,
                    "operand wider than output; truncation unsupported"
                );
                let sa = self.to_smt(smt, *a, assgn, cache, vars)?;
                let sb = self.to_smt(smt, *b, assgn, cache, vars)?;
                match op {
                    BitVecBinArithOp::And => smt.bvand(sa, sb),
                    BitVecBinArithOp::Add => smt.bvadd(sa, sb),
                    BitVecBinArithOp::Or => smt.bvor(sa, sb),
                }
            }
            BitVecExpr::Cmp(op, a, b) => {
                let sa = self.to_smt(smt, *a, assgn, cache, vars)?;
                let sb = self.to_smt(smt, *b, assgn, cache, vars)?;
                let cond = match op {
                    BitVecCmpOp::Gt => smt.bvugt(sa, sb),
                    BitVecCmpOp::Eq => smt.eq(sa, sb),
                };
                smt.ite(cond, smt.binary(1, 1), smt.binary(1, 0))
            }
            BitVecExpr::Cat(_, a, b) => {
                let sa = self.to_smt(smt, *a, assgn, cache, vars)?;
                let sb = self.to_smt(smt, *b, assgn, cache, vars)?;
                smt.concat(sa, sb)
            }
            BitVecExpr::Ite(_, c, t, e) => {
                let sc = self.to_smt(smt, *c, assgn, cache, vars)?;
                let st = self.to_smt(smt, *t, assgn, cache, vars)?;
                let se = self.to_smt(smt, *e, assgn, cache, vars)?;
                smt.ite(smt.eq(sc, smt.binary(1, 1)), st, se)
            }
            BitVecExpr::SExt(w, a) => {
                let sa = self.to_smt(smt, *a, assgn, cache, vars)?;
                smt_extend(smt, sa, self.eval_width(*w, assgn), Ext::SignExt)
            }
            BitVecExpr::ZExt(w, a) => {
                let sa = self.to_smt(smt, *a, assgn, cache, vars)?;
                smt_extend(smt, sa, self.eval_width(*w, assgn), Ext::ZeroExt)
            }
            BitVecExpr::UnArith(w, op, a) => {
                let out_w = self.eval_width(*w, assgn);
                let wa = self.eval_width(self.width(*a), assgn);
                assert!(
                    wa <= out_w,
                    "operand wider than output; truncation unsupported"
                );
                let sa = self.to_smt(smt, *a, assgn, cache, vars)?;
                match op {
                    BitVecUnArithOp::Neg => smt.bvneg(sa),
                    BitVecUnArithOp::Not => smt.bvnot(sa),
                }
            }
        };
        cache.insert(expr, s);
        Ok(s)
    }

    /// Checks whether `rule.lhs` and `rule.rhs` are equivalent at a
    /// concrete width assignment, using Z3.
    pub fn check_rule(&self, rule: &RwRule, assign: &WidthAssignment) -> Result<Response> {
        let mut smt = ContextBuilder::new()
            .with_z3_defaults()
            // .replay_file(Some(std::fs::File::create("log?")?))
            .build()?;
        let mut cache = HashMap::new();
        let mut vars = HashMap::new();
        let lhs = self.to_smt(&mut smt, rule.lhs, assign, &mut cache, &mut vars)?;
        let rhs = self.to_smt(&mut smt, rule.rhs, assign, &mut cache, &mut vars)?;
        smt.assert(smt.not(smt.eq(lhs, rhs)))?;

        smt.check()
    }
}
