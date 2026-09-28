use std::env;

use rewrite_dsl::parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    let _ = parser::parse_rw_file(&args[1]);
}
