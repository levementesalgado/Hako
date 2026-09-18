# Hako

[![CI](https://github.com/levementesalgado/Hako/actions/workflows/ci.yml/badge.svg)](https://github.com/levementesalgado/Hako/actions)
[![Crates.io](https://img.shields.io/crates/v/hako.svg)](https://crates.io/crates/hako)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

DSL para programação de baixo nível, transpilada para Rust. Projetada para kernels, drivers, bootloaders e sistemas embarcados.

## Por que Hako?

Rust é poderoso, mas verboso para hardware. Hako simplifica:

```hako
// Hako
box serial {
    COM1 = 0x3F8
    config => default
    write_byte(b) => default
}

box main {
    config()
    write_byte(0x41)
}
```

Transpila para Rust puro, pronto pra compilar com `rustc` ou `cargo`.

## Instalação

```bash
cargo install hako
```

## Uso

```bash
# Transpilar
hako input.hako -o output.rs

# Só validar syntax
hako input.hako --check

# Compilar e rodar
rustc output.rs -o output
./output
```

## Exemplos

### Hello World (Serial)

```hako
box serial {
    COM1 = 0x3F8
    config => default
    write_byte(b) => default
}

box main {
    config()
    write_byte(72)  // H
    write_byte(101) // e
    write_byte(108) // l
    write_byte(108) // l
    write_byte(111) // o
}
```

### GPIO LED

```hako
box led {
    PIN = 21

    init {
        gpio_output(PIN)
    }

    on {
        gpio_write(PIN, 1)
    }

    off {
        gpio_write(PIN, 0)
    }
}
```

### UART Echo

```hako
box uart {
    BASE = 0x3F8

    init {
        uart_init(BASE)
    }

    echo {
        init()
        loop {
            let byte = uart_read(BASE)
            uart_write(BASE, byte)
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

### SPI Display

```hako
box display {
    MOSI = 2
    SCLK = 3
    CS = 4

    init {
        spi_init(MOSI, SCLK, CS)
    }

    send_cmd(cmd) {
        spi_transfer_byte(MOSI, SCLK, CS, cmd)
    }
}
```

## Stdlib

Hako inclui uma stdlib completa para hardware:

| Módulo | Funções |
|--------|---------|
| **Serial** | `config`, `write_byte`, `read_byte`, `write_str` |
| **UART** | `init`, `write`, `read`, `write_str` |
| **SPI** | `init`, `transfer_byte`, `write` |
| **I2C** | `init`, `start`, `stop`, `write_bit`, `read_bit`, `write_byte` |
| **GPIO** | `output`, `input`, `write`, `read` |
| **VGA** | `put_char`, `write_str`, `clear`, `scroll`, `set_cursor` |
| **Port I/O** | `outb`, `inb`, `outw`, `inw` |
| **PIT** | `config` |
| **Keyboard** | `init` |

## Sintaxe

Ver [SYNTAX.md](SYNTAX.md) para referência completa.

```hako
// Constantes
box hardware {
    BASE = 0x3F8
    FLAG = 0xFF
}

// Funções auto (mapeia pra stdlib)
config => default

// Funções com bloco
minha_funcao {
    let x = 42
    gpio_output(x)
}

// Flows (sequência de boxes)
flow boot {
    serial
    vga
    keyboard
}
```

## Estrutura do Repo

```
Hako/
├── src/
│   ├── lib.rs          API pública
│   ├── main.rs         CLI
│   ├── parser.rs       Parser recursivo descendente
│   ├── codegen.rs      Gerador de código Rust
│   ├── ast.rs          Tipos da AST
│   └── stdlib.rs       Stdlib (serial, VGA, SPI, I2C, GPIO)
├── tests/              81 testes
├── examples/           5 exemplos
├── SYNTAX.md           Documentação de sintaxe
├── LICENSE-MIT
└── LICENSE             Apache 2.0
```

## Desenvolvimento

```bash
# Rodar testes
cargo test

# Clippy
cargo clippy -- -D warnings

# Formatar
cargo fmt

# Build release
cargo build --release
```

## Licença

Dual-licenciado sob [MIT](LICENSE-MIT) e [Apache 2.0](LICENSE).
