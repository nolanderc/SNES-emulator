.include "Header.inc"
; .include "SnesInit.asm"

VBlank:
    RTI

.bank 0
.section "MainCode"

Start:
    ; Snes_Init
 	sei 	 	; Disabled interrupts
 	clc 	 	; clear carry to switch to native mode
 	xce 	 	; Xchange carry & emulation bit. native mode

Forever:
    jml Later

.ends

.bank 1
.org 0
Later:
    jml Forever
