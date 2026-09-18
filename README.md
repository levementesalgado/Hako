# Hako

[![CI](https://github.com/levementesalgado/Hako/actions/workflows/ci.yml/badge.svg)](https://github.com/levementesalgado/Hako/actions)
[![Crates.io](https://img.shields.io/crates/v/hako.svg)](https://crates.io/crates/hako)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

DSL otimizada para LLMs. Programação de baixo nível com contexto mínimo.

## Otimização pra LLM

LLMs trabalham com janelas de contexto finitas. Hako minimiza tokens:

| Feature | Reduz | Exemplo |
|---------|-------|---------|
| **Boxes** | linearidade | `serial.config()` vs 10 linhas de port I/O |
| **Flows** | código de init | `flow boot { serial vga }` vs sequência procedural |
| **Auto mode** | implementação | `config => default` vs `port_outb(0x80, 0x3F8+3)` |
| **Constantes** | repetição | `COM1 = 0x3F8` definido 1x, injetado N vezes |

## Instalação

```bash
cargo install hako
```

## Uso

```bash
hako input.hako -o output.rs    # transpilar
hako input.hako --check         # só validar
rustc output.rs -o output       # compilar
```

## Exemplo

```hako
box serial {
    COM1 = 0x3F8
    config => default
    write_byte(b) => default
}

box main {
    config()
    write_byte(72)  // H
}
```

Transpila pra Rust puro, sem dependências.

## Boxes

Módulos nomeados. LLM referência por nome, não por código:

```hako
box vga {
    BASE = 0xB8000
    clear => default
    write(s) => default
}
```

## Flows

Sequência de inicialização. Só nomes, sem código:

```hako
flow boot {
    serial
    vga
    keyboard
}
```

## Stdlib

Mapeamentos auto pra hardware:

```
Serial:    config, write_byte, read_byte
UART:      uart_init, uart_write, uart_read
SPI:       spi_init, spi_transfer
I2C:       i2c_init, i2c_start, i2c_stop
GPIO:      gpio_output, gpio_input, gpio_write, gpio_read
VGA:       clear, write, put_char, scroll
Port I/O:  outb, inb, outw, inw
Timer:     pit_config
Keyboard:  init
```

Ver [SYNTAX.md](SYNTAX.md) para referência completa.

## Exemplos

### GPIO LED
```hako
box led {
    PIN = 21
    init { gpio_output(PIN) }
    on { gpio_write(PIN, 1) }
    off { gpio_write(PIN, 0) }
}
```

### UART Echo
```hako
box uart {
    BASE = 0x3F8
    init { uart_init(BASE) }
    echo {
        init()
        loop {
            let b = uart_read(BASE)
            uart_write(BASE, b)
        }
    }
}
```

### I2C Sensor
```hako
box sensor {
    SDA = 2
    SCL = 3
    read {
        i2c_init(SDA, SCL)
        i2c_start(SDA, SCL)
        i2c_write_byte(SDA, SCL, 0xEC)
        i2c_stop(SDA, SCL)
    }
}
```

## Desenvolvimento

```bash
cargo test      # 81 testes
cargo clippy    # 0 warnings
cargo fmt       # formatar
```

## Licença

Dual-licenciado sob [MIT](LICENSE-MIT) e [Apache 2.0](LICENSE).
