# Hako — TODO

DSL de boxes/flows otimizada pra LLM. Transpila pra Rust puro.

## ✅ Concluído

- Transpilador `hako input.hako -o output.rs` + `--check`
- Boxes, flows, constantes injetadas, auto mode, block mode, raw mode
- Stdlib: serial/UART/SPI/I2C/GPIO/VGA, Port I/O (`outb`, `inb`, `outw`, `inw`), timer, keyboard
- Macros GPIO, 81 testes, CI no GitHub Actions
- `cargo clippy` 0 warnings, dual license MIT/Apache-2.0
- `if/else` sem `=>` como forma canônica
- `inb` com **retorno** (`port_inb` propaga o valor, sem `;` no auto mode)
- `outb(value, port)` documentado no `SYNTAX.md` (valor primeiro, porta depois)

## 🔴 Alta prioridade

- Tipos e inferência: Rust tem `u8`, `&str`, `fn()`; Hako infere por nome (`s→&str`, `buf→&[u32]`, resto `u32` batendo com a stdlib). Tipagem `u8` de verdade exige migrar a stdlib `port_inb` de `u32` para `u8` (breaking)
- `flow` gerando `run()` — hoje `flow boot { serial vga }` vira `flow_boot()`, mas o Mizu usa `flow default { process }` → `flow_default()`. Documentar a convenção

## 🟡 Média prioridade

- Borrow checker / segurança de memória (Rust ✅, Hako ❌ — ver `language_idea.md`)
- Traits/genéricos (Rust ✅, Hako ❌)
- Macros (Rust ✅, Hako ❌, mas `raw` cobre alguns casos)
- Ecossistema (crates.io massivo no Rust, nada no Hako ainda — `cargo install hako` é só o transpilador)
- Ferramentas (`cargo`, `rustfmt` no Rust, nada no Hako ainda — formatar, `cargo test` de saída gerada)
- Erros de parsing com span útil (hoje só mensagem curta)
- `outw`/`inw` e `pit_config`/`timer` sem exemplos no `SYNTAX.md`

## 🟢 Baixa prioridade

- Editor com visualização de caixas (emoji) mantendo fonte ASCII (`box`/`inst`)
- Propagar `Hako` como crate de lib (`lib.rs` existe, mas exemplos tiram bem mais dela)
- Suporte a `in`/`for in` mais completo (hoje mínimo)
- `loop` com condição e `break` documentados no `SYNTAX.md` atual
