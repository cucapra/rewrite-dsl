use std::env;

use rewrite_dsl::{ir::Ctx, parser, printer::PrettyPrintable};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut ctx = Ctx::default();
    let prog = parser::parse_rw_file(&args[1], &mut ctx);
    println!("{}", prog.pretty_print(&ctx))
}
