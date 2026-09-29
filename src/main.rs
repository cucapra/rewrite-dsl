use std::env;

use rewrite_dsl::{ir::Ctx, parser};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut ctx = Ctx::default();
    let _ = parser::parse_rw_file(&args[1], &mut ctx);
}
