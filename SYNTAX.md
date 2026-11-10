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
Serial:    config, write_byte, read_byte (-> u8), write_str
UART:      uart_init, uart_write, uart_read (-> u8)
SPI:       spi_init, spi_transfer (-> u32), spi_write
I2C:       i2c_init, i2c_start, i2c_stop, i2c_write_byte (-> u32)
GPIO:      gpio_output, gpio_input, gpio_write, gpio_read (-> u32)
VGA:       clear, write, put_char, scroll, set_cursor
Port I/O:  outb(value, port), inb(port) -> u8, outw(value, port), inw(port) -> u16
Timer:     pit_config, timer
Keyboard:  init
```

Ordem de args no Port I/O: `outb(0x11, 0x20)` = valor 0x11 na porta 0x20 (valor primeiro, porta depois). `inb(port)` devolve `u8` (o que `in al` lê); `inw(port)` devolve `u16`. As funções de escrita (`write_byte`, `uart_write`) ainda recebem `u32`, então leitura → escrita pede `as u32`.

Com auto mode, `read_byte(p) => default` vira `return port_inb(p)` implícito (sem `;`), então `b = read_byte(0x60)` pode usar o retorno diretamente.

```hako
box pit {
    init { pit_config(100) }          // Timer a 100 Hz
    readw { inw(0x40) }
    writew { outw(0x1234, 0x40) }
}
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

Convenção: todo box no flow precisa ter uma função `run()`. No Mizu usa-se `flow default { process }` que gera `flow_default()` chamando `process::run()`.

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
    gpio_output(pin) => default
    gpio_write(pin, value) => default
    init { gpio_output(PIN) }
    on { gpio_write(PIN, 1) }
    off { gpio_write(PIN, 0) }
    run {
        init()
        on()
    }
}
```

LLM gera `init()`, `on()`, `off()` sem saber que PIN=21. Em block mode, chamadas sem prefixo resolvem pra funções do próprio box (os wrappers `gpio_output`/`gpio_write`).

### Sensor I2C (sequência complexa, nomes claros)

```hako
box sensor {
    SDA = 2
    SCL = 3
    i2c_init(sda, scl) => default
    i2c_start(sda, scl) => default
    i2c_stop(sda, scl) => default
    i2c_write_byte(sda, scl, byte) => default
    run {
        i2c_init(SDA, SCL)
        i2c_start(SDA, SCL)
        i2c_write_byte(SDA, SCL, 0xEC)
        i2c_stop(SDA, SCL)
    }
}
```

### UART Echo (flow explícito)

```hako
box serial {
    COM1 = 0x3F8
    config => default
    read_byte => default
    write_byte(b) => default
    run {
        config()
        loop {
            let b = read_byte()
            write_byte(b as u32)
        }
    }
}

flow default {
    serial
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
