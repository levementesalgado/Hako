# Hako Syntax

Referência completa da linguagem Hako.

## Estrutura Básica

```hako
// Comentário de linha
/* Comentário de bloco */

box nome {
    //常量
    CONSTANTE = 0x3F8

    // Função (modo auto)
    funcao => default

    // Função com parâmetros
    funcao_com_param(param) => default

    // Função com bloco
    funcao_com_bloco {
        // código
    }
}

flow nome {
    box1
    box2
}
```

## Boxes

Boxes são módulos que agrupam hardware relacionado.

```hako
box serial {
    COM1 = 0x3F8
    COM2 = 0x2F8

    config => default
    write_byte(b) => default
    read_byte => default
}

box vga {
    BASE = 0xB8000

    clear => default
    write(s) => default
    put_char(c, x, y) => default
}
```

## Constantes

```hako
box exemplo {
    HEX = 0xFF        // hexadecimal
    DEC = 255         // decimal
    BIN = 0b10101010  // binário
    OCT = 0o377       // octal
    NAME = "serial"   // string
}
```

## Modos de Função

### Auto Mode

Mapeia automaticamente para funções da stdlib:

```hako
box serial {
    COM1 = 0x3F8
    config => default      // → serial_config(COM1)
    write_byte(b) => default // → serial_write_byte(b, COM1)
    read_byte => default   // → serial_read_byte(COM1)
}
```

**Mapeamentos disponíveis:**

| Hako | Stdlib |
|------|--------|
| `config` / `setup` | `serial_config` |
| `write_byte(b)` | `serial_write_byte` |
| `read_byte` | `serial_read_byte` |
| `write_str(s)` | `serial_write_str` |
| `init` | `keyboard_init` |
| `clear` | `vga_clear` |
| `write(s)` | `vga_write_str` |
| `put_char(c)` | `vga_put_char` |
| `scroll` | `vga_scroll` |
| `outb(port, val)` | `port_outb` |
| `inb(port)` | `port_inb` |
| `outw(port, val)` | `port_outw` |
| `inw(port)` | `port_inw` |
| `uart_init` | `uart_init` |
| `uart_write(b)` | `uart_write` |
| `uart_read` | `uart_read` |
| `spi_init` | `spi_init` |
| `spi_transfer(b)` | `spi_transfer_byte` |
| `i2c_init` | `i2c_init` |
| `i2c_start` | `i2c_start` |
| `i2c_stop` | `i2c_stop` |
| `i2c_write_byte(b)` | `i2c_write_byte` |
| `gpio_output` | `gpio_output` |
| `gpio_input` | `gpio_input` |
| `gpio_write(v)` | `gpio_write` |
| `gpio_read` | `gpio_read` |
| `pit_config` / `timer` | `pit_config` |

### Block Mode

Bloco de código Rust direto:

```hako
box exemplo {
    my_func {
        let x = 42;
        let y = x * 2;
        x + y
    }
}
```

### Raw Mode

Assembly inline:

```hako
box halt {
    halt raw {
        out(al, x)
        "hlt"
    }
}
```

## Flows

Flows conectam boxes em sequência:

```hako
box serial { COM1 = 0x3F8 }
box vga { BASE = 0xB8000 }
box keyboard { }

flow boot {
    serial
    vga
    keyboard
}
```

Gera:
```rust
pub fn flow_boot() {
    serial::run();
    vga::run();
    keyboard::run();
}
```

## Variáveis Locais

```hako
box exemplo {
    funcao {
        let x = 10
        let y = 20
        let soma = x + y
    }
}
```

## Controle de Fluxo

### If/Else

```hako
box exemplo {
    checa {
        if x > 10 => {
            // maior
        } else {
            // menor ou igual
        }
    }
}
```

### For

```hako
box exemplo {
    loop {
        for i in 0..10 {
            // i vai de 0 a 9
        }
    }
}
```

### Loop

```hako
box exemplo {
    infinito {
        loop {
            // código infinito
            ifcond => {
                break
            }
        }
    }
}
```

### Break

```hako
loop {
    if x == 0 => {
        break
    }
}
```

## Exemplos Completos

### LED Driver (GPIO)

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

    blink {
        init()
        loop {
            on()
            // delay
            off()
            // delay
        }
    }
}
```

### Sensor I2C

```hako
box sensor {
    SDA = 2
    SCL = 3

    init {
        i2c_init(SDA, SCL)
    }

    read_temp {
        i2c_start(SDA, SCL)
        i2c_write_byte(SDA, SCL, 0xEC)
        i2c_write_byte(SDA, SCL, 0xFA)
        i2c_stop(SDA, SCL)
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

### SPI LED Strip

```hako
box ws2812 {
    MOSI = 2
    SCLK = 3
    CS = 4

    init {
        spi_init(MOSI, SCLK, CS)
    }

    color(r, g, b) {
        spi_transfer_byte(MOSI, SCLK, CS, r)
        spi_transfer_byte(MOSI, SCLK, CS, g)
        spi_transfer_byte(MOSI, SCLK, CS, b)
    }
}
```

## Stdlib

### Port I/O
- `port_outb(value, port)` — escreve byte
- `port_inb(port)` — lê byte
- `port_outw(value, port)` — escreve word
- `port_inw(port)` — lê word

### Serial (COM)
- `serial_config(com)` — configura COM
- `serial_write_byte(b, com)` — escreve byte
- `serial_read_byte(com)` — lê byte
- `serial_write_str(s, com)` — escreve string
- `serial_read_str(com)` — lê string

### UART (16550)
- `uart_init(base)` — inicializa UART
- `uart_write(base, byte)` — escreve byte
- `uart_read(base)` — lê byte
- `uart_write_str(base, s)` — escreve string

### SPI
- `spi_init(mosi, sclk, cs)` — inicializa SPI
- `spi_transfer_byte(mosi, sclk, cs, byte)` — transfere byte
- `spi_write(mosi, sclk, cs, data)` — escreve dados

### I2C
- `i2c_init(sda, scl)` — inicializa I2C
- `i2c_start(sda, scl)` — condição START
- `i2c_stop(sda, scl)` — condição STOP
- `i2c_write_bit(sda, scl, bit)` — escreve bit
- `i2c_read_bit(sda, scl)` — lê bit
- `i2c_write_byte(sda, scl, byte)` — escreve byte

### GPIO
- `gpio_output(pin)` — configura como saída
- `gpio_input(pin)` — configura como entrada
- `gpio_write(pin, value)` — escreve valor
- `gpio_read(pin)` — lê valor

### VGA
- `vga_put_char(c, x, y)` — imprime caractere
- `vga_write_str(s)` — imprime string
- `vga_clear()` — limpa tela
- `vga_scroll()` — rola tela
- `vga_set_cursor(x, y)` — posiciona cursor
- `vga_read_cursor()` — lê posição do cursor

### PIT (Timer)
- `pit_config(freq)` — configura timer

### Keyboard
- `keyboard_init()` — inicializa teclado

## CLI

```bash
# Transpilar
hako input.hako -o output.rs

# Só validar syntax
hako input.hako --check

# Instalar
cargo install hako
```
