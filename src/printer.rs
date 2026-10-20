use crate::ir::{BoolExpr, Ctx, WidthExpr};

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
                    "{} {} {}",
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
                    "{} ? {} : {}",
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
        todo!()
    }
}
