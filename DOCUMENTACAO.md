# Mizu Kernel v0.1.0

**Mizu** é um kernel híbrido com múltiplas ABIs para i686, escrito em Rust, com
suporte a múltiplas personalidades (personalities) para executar binários de
diferentes sistemas operacionais (Linux, Windows, DOS) em ring 3.

---

## Arquitetura

```
┌──────────────────────────────────────────────────────┐
│             Shell (ring 0)                           │
│  LineEditor, comandos: ls, cat, echo, help, etc      │
├──────────────────────────────────────────────────────┤
│            Personality Layer                          │
│  Linux ABI (int 0x80): sys_write, sys_exit            │
│  ELF loader (32-bit, PT_LOAD)                         │
├──────────────────────────────────────────────────────┤
│              Kernel Core (ring 0)                     │
│  GDT · IDT · TSS · PIC · PIT · Keyboard              │
│  Frame allocator · Heap allocator · Initramfs         │
│  VGA text mode · Serial (COM1)                        │
│  Hako (DSL flow engine)                               │
├──────────────────────────────────────────────────────┤
│       Arch: i686 (x86 protegido, sem paginação)       │
│       Boot: GRUB · Multiboot · link.ld (0x100000)     │
└──────────────────────────────────────────────────────┘
```

### Fluxo de inicialização

```
kmain()
 ├─ serial_driver::init()          ← COM1 38400 baud
 ├─ vga_driver::clear()            ← modo texto 80×25
 ├─ memory::init(mem_upper)        ← frame bitmap + heap 2MB
 ├─ fs::init()                     ← initramfs embutido (tar)
 ├─ arch::gdt::init(0x11B590)      ← GDT + TSS (ring 3)
 ├─ interrupts::init()             ← IDT, int 0x80 (DPL=3)
 ├─ pic::init() + pit::init(100Hz) ← IRQ remapeamento
 ├─ keyboard::init()               ← PS/2, scan codes
 ├─ crate::sys::sti()              ← habilita IRQs
 ├─ flow_default()                 ← Hako (teste)
 ├─ personality::init()            ← carrega ELF da initramfs
 │   └─ linux::load_elf()          ← parseia e copia segmentos
 │   └─ linux::jump_to_user()      ← iretd → ring 3
 │       └─ ELF executa em ring 3
 │           ├─ sys_write → VGA + serial
 │           └─ sys_exit  → shell::shell_loop()
 └─ shell::shell_loop()            ← (fallback se não achar ELF)
```

---

## GDT — Global Descriptor Table

Endereçada em `arch/i686/gdt.rs`. Layout:

| Índice | Seletor | Tipo          | DPL | Descrição              |
|--------|---------|---------------|-----|------------------------|
| 0      | —       | null          | —   | obrigatório pelo x86   |
| 1      | 0x08    | kernel code   | 0   | execução em ring 0     |
| 2      | 0x10    | kernel data   | 0   | dados/stack em ring 0  |
| 3      | 0x1B    | user code     | 3   | execução em ring 3     |
| 4      | 0x23    | user data     | 3   | dados/stack em ring 3  |
| 5      | 0x28    | TSS           | 3   | Task State Segment     |

- **TSS** contém `ss0` e `esp0` — stack que o CPU carrega ao fazer
  ring 3 → ring 0 (interrupções, `int 0x80`).
- `esp0` = topo da kernel stack (`~0x11B590`, símbolo `stack_top` no link.ld).

### Bug corrigido: `gdt_entry()` / `tss_entry()`

As funções que montam o descriptor de 64 bits estavam colocando `b2` (bits 23-16
da base) no slot de `b3` (bits 63-56), e `b3` nunca era escrito. Como os segmentos
de código/dados usam base=0, só o TSS (base não-zero) era afetado, causando
**Invalid TSS (#10)** em qualquer transição ring 3 → ring 0.

Correção: `(b2 << 32) | ... | (access << 40) | ... | (b3 << 56)`

---

## IDT — Interrupt Descriptor Table

- 48 entries para exceções + IRQs + 1 para syscall.
- Entradas 0-31: exceções da CPU (DPL=0).
- Entradas 32-47: IRQs remapeadas do PIC (DPL=0).
- **Entrada 0x80 (int 0x80)**: DPL=3, gate type=0xE (interrupt gate).
- Handler em assembly (`arch/i686/interrupts.asm`): `isr_common` para exceções,
  `syscall_handler` para int 0x80.

### Bug corrigido: `isr_common` vs `syscall_handler`

`isr_common` empurra `error_code` + `ISR_num` (2 × 4 bytes) antes do `pusha`.
Esses 8 bytes são limpos com `add esp, 8` antes do `iretd`.

O `syscall_handler` original copiava esse `add esp, 8` cegamente, mas a pilha
do syscall NÃO tem error code nem ISR number — apenas **5 items** do CPU
(SS, ESP, EFLAGS, CS, EIP) + pusha + segmentos. O `add esp, 8` a mais fazia
o `iretd` ler EFLAGS como EIP, causando GPF.

---

## Syscall Handler (int 0x80)

Convenção Linux i386: `eax` = número da syscall, `ebx/ecx/edx/esi/edi` = args.

### Pilha durante `rust_handle_syscall`

```
End. baixo
  [GS]          ← saved_regs[0]   ← &saved_regs aponta aqui
  [FS]          ← saved_regs[1]
  [ES]          ← saved_regs[2]
  [DS]          ← saved_regs[3]
  [EDI]         ← saved_regs[4]
  [ESI]         ← saved_regs[5]
  [EBP]         ← saved_regs[6]
  [ESP_pusha]   ← saved_regs[7]
  [EBX]         ← saved_regs[8]
  [EDX]         ← saved_regs[9]
  [ECX]         ← saved_regs[10]
  [EAX]         ← saved_regs[11]
  [EIP_user]    ← saved_regs + 12 (CPU frame)
  [CS_user]     ← saved_regs + 13
  [EFLAGS]      ← saved_regs + 14
  [ESP_user]    ← saved_regs + 15
  [SS_user]     ← saved_regs + 16
End. alto
```

### Bug corrigido: `&[u32; 8]` → `&[u32; 12]`

O handler original recebia `&[u32; 8]` assumindo só os 8 registradores do
`pusha`. Mas o assembly empurra **4 segment registers** (gs/fs/es/ds) ANTES do
`pusha`, totalizando 12 valores. O `saved_regs[8]` lia `EBP` em vez de `EBX`,
trocando os argumentos da `sys_write` (buf e len trocados).

---

## Syscalls implementados

| Nº | Nome          | args                        | Comportamento                          |
|----|---------------|-----------------------------|----------------------------------------|
| 1  | `sys_exit`    | `int status`                | Chama `shell::shell_loop()` (ring 0)   |
| 4  | `sys_write`   | `int fd, char *buf, size_t` | Escreve em VGA + serial se fd=1 ou fd=2|

---

## Shell

Definido em `shell.rs`. Shell em Português com line editor completo.

### Funcionalidades

- Navegação: setas direita/esquerda, Ctrl+A/E (home/end)
- Histórico: setas cima/baixo, Ctrl+P/N
- Tab completion (comandos internos + arquivos da initramfs)
- Ctrl+C interrompe, Ctrl+D sai se linha vazia, Ctrl+L limpa tela
- Ctrl+U limpa linha, Backspace apaga
- Escape sequences ANSI interpretadas (setas, home, end, del)

### Comandos implementados

`help`, `echo`, `clear`, `uname`, `whoami`, `date`, `yes`, `ls`, `cat`,
`head`, `wc`, `hexdump`, `true`, `false`, `exit`, `quit`, `sh`

### `sys_exit` → shell

Quando um programa ring 3 chama `sys_exit(0)`, o handler em `interrupts.rs`
simplesmente invoca `shell::shell_loop()` diretamente. Como a função tem
tipo `-> !`, ela nunca retorna — o `iretd` no final do `syscall_handler` nunca
é alcançado. O shell roda em ring 0 com os segmentos de kernel (CS=0x08,
DS=ES=FS=GS=0x10) e a pilha do kernel.

---

## Personality Layer (Linux ELF)

`personality/linux.rs` — carrega e executa ELF 32-bit.

### ELF loader

1. Valida magic `\x7fELF`, classe 32-bit, tipo ET_EXEC
2. Itera sobre program headers, busca `PT_LOAD`
3. Para cada segmento: aloca frames físicos no endereço `p_vaddr`
   via `memory::alloc_frame_at()`
4. Copia dados do segmento para a memória
5. Aloca frame extra para stack (topo = `stack_frame + 0x1000`)

### jump_to_user()

Monta frame na pilha e executa `iretd`:

```
push SS_user    (0x23)
push stack_top  (esp do usuário)
push EFLAGS     (0x202 = IF=1)
push CS_user    (0x1B)
push entry      (ponto de entrada do ELF)
iretd
```

### hello.elf (teste)

Fonte em `/tmp/hello.S`, linked em `0x500000`. `sys_write(1, msg, 19)` →
"Hello from Linux!" em VGA + serial. `sys_exit(0)` → shell.

---

## Memória

### Frame allocator

- Bitmap de 128K frames (512 MB max)
- Frames 0 até `kernel_end()/4096`: marcados como usados
- Frames acima: livres
- `alloc_frame()`: first-fit
- `alloc_frame_at(addr)`: aloca frame específico (usado pelo ELF loader)

### Heap allocator

- 2 MB contíguos logo após o kernel
- Free list encadeada com splitting
- `GlobalAlloc` impl → Vec, String, etc. funcionam

### Layout de memória

```
Endereço    Região
0x000000    BIOS / IVT / BDA
0x000500    BIOS data area
0x100000    Kernel (text + data + bss) ← linked aqui
...         (heap 2MB logo após bss)
0x500000    hello.elf (carregado pelo personality)
0xB8000     VGA text buffer
0x11B590    Kernel stack top (stack_top, ~16KB)
0x11D000    Stack do usuário (ring 3, 4KB)
```

---

## Drivers

### VGA (modo texto 80×25)

- Buffer em `0xB8000`, 2 bytes por caractere (char + atributo)
- Scroll, cursor (via ports 0x3D4/0x3D5), cores ANSI
- Toda saída espelha para serial (`serial_driver::put_char`)

### Serial (COM1, 0x3F8)

- 38400 baud, 8n1
- `put_char()` com polling (timeout 10000 iterações)
- `get_char()`: poll, retorna `Some(c)` ou `None`
- Usado para debug e entrada em modo `-nographic`

### Teclado PS/2

- IRQ1 (interrupt 33)
- Scan codes traduzidos via `SCAN_NORMAL` / `SCAN_SHIFT`
- Suporte a Shift, Ctrl, Caps Lock
- Buffer circular de 128 bytes
- Polling direto do port 0x60 como fallback

### PIC (8259A)

- Remapeamento: IRQ0→INT 0x20, IRQ8→INT 0x28
- Mascara tudo exceto timer (IRQ0) e teclado (IRQ1)

### PIT (8253)

- 100 Hz, channel 0, rate generator
- `tick()` incrementa contador atômico

---

## Hako

DSL para orquestração de hardware, escrita em Rust puro via macro.
Arquivos `.hako` em `mizu-hako/` são transpilados para Rust via `build.rs`.
Usado para inicialização de PIC, PIT, e teste de fluxo (`flow_default()`).

---

## Build & Run

```bash
# Dependências
rustup target add i686-unknown-linux-gnu
cargo install cargo-binutils

# Build
cd mizu-kernel
cargo +nightly build -Zbuild-std=core,alloc,compiler_builtins \
  --target i686-mizu.json --release -p mizu-kernel

# ISO
cp target/i686-mizu/release/mizu-kernel /tmp/mizu-iso/boot/mizu.bin
grub-mkrescue -o /tmp/mizu.iso /tmp/mizu-iso

# Executar (modo texto serial)
qemu-system-i386 -cdrom /tmp/mizu.iso -m 128M -nographic -no-reboot

# Executar (modo gráfico)
qemu-system-i386 -cdrom /tmp/mizu.iso -m 128M -vga std
```

### Initramfs

Arquivos em `/tmp/initramfs/`, empacotados em tar via `build.rs`.
Atualmente contém: `hello.elf` (ELF de teste), `dummy`, `hello.txt`, `lorem.txt`.

---

## Próximos passos

1. `brk` (syscall 45) — heap dinâmico para programas ring 3
2. `mmap2` (syscall 192) — mapeamento de memória
3. `stat64`, `fstat64` — consulta de metadados de arquivo
4. Port do `musl` e `busybox` para Mizu
5. Múltiplas personalidades (Win32, DOS)
6. Paginação para ELFs com endereços virtuais arbitrários
7. Shell executar ELFs da initramfs como comando `sh`
