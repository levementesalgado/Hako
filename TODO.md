# Hako — TODO

DSL de boxes/flows otimizada pra LLM. Transpila pra Rust puro.

## ✅ Concluído

- Transpilador `hako input.hako -o output.rs` + `--check`
- Boxes, flows, constantes injetadas, auto mode, block mode, raw mode
- Stdlib: serial/UART/SPI/I2C/GPIO/VGA, Port I/O (`outb`, `inb`, `outw`, `inw`), timer, keyboard
- Macros GPIO, 81 testes, CI no GitHub Actions
- `cargo clippy` 0 warnings, dual license MIT/Apache-2.0
- `if/else` sem `=>` como forma canônica

## 🔴 Alta prioridade

- `inb(port) -> u8` com **retorno** — hoje o Hako não devolve valores, só chama `port_inb` e descarta. Trava driver de teclado (Mizu `kbd.hako` usa wrapper)
- `outb(value, port)` — ordem de argumentos já definida, mas falta validação e invariante documentado no `SYNTAX.md`
- Tipos e inferência: Rust tem `u8`, `&str`, `fn()`; Hako ainda infere por nome. Sem isso não dá pra validar `inb` retornando `u8` vs `u32`
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
