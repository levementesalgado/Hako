# Hako Syntax

DSL otimizada para LLMs. Cada construção reduz contexto consumido por modelos de linguagem.

## Por que essa sintaxe?

LLMs trabalham com janelas de contexto finitas. Hako minimiza tokens necessários:

- **Boxes** quebram linearidade → LLM referência por nome, não por posição
- **Flows** são listas de nomes → sequência sem código
- **Auto mode** mapeia 1 pra 1 → LLM não precisa saber a implementação
- **Constantes** em box → injetadas automaticamente, não repetidas

## Estrutura

```hako
box nome {
    CONST = valor     // constante (injetada no auto mode)
    func => default   // mapeia pra stdlib (1 linha)
}

flow nome {
    box1              // só nomes, sem código
    box2
}
```

## Boxes

Box = módulo nomeado. LLM pode referenciar por nome.

```hako
box serial {
    COM1 = 0x3F8
    config => default
    write_byte(b) => default
}
```

**Regra:** cada box tem um nome único. LLMs entendem `serial::config()` melhor que `port_outb(0x80, 0x3F8 + 3)`.

## Constantes

Constantes do box são injetadas automaticamente no auto mode:

```hako
box serial {
    COM1 = 0x3F8     // injetada como 1º arg
    config => default // vira serial_config(COM1)
}
```

LLM não precisa repetir o endereço. Uma vez definido, esquece.

## Auto Mode

Função mapeada direto pra stdlib. LLM só precisa saber o nome:

```hako
config => default      // serial_config(COM1)
write_byte(b) => default // serial_write_byte(b, COM1)
read_byte => default   // serial_read_byte(COM1)
```

**Mapeamento completo:**

```
Serial:    config, write_byte, read_byte, write_str
UART:      uart_init, uart_write, uart_read
SPI:       spi_init, spi_transfer, spi_write
I2C:       i2c_init, i2c_start, i2c_stop, i2c_write_byte
GPIO:      gpio_output, gpio_input, gpio_write, gpio_read
VGA:       clear, write, put_char, scroll, set_cursor
Port I/O:  outb, inb, outw, inw
Timer:     pit_config, timer
Keyboard:  init
```

## Block Mode

Código Rust direto quando auto mode não cobre:

```hako
box calc {
    somar {
        let x = 10
        let y = 20
        x + y
    }
}
```

## Raw Mode

Assembly inline pra kernel/bootloader:

```hako
box halt {
    halt raw {
        out(al, x)
        "hlt"
    }
}
```

## Flows

Flow = sequência de boxes. LLM entende a ordem sem ver o código:

```hako
flow boot {
    serial      // serial::run()
    vga         // vga::run()
    keyboard    // keyboard::run()
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

## Controle de Fluxo

Mínimo de tokens:

```hako
if x > 10 => {
    // verdadeiro
} else {
    // falso
}

for i in 0..10 {
    // i de 0 a 9
}

loop {
    if cond => { break }
}
```

## Exemplos Otimizados

### LED (3 boxes, 0 contexto repetido)

```hako
box led {
    PIN = 21
    init { gpio_output(PIN) }
    on { gpio_write(PIN, 1) }
    off { gpio_write(PIN, 0) }
}
```

LLM gera `init()`, `on()`, `off()` sem saber que PIN=21.

### Sensor I2C (sequência complexa, nomes claros)

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

### UART Echo (flow explícito)

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

## Dicas pra LLMs

1. **Boxes pequenos** — 1-3 funções por box
2. **Nomes descritivos** — `serial`, `vga`, `keyboard`
3. **Constantes no topo** — injetadas automaticamente
4. **Flows separados** — ordem de init visível
5. **Auto mode sempre que possível** — menos tokens

## CLI

```bash
hako input.hako -o output.rs    # transpilar
hako input.hako --check         # só validar
```
