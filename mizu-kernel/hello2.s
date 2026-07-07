.intel_syntax noprefix
.globl _start
_start:
    # pipe(pipefds)
    sub esp, 8
    lea ebx, [esp]
    mov eax, 42         # SYS_PIPE
    int 0x80

    # Check pipe success (eax should be 0)
    test eax, eax
    jnz fail

    mov edx, [esp]      # edx = read_fd
    mov ecx, [esp+4]    # ecx = write_fd

    # fork
    mov eax, 2          # SYS_FORK
    int 0x80

    test eax, eax
    jz child
    jmp parent

child:
    # Child: write "hello" to pipe
    mov ebx, ecx        # fd = write_fd
    lea ecx, [msg]
    mov edx, 5          # len
    mov eax, 4          # SYS_WRITE
    int 0x80

    # exit(0)
    mov eax, 1
    xor ebx, ebx
    int 0x80

parent:
    # Parent: save child pid
    push eax

    # Read from pipe
    mov ebx, edx        # fd = read_fd
    lea ecx, [buf]
    mov edx, 16
    mov eax, 3          # SYS_READ
    int 0x80

    # Write to stdout
    mov ebx, 1
    lea ecx, [buf]
    mov edx, 16
    mov eax, 4          # SYS_WRITE
    int 0x80

    # Wait for child
    pop ebx             # child pid
    lea ecx, [esp+4]    # status
    xor edx, edx        # options
    mov eax, 7          # SYS_WAITPID
    int 0x80

    # Write "ok\n"
    mov ebx, 1
    lea ecx, [okmsg]
    mov edx, 3
    mov eax, 4
    int 0x80

    # exit(0)
    mov eax, 1
    xor ebx, ebx
    int 0x80

fail:
    # Write "fail\n"
    mov ebx, 1
    lea ecx, [failmsg]
    mov edx, 5
    mov eax, 4
    int 0x80

    mov eax, 1
    mov ebx, 1
    int 0x80

msg:    .ascii "hello"
okmsg:  .ascii "ok\n"
failmsg: .ascii "fail\n"
.bss
buf:    .space 16
