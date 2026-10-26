use easy_smt::Response::{Sat, Unsat};
use rewrite_dsl::{
    ir::{Ctx, WidthAssignment},
    parser,
    printer::PrettyPrintable,
};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Subcommand)]
enum Mode {
    /// Type-check given a range of widths, both ends included.
    Check {
        /// Minimum width.
        #[arg(short, long)]
        from: u32,
        /// Maximum width
        #[arg(short, long)]
        to: u32,
    },
    Print,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Mode.
    #[command(subcommand)]
    mode: Mode,
    #[arg(value_name = "FILE")]
    /// Input file to parse.
    file: PathBuf,
}

fn main() {
    let args = Args::parse();

    let mut ctx = Ctx::default();
    let prog = parser::parse_rw_file(&args.file, &mut ctx);

    match args.mode {
        Mode::Check { from, to } => prog.iter().for_each(|r| {
            let a = WidthAssignment::from_range(&r.width_vars, from, to + 1);
            a.iter()
                .map(|a| (a, ctx.check_rule(r, a)))
                .filter_map(|res| match res {
                    (_, Ok(Unsat)) => None,
                    (a, Ok(Sat)) => {
                        Some(format!("Rule is unsound for width assignment: {}", a).to_string())
                    }
                    (_, e) => Some(format!("Error: {:?}", e)),
                })
                .for_each(|r| println!("{}", r));
        }),
        Mode::Print => {
            println!("{}", prog.pretty_print(&ctx))
        }
    }
}
