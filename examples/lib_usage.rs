// Exemplo de uso da lib `hako` (transpilação programática).
use std::path::{Path, PathBuf};

fn main() {
    let src = r#"
box serial {
    COM1 = 0x3F8
    config => default
    write_byte(b) => default
}
"#;
    let (code, boxes, flows) = hako::transpile_string(src).expect("parse");
    println!("{} boxes, {} flows", boxes, flows);
    println!("{}", code);

    // E via arquivo, como faz o binário `hako`.
    let out = PathBuf::from("output.rs");
    let (boxes, impls) = hako::transpile_file(Path::new("examples/collatz.hako"), &out).expect("file");
    println!("file: {} boxes, {} impls -> {}", boxes, impls, out.display());
}
