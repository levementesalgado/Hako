# Mizu OS — Planejamento

**Target:** ASUS EeePC 701 (Intel Celeron M 900MHz, 512MB RAM, 4GB SSD, i686)
**Filosofia:** minimalista, fluido, kawaii, didático, do zero
**Sheet:** "Empty your mind, be formless, shapeless — like water." — Bruce Lee

---

## Fase 1 — CLI Environment ✅ (Concluída)

Shell + utilitários Unix rodando no Linux (userspace).

### Entregues
- `mizu-sh` — REPL com prompt colorido, histórico, autocomplete, pipes, redirecionamento
- `mizu-core` — tokenizer, parser de pipeline, expansão de caminho
- 20 builtins: `cd`, `ls`, `cat`, `echo`, `rm`, `rmdir`, `mkdir`, `touch`, `cp`, `mv`, `clear`, `pwd`, `whoami`, `date`, `uname`, `hostname`, `mount`, `umount`, `help`, `exit`
- Cross-compilação i686-unknown-linux-musl (binário estático 1.4MB)
- Boot via initramfs no QEMU

---

## Fase 2 — Kernel bare-metal 🚧 (Em andamento)

### Meta
Kernel i686 do zero com compatibilidade Linux i386 (syscalls) que roda ELFs estáticos musl.

### Etapas

#### Etapa 2.1 — Boot mínimo ✅
- [x] Target spec `i686-mizu.json` + linker script
- [x] Multiboot header (GRUB-compatível)
- [x] Entry point em assembly inline (`global_asm!`)
- [x] VGA text mode driver (write, clear, scroll, cursor)
- [x] Serial port debug output
- [x] PS/2 keyboard polling driver (scancode → ASCII)
- [x] Shell loop com echo, help, clear, uname, whoami, exit
- [x] Boot verificado no QEMU (serial output + VGA)

#### Etapa 2.2 — Interrupts ✅
- [x] GDT (segmentos code/data ring 0 + ring 3 + TSS)
- [x] IDT (exception handlers + IRQ stubs)
- [x] PIC remapeamento (IRQ0-15 → INT 0x20-0x2F)
- [x] PIT timer (IRQ0, 100Hz)
- [x] Teclado via IRQ (substituir polling)

#### Etapa 2.3 — Memória ✅
- [x] Frame allocator bitmap
- [x] Heap com linked-list allocator
- [x] Paging (identiy map + PDE/PTE para processos)
- [x] Page dir por processo

#### Etapa 2.4 — Shell avançado ✅
- [x] Portar mizu-core (tokenizer + pipeline parser) para no_std
- [x] Portar builtins do mizu-sh
- [x] Tab autocomplete

#### Etapa 2.5 — Filesystem ✅
- [x] ATA PIO driver
- [x] Initramfs (tarfs) loader
- [x] FAT leitura/escrita

#### Etapa 2.6 — Multitasking ✅
- [x] Round-robin scheduler
- [x] Modo usuário (ring 3)
- [x] Process table (PID, estado, kernel stack)
- [x] fork (clone de page tables + kernel stack)
- [x] execve (carrega ELF, switch CR3)
- [x] exit/waitpid (zombie reaping)
- [x] Switch entre processos via syscall
- [x] Scheduler com prioridade a PID mais alto

---

## Fase 3 — Compatibilidade Linux i386 🚧 (Em andamento)

### Meta
Rodar binários ELF estáticos compilados com musl (bash, busybox, etc.) via syscalls Linux i386.

### Arquitetura do Kernel

```
mizu-kernel/
├── src/
│   ├── main.rs          → kmain(): init → setup_pid1 → create_user → scheduler
│   ├── interrupts.rs    → IDT, syscall dispatcher (int 0x80), ISR handlers
│   ├── process.rs       → process table, fork/execve/exit/waitpid, scheduler
│   ├── syscall.rs       → constantes (SYS_*, errno)
│   ├── memory.rs        → frame allocator, heap
│   ├── fs.rs            → initramfs, file ops, getdents
│   ├── vga_driver.rs    → VGA text mode
│   ├── serial_driver.rs → COM1 debug
│   ├── keyboard.rs      → PS/2 IRQ driver
│   ├── shell.rs         → REPL nativo do kernel
│   ├── personality/
│   │   ├── mod.rs       → find_first_elf()
│   │   └── linux.rs     → load_elf(), exec_elf()
│   ├── arch/i686/
│   │   ├── gdt.rs       → segmentos ring 0/3, TSS
│   │   ├── paging.rs    → PageDir, PageTable
│   │   └── interrupts.asm → isr stubs, syscall handler
│   └── (hako)           → transpiler de flow pra Rust
├── i686-mizu.json       → target spec
├── build.rs             → hako build script
```

### Status das Syscalls

#### Syscalls implementadas (~40)

| Syscall | Status | Notas |
|---------|--------|-------|
| `exit` | ✅ | Marca zombie, switch proc |
| `exit_group` | ✅ | Wrapper para exit |
| `fork` | ✅ | Clona page tables + kstack |
| `execve` | ✅ | Carrega ELF, argv/envp user |
| `waitpid` / `wait4` | ✅ | Zombie reaping |
| `read` | ⚠️ | Só fd 0 (keyboard/serial) |
| `write` | ✅ | Fd 1/2 → VGA + serial |
| `open` | ⚠️ | Só /dev/tty, /dev/null, initramfs |
| `close` | ✅ | Stub (sempre sucesso) |
| `brk` | ✅ | Aloca heap |
| `mmap` | ✅ | Mapeia páginas anônimas |
| `munmap` | ⚠️ | Stub básico |
| `mprotect` | ⚠️ | Stub básico |
| `getpid` / `getppid` | ✅ | |
| `getuid` / `geteuid` | ✅ | |
| `getgid` / `getegid` | ✅ | |
| `setuid` / `setgid` | ✅ | |
| `chdir` | ✅ | |
| `getcwd` | ✅ | |
| `ioctl` | ⚠️ | Stub |
| `fcntl` / `fcntl64` | ⚠️ | Stub |
| `sigaction` / `rt_sigaction` | ⚠️ | Aceita mas ignora |
| `sigprocmask` / `rt_sigprocmask` | ⚠️ | Aceita mas ignora |
| `sigsuspend` | ⚠️ | Stub |
| `kill` / `tgkill` | ⚠️ | Stub |
| `alarm` | ⚠️ | Stub |
| `pipe` | ❌ | **Próximo passo** |
| `dup` / `dup2` | ❌ | **Próximo passo** |
| `getdents` / `getdents64` | ✅ | Lista initramfs |
| `stat` / `fstat` / `lstat` | ⚠️ | Stat básico |
| `access` | ⚠️ | Stub |
| `uname` | ✅ | |
| `time` | ⚠️ | Retorna 0 |
| `sched_yield` | ✅ | Switch de contexto |
| `getrandom` | ⚠️ | Stub |

---

## Fase 4 — Bash Completo 🎯

### Roadmap para bash rodar

#### Passo 1 — `pipe` + `dup2` (atual)
- [ ] `sys_pipe()` — aloca pipe buffer, cria FDs reader/writer
- [ ] `sys_dup2()` — duplica FD, fecha target se aberto
- [ ] Testar com hello.elf que faz fork + pipe + write

#### Passo 2 — Termios + TTY
- [ ] `ioctl TCGETS/TCSETS` — bash verifica se é terminal
- [ ] `ioctl TIOCGWINSZ` — tamanho do terminal
- [ ] `isatty()` via ioctl
- [ ] `/dev/tty` como dispositivo especial

#### Passo 3 — Job Control básico
- [ ] `setpgid` / `getpgid` — grupos de processo
- [ ] `setsid` — sessões
- [ ] `tcsetpgrp` — foreground process group
- [ ] `SIGCHLD` handling — waitpid sem travar

#### Passo 4 — FD_CLOEXEC + open multi-FD
- [ ] `fcntl F_SETFD/F_GETFD` com FD_CLOEXEC
- [ ] `open()` com pathnames reais do initramfs
- [ ] `close()` multi-FD

#### Passo 5 — bash abre prompt
- [ ] Compilar bash estático com musl
- [ ] Adicionar ao initramfs
- [ ] bash exibe `$ ` e aceita comandos built-in

#### Passo 6 — bash roda comandos simples
- [ ] `execve` com PATH search
- [ ] `pipe` + `dup2` para pipelines
- [ ] Comandos como `ls`, `echo`, `cat` funcionam

#### Passo 7 — Job Control completo
- [ ] `SIGINT` / `SIGQUIT` handling
- [ ] Process groups em foreground/background
- [ ] `fg`, `bg`, `&`, `Ctrl+C`, `Ctrl+Z`

#### Passo 8 — bash completo
- [ ] PTY driver
- [ ] Variáveis de ambiente
- [ ] Scripts complexos
- [ ] Tab completion
- [ ] Aliases, functions

### Cronograma estimado

| Estágio | Prazo | Entrega |
|---------|-------|---------|
| **Passo 1** (pipe+dup2) | 1-2 dias | hello.elf com fork/pipe/write |
| **Passo 2** (termios) | 3-5 dias | bash abre mas morre ao ler comando |
| **Passo 3-4** (job control) | 1 semana | bash aceita Enter, reclama de comandos |
| **Passo 5** (bash prompt) | 2 semanas | bash exibe `$ `, timeout |
| **Passo 6** (comandos) | 3-4 semanas | bash roda `ls`, `echo` |
| **Passo 7-8** (completo) | 2-3 meses | bash funcional com job control |

---

## Marcos

| Marco | O que entrega | Status |
|-------|---------------|--------|
| **M1** | `mizu-sh` rodando no Linux com builtins + pipe | ✅ |
| **M2** | `mizu-sh` cross-compilado i686 rodando no QEMU | ✅ |
| **M3** | Kernel boota em QEMU com VGA TTY + shell | ✅ |
| **M4** | Multitasking + modo usuário + syscalls Linux | ✅ |
| **M5** | hello.elf roda em ring 3, faz exit, volta pro shell | ✅ |
| **M6** | Processos com fork/pipe/dup2 | ❌ |
| **M7** | bash abre prompt no QEMU | ❌ |
| **M8** | Kernel boota no EEEPC com shell interativo | ❌ |

---

## Build

```bash
# mizu-kernel (bare-metal, requer nightly)
cd mizu-kernel && cargo +nightly build \
  -Zjson-target-spec -Zbuild-std-features=compiler-builtins-mem \
  --target i686-mizu.json --release

# Boot no QEMU
qemu-system-i386 \
  -kernel target/i686-mizu/release/mizu-kernel \
  -m 128M -display none -serial stdio
```

---

## Referências
- `https://wiki.osdev.org`
- `https://os.phil-opp.com` — Rust OS Dev (adaptado p/ i686)
- `https://github.com/anomalyco/opencode` — agentic dev
- EEEPC 701 specs: Celeron M 353, 512MB DDR2, 4GB PATA SSD, 7" 800×480
