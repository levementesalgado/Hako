┌─────────────────────────────────────────────────────────────────────────────┐
│                          HARDWARE (qualquer um)                             │
│              x86, ARM, RISC-V, PowerPC, MIPS, SPARC, ...                    │
└─────────────────────────────────┬───────────────────────────────────────────┘
                                  │
┌─────────────────────────────────▼───────────────────────────────────────────┐
│                    MIZU MICROKERNEL (núcleo imutável)                       │
│  • Scheduler (round-robin, priority, real-time)                             │
│  • IPC (mensagens, shared memory, signals)                                  │
│  • VM (paging, MMU, TLB shootdown)                                          │
│  • IRQ (interrupts, APIC, MSI)                                              │
│  • Timers (PIT, HPET, TSC)                                                  │
│  • Boot (multiboot, UEFI)                                                   │
└─────────────────────────────────┬───────────────────────────────────────────┘
                                  │
        ┌─────────────────────────┼─────────────────────────┐
        │                         │                         │
┌───────▼────────┐       ┌───────▼────────┐       ┌───────▼────────┐
│  PERSONALITY   │       │  PERSONALITY   │       │  PERSONALITY   │
│    LOADER      │       │    LOADER      │       │    LOADER      │
│  (detecta ABI) │       │  (detecta ABI) │       │  (detecta ABI) │
└───────┬────────┘       └───────┬────────┘       └───────┬────────┘
        │                         │                         │
┌───────▼────────┐       ┌───────▼────────┐       ┌───────▼────────┐
│   LINUX        │       │   WIN32        │       │   POSIX        │
│   PERSONALITY  │       │   PERSONALITY  │       │   PERSONALITY  │
└───────┬────────┘       └───────┬────────┘       └───────┬────────┘
        │                         │                         │
┌───────▼────────┐       ┌───────▼────────┐       ┌───────▼────────┐
│  LINUX         │       │  WINDOWS       │       │  BSD           │
│  SYSCALLS      │       │  SYSCALLS      │       │  SYSCALLS      │
│  (400+)        │       │  (2000+)       │       │  (300+)        │
└───────┬────────┘       └───────┬────────┘       └───────┬────────┘
        │                         │                         │
        └─────────────────────────┼─────────────────────────┘
                                  │
┌─────────────────────────────────▼───────────────────────────────────────────┐
│                    EMULATION LAYER (se falhar em nativo)                    │
│  • QEMU integration (full system emulation)                                 │
│  • Rosetta (x86 → ARM)                                                      │
│  • Box64/86 (userspace emulation)                                           │
└─────────────────────────────────┬───────────────────────────────────────────┘
                                  │
┌─────────────────────────────────▼───────────────────────────────────────────┐
│                      DRIVER FRAMEWORK (unificado)                           │
│  • Linux DRM/KMS, Windows WDDM, Android HIDL, ...                           │
│  • Hako como linguagem padrão para drivers                                  │
└─────────────────────────────────┬───────────────────────────────────────────┘
                                  │
┌─────────────────────────────────▼───────────────────────────────────────────┐
│                         USERSPACE (múltiplo)                                │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐                │
│  │ Linux   │ │ Windows │ │ Android │ │   Mac   │ │   Web   │                │
│  │ Apps    │ │ Apps    │ │ Apps    │ │   Apps  │ │   Apps  │                │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘                │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐                │
│  │  DOS    │ │  VB6    │ │ .NET    │ │  Java   │ │  WASM   │                │
│  │  Apps   │ │  Apps   │ │  Apps   │ │  Apps   │ │  Apps   │                │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘                │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐                │
│  │ Python  │ │ Node.js │ │   Go    │ │  Rust   │ │  Zig    │                │
│  │  Apps   │ │  Apps   │ │  Apps   │ │  Apps   │ │  Apps   │                │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘                │
└─────────────────────────────────────────────────────────────────────────────┘


// ============================================================================
// Mizu Universal Kernel — Arquitetura Completa em Hako
// ============================================================================

box mizu_core {
  // Estrutura de uma tarefa (PCB — Process Control Block)
  struct Task {
    id: u32
    name: [u8; 32]
    personality: Personality  // LINUX, WIN32, POSIX, etc.
    state: TaskState          // READY, RUNNING, BLOCKED, ZOMBIE
    priority: u8
    stack: *mut u8
    page_dir: *mut u8
    syscall_table: *mut SyscallEntry
    emu_level: EmuLevel       // NATIVE, EMULATED, INTERPRETED
  }

  enum Personality {
    LINUX = 1
    WIN32 = 2
    POSIX = 3
    BSD = 4
    DARWIN = 5
    ANDROID = 6
    DOS = 7
    VB6 = 8
    DOTNET = 9
    JAVA = 10
    WASM = 11
    WEB = 12
  }

  enum EmuLevel {
    NATIVE = 0      // syscall nativa Mizu
    EMULATED = 1    // syscall emulada (ex: Linux no Mizu)
    INTERPRETED = 2 // VM completa (ex: DOS, VB6)
    HARDWARE = 3    // QEMU full system
  }

  // Tabela global de personalidades
  personality_table: [PersonalityHandler; 16]

  // Registra uma nova personalidade (ABI)
  register_personality(id: Personality, handler: *mut PersonalityHandler) {
    personality_table[id as usize] = handler
  }

  // Despacha syscall baseado na personalidade da tarefa
  dispatch_syscall(task: &mut Task, syscall_num: u32, args: &[u32]) -> u32 {
    let handler = personality_table[task.personality as usize]
    if handler.emu_level == EmuLevel::NATIVE {
      // syscall implementada nativamente no Mizu
      handler.native_syscall(syscall_num, args)
    } else if handler.emu_level == EmuLevel::EMULATED {
      // traduz syscall para Mizu
      handler.emulate_syscall(syscall_num, args)
    } else {
      // fallback para VM ou QEMU
      handler.interpret_syscall(syscall_num, args)
    }
  }

  // Scheduler round-robin com prioridades
  schedule() {
    let current = self::get_current_task()
    let next = self::find_next_ready_task()
    if next != current {
      self::switch_context(current, next)
    }
  }

  // Switch de contexto (assembly)
  switch_context(old: &mut Task, new: &mut Task) raw {
    "pusha"
    "push ds"
    "push es"
    "push fs"
    "push gs"
    "mov [old.stack], esp"
    "mov esp, [new.stack]"
    "mov cr3, [new.page_dir]"   // troca tabela de páginas
    "pop gs"
    "pop fs"
    "pop es"
    "pop ds"
    "popa"
    "iret"
  }
}

// ============================================================================
// PERSONALITY: Linux (emulação nativa)
// ============================================================================

box personality_linux {
  // Tabela de syscalls Linux (400+)
  syscall_table: [fn(&[u32]) -> u32; 450]

  init() {
    // Preenche tabela com handlers
    syscall_table[0] = self::sys_read
    syscall_table[1] = self::sys_write
    syscall_table[2] = self::sys_open
    syscall_table[3] = self::sys_close
    syscall_table[57] = self::sys_fork
    syscall_table[59] = self::sys_execve
    // ... mais 400+
  }

  // read(fd, buf, count)
  sys_read(args: &[u32]) -> u32 {
    let fd = args[0] as i32
    let buf = args[1] as *mut u8
    let count = args[2] as u32

    if fd == 0 {  // stdin
      // ler do teclado
      keyboard::read(buf, count)
    } else {
      // chamar driver de arquivo Mizu
      file::read(fd, buf, count)
    }
  }

  // write(fd, buf, count)
  sys_write(args: &[u32]) -> u32 {
    let fd = args[0] as i32
    let buf = args[1] as *const u8
    let count = args[2] as u32

    if fd == 1 || fd == 2 {  // stdout/stderr
      vga::write_str(from_raw_parts(buf, count))
    } else {
      file::write(fd, buf, count)
    }
    count
  }

  // fork() — cria novo processo
  sys_fork(args: &[u32]) -> u32 {
    let parent = scheduler::current_task()
    let child = scheduler::clone_task(parent)
    // Copia página por página (COW - copy on write)
    vm::copy_address_space(parent, child)
    child.pid as u32
  }

  // execve(filename, argv, envp)
  sys_execve(args: &[u32]) -> u32 {
    let filename = args[0] as *const u8
    let argv = args[1] as *const *const u8
    let envp = args[2] as *const *const u8

    // Carrega ELF
    let elf = elf::load(filename)
    // Configura novo espaço de endereçamento
    vm::new_address_space(elf.entry)
    // Carrega argumentos na stack
    stack::setup_argv(argv, envp)
    // Jump para entry point
    elf::jump(elf.entry)
  }
}

// ============================================================================
// PERSONALITY: Win32 (emulação via Wine/ReactOS)
// ============================================================================

box personality_win32 {
  // Tabela de syscalls Windows (NT API)
  syscall_table: [fn(&[u32]) -> u32; 2000]

  init() {
    // Syscalls base do NT kernel
    syscall_table[0] = self::nt_create_file
    syscall_table[1] = self::nt_read_file
    syscall_table[2] = self::nt_write_file
    syscall_table[3] = self::nt_close
    syscall_table[4] = self::nt_create_process
    // ... mais 1996
  }

  // NtCreateFile
  nt_create_file(args: &[u32]) -> u32 {
    let path = args[1] as *const u16  // UTF-16
    let access = args[2] as u32
    let share = args[3] as u32

    // Converte path UTF-16 para UTF-8
    let utf8_path = utf16_to_utf8(path)
    // Abre no VFS do Mizu
    let fd = file::open(utf8_path, access, share)
    args[0] = fd as u32  // retorna handle
    0  // STATUS_SUCCESS
  }

  // NtReadFile
  nt_read_file(args: &[u32]) -> u32 {
    let handle = args[0] as i32
    let buf = args[1] as *mut u8
    let len = args[2] as u32

    let count = file::read(handle, buf, len)
    args[3] = count  // bytes lidos
    0
  }

  // NtCreateProcess (cria processo Windows)
  nt_create_process(args: &[u32]) -> u32 {
    let image_name = args[0] as *const u16
    let cmdline = args[1] as *const u16

    // Carrega PE (Portable Executable)
    let pe = pe::load(image_name)
    // Configura espaço de endereçamento (4GB, 2GB user + 2GB kernel)
    vm::win32_address_space(pe.entry)
    // Carrega DLLs necessárias (kernel32.dll, ntdll.dll)
    dll::load_dependencies(pe)
    // Cria thread principal
    thread::create(pe.entry, cmdline)
    0
  }
}

// ============================================================================
// PERSONALITY: DOS (emulação via DOSBox)
// ============================================================================

box personality_dos {
  // Emulador de DOS (VM completa)
  dos_vm: *mut DosVM

  init() {
    dos_vm = dosbox::create_vm()
    dosbox::init_cpu(dos_vm, "8086")
    dosbox::init_memory(dos_vm, 640 * 1024)  // 640KB conventional
    dosbox::init_int_handlers(dos_vm)
  }

  // Interrupção 0x21 (DOS syscall)
  int21_handler(ax: u16, bx: u16, cx: u16, dx: u16) -> u16 {
    case ax >> 8 == 0x09 => {  // print string
      let str_ptr = dx as u32 + (ds as u32 << 4)  // real mode address
      vga::write_str(dos_vm.read_string(str_ptr))
      return 0
    }
    case ax >> 8 == 0x3C => {  // create file
      let filename_ptr = dx as u32 + (ds as u32 << 4)
      let filename = dos_vm.read_string(filename_ptr)
      let fd = file::create(filename, 0o666)
      return fd as u16
    }
    case ax >> 8 == 0x3E => {  // close file
      let fd = bx
      file::close(fd as i32)
      return 0
    }
    else => {
      // fallback: DOSBox emulation
      return dosbox::int21(dos_vm, ax, bx, cx, dx)
    }
  }

  // Executa um programa DOS (.COM ou .EXE)
  run(program: &str) {
    let com_file = file::read_all(program)
    dos_vm.load_program(com_file)
    dos_vm.run()
  }
}

// ============================================================================
// PERSONALITY: VB6 (Virtual Machine para P-code)
// ============================================================================

box personality_vb6 {
  // VM VB6 (interpreta P-code)
  vb6_vm: *mut Vb6VM

  init() {
    vb6_vm = vb6::create_vm()
    vb6::init_runtime(vb6_vm)  // msvbvm60.dll emulado
    vb6::init_controls(vb6_vm)  // Forms, Button, TextBox
  }

  // Executa formulário VB6
  run_form(frm: &str) {
    let pcode = file::read_all(frm)
    vb6_vm.load_form(pcode)
    vb6_vm.run_message_loop()
  }

  // Evento Click de um botão (exemplo)
  event_click(control_id: u32) {
    let event_handler = vb6_vm.get_event_handler(control_id, "Click")
    if event_handler != 0 {
      vb6_vm.call(event_handler)
    }
  }
}

// ============================================================================
// PERSONALITY: .NET (CLR via Mono)
// ============================================================================

box personality_dotnet {
  // CLR embarcado
  clr: *mut MonoVM

  init() {
    clr = mono::create_vm()
    mono::load_framework(clr, "v4.0.30319")
    mono::init_runtime(clr)
  }

  // Executa assembly .NET
  run_assembly(dll_path: &str, class: &str, method: &str, args: &[&str]) {
    let assembly = mono::load_assembly(clr, dll_path)
    let image = mono::get_image(assembly)
    let klass = mono::get_class(image, class)
    let method_desc = mono::get_method(klass, method, args.len())
    mono::invoke_method(clr, method_desc, args)
  }
}

// ============================================================================
// DRIVER FRAMEWORK (unificado)
// ============================================================================

box driver_framework {
  // Tipos de driver
  enum DriverType {
    BLOCK = 1   // disk, SSD, NVMe
    CHAR = 2    // serial, USB, HID
    NET = 3     // ethernet, wifi
    GPU = 4     // VGA, DRM, KMS
    AUDIO = 5   // HDA, USB audio
    INPUT = 6   // keyboard, mouse, touch
  }

  struct Driver {
    name: [u8; 32]
    driver_type: DriverType
    init: fn() -> u32
    read: fn(buf: &mut [u8], offset: u64) -> u32
    write: fn(buf: &[u8], offset: u64) -> u32
    ioctl: fn(cmd: u32, arg: *mut u8) -> u32
    irq_handler: fn(irq: u32)
  }

  driver_table: [Driver; 256]

  // Registra um driver escrito em Hako
  register(driver: &Driver) -> u32 {
    let slot = self::find_free_slot()
    driver_table[slot] = driver
    driver.init()
    slot as u32
  }

  // Forward syscall para o driver correto
  forward_read(major: u32, minor: u32, buf: &mut [u8], offset: u64) -> u32 {
    let drv = driver_table[major as usize]
    drv.read(buf, offset)
  }
}

// ============================================================================
// VFS (Virtual File System) — unificado
// ============================================================================

box vfs {
  struct File {
    path: [u8; 256]
    inode: u64
    driver_major: u32
    driver_minor: u32
    pos: u64
    flags: u32
  }

  file_table: [File; 1024]

  open(path: &str, flags: u32) -> i32 {
    // Descobre qual driver montado nesse path
    let (major, minor) = mount::find_driver(path)
    let fd = self::alloc_fd()
    file_table[fd].path = path
    file_table[fd].driver_major = major
    file_table[fd].driver_minor = minor
    file_table[fd].flags = flags
    fd as i32
  }

  read(fd: i32, buf: &mut [u8], count: u32) -> u32 {
    let f = file_table[fd as usize]
    driver_framework::forward_read(f.driver_major, f.driver_minor, buf, f.pos)
    f.pos = f.pos + count as u64
    count
  }
}

// ============================================================================
// BOOT: Inicialização de todas as personalidades
// ============================================================================

box boot {
  init() {
    // Core microkernel
    mizu_core::init()
    pit::config(1000)
    interrupts::enable()

    // Registra todas as personalidades
    mizu_core::register_personality(Personality::LINUX, &personality_linux)
    mizu_core::register_personality(Personality::WIN32, &personality_win32)
    mizu_core::register_personality(Personality::DOS, &personality_dos)
    mizu_core::register_personality(Personality::VB6, &personality_vb6)
    mizu_core::register_personality(Personality::DOTNET, &personality_dotnet)
    mizu_core::register_personality(Personality::JAVA, &personality_java)
    mizu_core::register_personality(Personality::WASM, &personality_wasm)
    mizu_core::register_personality(Personality::WEB, &personality_web)

    // Monta sistemas de arquivos virtuais
    vfs::mount("/linux", "ext4", "/dev/sda1")
    vfs::mount("/windows", "ntfs", "/dev/sda2")
    vfs::mount("/dos", "fat", "/dos.img")
    vfs::mount("/vb6", "fat", "/vb6.img")

    vga::write_str("Mizu Universal Kernel v1.0\n")
    vga::write_str("Supported ABIs: Linux, Win32, DOS, VB6, .NET, Java, WASM, Web\n")
    vga::write_str("\nmizu> ")

    // Shell que detecta personalidade do executável
    loop {
      let cmd = keyboard::read_line()
      let personality = detect_abi(cmd)
      case personality == Personality::LINUX => {
        linux_loader::load(cmd)
      }
      case personality == Personality::WIN32 => {
        win32_loader::load(cmd)
      }
      case personality == Personality::DOS => {
        dos_loader::load(cmd)
      }
      // ... outros casos
    }
  }

  // Detecta ABI pelo magic number do arquivo
  detect_abi(filename: &str) -> Personality {
    let magic = file::read_u32(filename)
    case magic == 0x7F454C46 => Personality::LINUX      // ELF
    case magic == 0x5A4D => Personality::DOS            // MZ (DOS/Windows)
    case (magic & 0xFFFF) == 0x5A4D => Personality::WIN32 // PE (Windows)
    case magic == 0xCAFEBABE => Personality::JAVA       // Class file
    case magic == 0x6D736100 => Personality::WASM       // WASM
    else => Personality::POSIX
  }
}

// ============================================================================
// FLOW: Execução final
// ============================================================================

flow universal_kernel {
  boot::init
}



Roteiro para o Projeto de Vida (30 anos)
Fase	Período	Objetivo	Marcos
1	2026-2028	Mizu Core	Scheduler, VM, IPC, syscalls próprios ✅
2	2028-2030	Hako madura	Compilador LLVM, stdlib completa
3	2030-2033	Linux personality	100 syscalls, rodar bash, coreutils
4	2033-2036	Win32 personality	200 syscalls, rodar notepad, solitaire
5	2036-2038	DOS + VB6	DOSBox integrado, rodar jogos clássicos
6	2038-2040	.NET + Java	Mono + OpenJDK embarcados
7	2040-2042	WASM + Web	Servidor HTTP, rodar React no kernel
8	2042-2045	Drivers universais	GPU, audio, net via Hako
9	2045-2050	Otimização	Performance comparável a nativo
10	2050+	Manutenção	Aposentar e ver o mundo usar Mizu

El Psy Kongroo
