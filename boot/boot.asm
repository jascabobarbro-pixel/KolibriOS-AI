; KolibriOS AI - Bare Metal Boot Entry
; x86_64 Multiboot2 compliant bootloader
;
; This file is assembled and linked with the kernel
; to create a bootable disk image.

BITS 64

; Multiboot2 Header
section .multiboot
align 8

MBOOT2_MAGIC        equ 0xE85250D6
MBOOT2_ARCH_I386    equ 0
MBOOT2_HEADER_LEN    equ (multiboot_header_end - multiboot_header_start)

header_start:
    dd MBOOT2_MAGIC                      ; Magic number
    dd MBOOT2_ARCH_I386                 ; Architecture: i386 (32-bit)
    dd MBOOT2_HEADER_LEN                 ; Header length
    dd -(MBOOT2_MAGIC + MBOOT2_ARCH_I386 + MBOOT2_HEADER_LEN)  ; Checksum

    ; Framebuffer tag (optional, for display)
    align 8
framebuffer_tag_start:
    dw 5                                ; type = framebuffer
    dw 0                                ; flags
    dd framebuffer_tag_end - framebuffer_tag_start
    dd 0                                ; mode type (linear graphics)
    dd 1024                             ; width
    dd 768                              ; height
    dd 32                               ; depth
framebuffer_tag_end:

    ; End tag
    dw 0                                ; type
    dw 0                                ; flags
    dd 8                                ; size
multiboot_header_end:

; Text Section
section .text
global _start

extern kernel_main

_start:
    ; Set up stack
    mov rsp, stack_top
    
    ; Clear direction flags
    cld
    
    ; Set up segment registers
    xor ax, ax
    xor bx, bx
    xor cx, cx
    xor dx, dx
    xor si, si
    xor di, di
    
    ; Save multiboot info pointer (passed in ebx)
    mov [rsp + 8], rdi
    mov [rsp + 12], rsi
    mov [rsp + 16], rdx
    mov [rsp + 20], rcx
    mov [rsp + 24], r8
    mov [rsp + 28], r9
    
    ; Initialize serial port for debugging
    ; COM1 port = 0x3F8 (0x3F8)
    ; COM2 port = 0x2F8 (0x2F8)
    mov dx, 0x3F8
    add dx, 0x03
    mov dx, 0x2F8
    add dx, 0x03
    
    ; Print boot message
    mov rsi, boot_msg
    call print_serial
    
    ; Call Rust kernel main
    call kernel_main
    
    ; If we return here, halt
halt_loop:
    hlt
    jmp halt_loop

; Print string to serial port
; Input: RSI = pointer to null-terminated string
print_serial:
    push rsi
    push rdi
    mov dx, 0x2F8        ; COM2 port base
    add dx, 0x03        ; COM2 port = 0x2F8 + 3
.loop:
    lodsb                   ; Load byte from string
    test al, al             ; Check if null terminator
    jz .done
    mov al, al              ; Move char to AL
    out dx, al             ; Send to serial port
    jmp .loop
.done:
    pop rdi
    pop rsi
    ret

; Data Section
section .rodata
boot_msg:
    db 10, 10, 10, 10          ; "####" separator
    db "KolibriOS AI v0.6.0", 10, 10
    db "==================", 10, 10
    db "Initializing kernel...", 10, 10
    db 10, 10, 10, 10
    db 0                          ; Null terminator

; BSS Section (Uninitialized Data)
section .bss
align 16
stack_bottom:
    resb 16384                  ; 16 KB stack
stack_top:
