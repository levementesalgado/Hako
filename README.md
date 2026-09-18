# Hako

Linguagem de programação transpilada para Rust, projetada para programação de baixo nível e sistemas embarcados.

## Estrutura

```
hako/                     → transpilador Hako → Rust
  src/
    ast.rs                → tipos da AST
    parser.rs             → parser recursivo descendente
    codegen.rs            → gerador de código Rust
    stdlib.rs             → stdlib predefinida (serial, VGA, PIT, teclado, port I/O)
    lib.rs                → API pública (transpile_file)
    main.rs               → CLI
  examples/               → exemplos de código Hako
collatz-analyzer/         → analisador de ciclos Collatz (teoria da linguagem)
DOCUMENTACAO.md           → documentação completa da linguagem
language_idea.md          → ideia e conceitos da linguagem
objects.md                → sistema de objetos
```

## Build

```bash
# Transpilar arquivo Hako para Rust
cargo run -p hako -- input.hako -o output.rs

# Compilar o resultado
rustc output.rs -o output
```

## Exemplos

```bash
# Rodar exemplo
cargo run -p hako -- examples/hello.hako -o /tmp/hello.rs
rustc /tmp/hello.rs -o /tmp/hello
/tmp/hello
```

## Documentação

- [DOCUMENTACAO.md](DOCUMENTACAO.md) — Documentação completa da linguagem
- [language_idea.md](language_idea.md) — Ideia e conceitos
- [objects.md](objects.md) — Sistema de objetos
- [planning.md](planning.md) — Planejamento do projeto

## Licença

Dual-licenciado sob as licenças [MIT](LICENSE-MIT) e [Apache 2.0](LICENSE).

Qualquer pessoa pode usar, modificar e distribuir este código sob qualquer uma das duas licenças.
