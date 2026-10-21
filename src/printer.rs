use crate::ir::{
    BitVecBinArithOp, BitVecCmpOp, BitVecExpr, BitVecUnArithOp, BoolExpr, Ctx, RwProg, RwRule,
    WidthExpr,
};

pub trait PrettyPrintable {
    fn pretty_print(&self, ctx: &Ctx) -> String;
}

impl PrettyPrintable for WidthExpr {
    fn pretty_print(&self, ctx: &Ctx) -> String {
        match self {
            WidthExpr::Var(v) => v.to_string(),
            WidthExpr::Const(c) => c.to_string(),
            WidthExpr::BinOp(op, lexpr, rexpr) => {
                let lw = ctx.get_width(*lexpr).unwrap();
                let rw = ctx.get_width(*rexpr).unwrap();
                format!(
                    "({} {} {})",
                    lw.pretty_print(ctx),
                    match op {
                        crate::ir::WidthBinOp::Add => "+",
                        crate::ir::WidthBinOp::Sub => "-",
                    },
                    rw.pretty_print(ctx)
                )
            }
            WidthExpr::Ite(bexpr, lexpr, rexpr) => {
                let b = ctx.get_bool(*bexpr).unwrap();
                let lw = ctx.get_width(*lexpr).unwrap();
                let rw = ctx.get_width(*rexpr).unwrap();
                format!(
                    "({} ? {} : {})",
                    b.pretty_print(ctx),
                    lw.pretty_print(ctx),
                    rw.pretty_print(ctx)
                )
            }
        }
    }
}

impl PrettyPrintable for BoolExpr {
    fn pretty_print(&self, ctx: &Ctx) -> String {
        match self {
            BoolExpr::Const(b) => b.to_string(),
            BoolExpr::Not(bexpr) => {
                let b = ctx.get_bool(*bexpr).unwrap();
                format!("(!{})", b.pretty_print(ctx))
            }
            BoolExpr::Or(lbexpr, rbexpr) => {
                let lb = ctx.get_bool(*lbexpr).unwrap();
                let rb = ctx.get_bool(*rbexpr).unwrap();
                format!("({} || {})", lb.pretty_print(ctx), rb.pretty_print(ctx))
            }
            BoolExpr::And(lbexpr, rbexpr) => {
                let lb = ctx.get_bool(*lbexpr).unwrap();
                let rb = ctx.get_bool(*rbexpr).unwrap();
                format!("({} && {})", lb.pretty_print(ctx), rb.pretty_print(ctx))
            }
            BoolExpr::Gt(lwexpr, rwexpr) => {
                let lw = ctx.get_width(*lwexpr).unwrap();
                let rw = ctx.get_width(*rwexpr).unwrap();
                format!("({} > {})", lw.pretty_print(ctx), rw.pretty_print(ctx))
            }
            BoolExpr::Eq(lwexpr, rwexpr) => {
                let lw = ctx.get_width(*lwexpr).unwrap();
                let rw = ctx.get_width(*rwexpr).unwrap();
                format!("({} == {})", lw.pretty_print(ctx), rw.pretty_print(ctx))
            }
        }
    }
}

impl PrettyPrintable for BitVecExpr {
    fn pretty_print(&self, ctx: &Ctx) -> String {
        match self {
            BitVecExpr::Var(wexpr, v) => {
                let w = ctx.get_width(*wexpr).unwrap();
                format!("{}:{}", v, w.pretty_print(ctx))
            }
            BitVecExpr::Const(wexpr, c) => {
                let w = ctx.get_width(*wexpr).unwrap();
                format!("{}:{}", c.to_string(), w.pretty_print(ctx))
            }
            BitVecExpr::UnArith(wexpr, op, expr) => {
                let w = ctx.get_width(*wexpr).unwrap();
                let e = ctx.get_bv(*expr).unwrap();
                format!(
                    "({}{}):{}",
                    match op {
                        BitVecUnArithOp::Not => "!",
                        BitVecUnArithOp::Neg => "-",
                    },
                    e.pretty_print(ctx),
                    w.pretty_print(ctx)
                )
            }
            BitVecExpr::BinArith(wexpr, op, lexpr, rexpr) => {
                let w = ctx.get_width(*wexpr).unwrap();
                let l = ctx.get_bv(*lexpr).unwrap();
                let r = ctx.get_bv(*rexpr).unwrap();
                format!(
                    "({} {} {}):{}",
                    l.pretty_print(ctx),
                    match op {
                        BitVecBinArithOp::And => "&",
                        BitVecBinArithOp::Add => "+",
                        BitVecBinArithOp::Or => "|",
                    },
                    r.pretty_print(ctx),
                    w.pretty_print(ctx)
                )
            }
            BitVecExpr::Cmp(op, lexpr, rexpr) => {
                let w = 1.to_string();
                let l = ctx.get_bv(*lexpr).unwrap();
                let r = ctx.get_bv(*rexpr).unwrap();
                format!(
                    "({} {} {}):{}",
                    l.pretty_print(ctx),
                    match op {
                        BitVecCmpOp::Gt => ">",
                        BitVecCmpOp::Eq => "==",
                    },
                    r.pretty_print(ctx),
                    w
                )
            }
            BitVecExpr::Cat(wexpr, lexpr, rexpr) => {
                let w = ctx.get_width(*wexpr).unwrap();
                let l = ctx.get_bv(*lexpr).unwrap();
                let r = ctx.get_bv(*rexpr).unwrap();
                format!(
                    "{{{}, {}}}:{}",
                    l.pretty_print(ctx),
                    r.pretty_print(ctx),
                    w.pretty_print(ctx)
                )
            }
            BitVecExpr::Ite(wexpr, lexpr, mexpr, rexpr) => {
                let w = ctx.get_width(*wexpr).unwrap();
                let l = ctx.get_bv(*lexpr).unwrap();
                let m = ctx.get_bv(*mexpr).unwrap();
                let r = ctx.get_bv(*rexpr).unwrap();
                format!(
                    "(({} ? {} : {}):{})",
                    l.pretty_print(ctx),
                    m.pretty_print(ctx),
                    r.pretty_print(ctx),
                    w.pretty_print(ctx)
                )
            }
            BitVecExpr::SExt(wexpr, expr) => {
                let w = ctx.get_width(*wexpr).unwrap();
                let e = ctx.get_bv(*expr).unwrap();
                format!("sext({}):{}", e.pretty_print(ctx), w.pretty_print(ctx))
            }
            BitVecExpr::ZExt(wexpr, expr) => {
                let w = ctx.get_width(*wexpr).unwrap();
                let e = ctx.get_bv(*expr).unwrap();
                format!("zext({}):{}", e.pretty_print(ctx), w.pretty_print(ctx))
            }
        }
    }
}

impl PrettyPrintable for RwRule {
    fn pretty_print(&self, ctx: &Ctx) -> String {
        let nlindent = "\n    ";
        let l = ctx.get_bv(self.lhs).unwrap();
        let r = ctx.get_bv(self.rhs).unwrap();
        format!(
            "rule {}({}) {{{nlindent}{}{nlindent}=>{nlindent}{}\n}}",
            self.name,
            self.width_vars.join(", "),
            l.pretty_print(ctx),
            r.pretty_print(ctx)
        )
    }
}

impl PrettyPrintable for RwProg {
    fn pretty_print(&self, ctx: &Ctx) -> String {
        self.iter()
            .map(|r| r.pretty_print(ctx))
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}
