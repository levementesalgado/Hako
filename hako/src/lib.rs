pub mod ast;
pub mod parser;
pub mod codegen;
pub mod stdlib;

use std::fs;
use std::path::Path;

pub fn transpile_string(input: &str) -> Result<(String, usize, usize), String> {
    let mut p = parser::Parser::new(input);
    let program = p.parse().map_err(|e| format!("parse error: {}", e))?;
    let mut cg = codegen::Codegen::new();
    let output = cg.generate_full(&program);
    Ok((output, program.boxes.len(), program.flows.len()))
}

pub fn transpile_file(input_path: &Path, output_path: &Path) -> Result<(usize, usize), String> {
    let input = fs::read_to_string(input_path)
        .map_err(|e| format!("could not read {}: {}", input_path.display(), e))?;
    let (code, boxes, impls) = transpile_string(&input)?;
    fs::write(output_path, &code)
        .map_err(|e| format!("could not write {}: {}", output_path.display(), e))?;
    Ok((boxes, impls))
}
