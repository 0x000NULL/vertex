pub mod codegen;
pub mod error;
pub mod explain;
pub mod lexer;
pub mod mir;
pub mod parser;
pub mod resolve;
pub mod span;
pub mod typecheck;
pub mod util;

pub fn run() {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--explain" {
            let code = match args.next() {
                Some(c) => c,
                None => {
                    eprintln!("--explain requires an error code argument");
                    std::process::exit(1);
                }
            };
            dispatch_explain(&code);
            return;
        }
        if let Some(code) = arg.strip_prefix("--explain=") {
            dispatch_explain(code);
            return;
        }
    }
}

fn dispatch_explain(code: &str) {
    match explain::explain(code) {
        Some(text) => println!("{}", text),
        None => {
            eprintln!("unknown error code: {}", code);
            std::process::exit(1);
        }
    }
}
