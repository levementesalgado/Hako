# Hako — TODO

DSL de boxes/flows otimizada pra LLM. Transpila pra Rust puro.

## ✅ Concluído

- Transpilador `hako input.hako -o output.rs` + `--check`
- Boxes, flows, constantes injetadas, auto mode, block mode, raw mode
- Stdlib: serial/UART/SPI/I2C/GPIO/VGA, Port I/O (`outb`, `inb`, `outw`, `inw`), timer, keyboard
- Macros GPIO, 84 testes, CI no GitHub Actions
- `cargo clippy` 0 warnings, dual license MIT/Apache-2.0
- `if/else` sem `=>` como forma canônica
- `inb` com **retorno** (`port_inb` propaga o valor, sem `;` no auto mode)
- `outb(value, port)` documentado no `SYNTAX.md` (valor primeiro, porta depois)
- `flow { box }` → `box::run()` com convenção `flow default → flow_default()` no `SYNTAX.md`
- `outw`/`inw` e `pit_config`/`timer` com exemplos no `SYNTAX.md`
- Exemplo de uso da lib (`examples/lib_usage.rs`, `transpile_string`/`transpile_file`)
- Teste de `for`/`in` e de erro com linha/coluna nas `parser_tests`
- `hako --fmt` (roda `rustfmt` na saída gerada)
- `hako view` (visualização ASCII de boxes/flows, fonte ASCII preservada)
- Exemplos `examples/*.hako` reescritos na sintaxe atual (todos parseiam via `hako view`)
- `port_inb -> u8` e `port_inw -> u16` na stdlib (tipagem correta do `in al/ax`)
- Borrow checker / traits / macros: herdados do Rust gerado (o transpiler só emite Rust seguro; `raw` usa `asm!` com `unsafe` explícito)
- Ecossistema: crate `hako` publicada (`cargo install hako`), CLI + lib

Fila vazia — nenhum item pendente.
