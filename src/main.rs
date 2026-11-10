/// Hako — CLI tool for transpiling Hako language to Rust
use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: hako <input.hako> [-o output.rs] [--check] [--fmt]");
        eprintln!("       hako view <input.hako>");
        process::exit(1);
    }

    if args[1] == "view" {
        if args.len() < 3 {
            eprintln!("Usage: hako view <input.hako>");
            process::exit(1);
        }
        let path = PathBuf::from(&args[2]);
        let src = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            process::exit(1)
        });
        let mut p = hako::parser::Parser::new(&src);
        let prog = p.parse().unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            process::exit(1)
        });
        for b in &prog.boxes {
            println!("box {}", b.name);
            for it in &b.items {
                match it {
                    hako::ast::Item::Fn { name, .. } => println!("  - {}", name),
                    hako::ast::Item::Var { name, .. } => println!("  = {}", name),
                    hako::ast::Item::Expr(e) => println!("  : {}", e),
                }
            }
        }
        for f in &prog.flows {
            println!("flow {}", f.name);
            for s in &f.steps {
                println!("  -> {}", s);
            }
        }
        return;
    }

    let input_path = PathBuf::from(&args[1]);
    let check_only = args.iter().any(|a| a == "--check");
    let fmt = args.iter().any(|a| a == "--fmt");
    let output_path = if args.len() >= 4 && args[2] == "-o" {
        PathBuf::from(&args[3])
    } else {
        let mut out = input_path.clone();
        out.set_extension("rs");
        out
    };

    match hako::transpile_file(&input_path, &output_path) {
        Ok((boxes, impls)) => {
            if fmt {
                let _ = std::process::Command::new("rustfmt").arg(&output_path).status();
            }
            if check_only {
                eprintln!("hako: OK ({} boxes, {} implementations)", boxes, impls);
            } else {
                eprintln!("hako: wrote {}", output_path.display());
                eprintln!("hako: {} boxes, {} implementations", boxes, impls);
            }
        }
        Err(e) => {
            eprintln!("error: {}", e);
            process::exit(1);
        }
    }
}
