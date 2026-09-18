/// Hako — CLI tool for transpiling Hako language to Rust
use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: hako <input.hako> [-o output.rs] [--check]");
        process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let check_only = args.iter().any(|a| a == "--check");
    let output_path = if args.len() >= 4 && args[2] == "-o" {
        PathBuf::from(&args[3])
    } else {
        let mut out = input_path.clone();
        out.set_extension("rs");
        out
    };

    match hako::transpile_file(&input_path, &output_path) {
        Ok((boxes, impls)) => {
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
