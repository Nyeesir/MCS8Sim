; Full reconstruction of MCS8 BIOS/Monitor ROM (Intel 8080)
; Labels: English. Comments: Polish.
; ROM: 0000h-07FFh (2kB), RAM: 800h-0FFFh (2kB).
; HW: 8251 #0 @84h/85h (konsola), 8251 #1 @A4h/A5h (init only), 8253 @88h-8Bh, 8255 @A0h-A3h (nieuzyte w tym ROM).

            ORG 0000h

; --- I/O ports ---
USART0_DATA   EQU 084h
USART0_STAT   EQU 085h
USART1_DATA   EQU 0A4h
USART1_STAT   EQU 0A5h
PPI_PORTA     EQU 0A0h
PPI_PORTB     EQU 0A1h
PPI_PORTC     EQU 0A2h
PPI_CTRL      EQU 0A3h
PIT_CTR0      EQU 088h
PIT_CTR1      EQU 089h
PIT_CTR2      EQU 08Ah
PIT_CTRL      EQU 08Bh

; --- RST vectors (JMP + padding do 8 bajtow) ---
RST0_VECTOR:
    JMP RST0_START_SYSTEM                              ; RST0 - START SYSTEMU
    DB  00h,00h,00h,00h,00h
            ORG 0008h
RST1_VECTOR:
    JMP RST1_PUTCHAR_A                                 ; RST1 - WYDRUK ZNAKU Z A
    DB  00h,00h,00h,00h,00h
            ORG 0010h
RST2_VECTOR:
    JMP RST2_GETCHAR_A                                 ; RST2 - WCZYTANIE ZNAKU DO A
    DB  00h,00h,00h,00h,00h
            ORG 0018h
RST3_VECTOR:
    JMP RST3_PUTS_HL_AT                                ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    DB  00h,00h,00h,00h,00h
            ORG 0020h
RST4_VECTOR:
    JMP RST4_PUTHEX2_A                                 ; RST4 - WYDRUK HEX(A)
    DB  00h,00h,00h,00h,00h
            ORG 0028h
RST5_VECTOR:
    JMP RST5_GETHEX4_DE                                ; RST5 - WCZYTAJ 4 HEX -> DE
    DB  00h,00h,00h,00h,00h
            ORG 0030h
RST6_VECTOR:
    JMP RST6_PRINT_REGS                                ; RST6 - WYDRUK REJESTROW
    DB  00h,00h,00h,00h,00h
            ORG 0038h
RST7_VECTOR:
    JMP 0B00h                                          ; RST7 - PRZERWANIE ZEGAROWE (JMP 0B00h)
    DB  00h,00h,00h,00h,00h

            ORG 0040h
; --- Code (0040h-03FFh) ---

    MVI A,00h
    OUT PIT_CTR1
    MVI A,4Bh
    OUT PIT_CTR1
    JMP 0D00h
RST0_START_SYSTEM:
    MVI A,CEh
    OUT USART0_STAT
    MVI A,CEh
    OUT USART1_STAT
    MVI A,37h
    OUT PIT_CTRL
    MVI A,70h
    OUT PIT_CTRL
    MVI A,26h
    OUT PIT_CTR0
    MVI A,00h
    OUT PIT_CTR0
    LXI HL,0D00h
    MVI M,C9h
    LXI SP,0FFFh
    LXI HL,0409h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    MVI A,40h
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    LXI HL,05AAh
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    LXI HL,06B8h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
L007A:
    RST 2                                                             ; RST2 - WCZYTANIE ZNAKU DO A
    CPI 0Dh
    JNZ L007A
MONITOR_PROMPT:
    LXI HL,0681h                                                      ; prompt/petla monitora
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    LXI HL,06A2h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    LXI HL,06B8h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
L008C:
    MVI A,07h
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    RST 2                                                             ; RST2 - WCZYTANIE ZNAKU DO A
    CPI 0Dh
    JNZ L008C
L0095:
    LXI HL,068Eh
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    RST 2                                                             ; RST2 - WCZYTANIE ZNAKU DO A
    PUSH PSW
    LXI HL,069Eh
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP PSW
    CPI 47h
    JZ CMD_GO
    CPI 53h
    JZ CMD_SETMEM
    CPI 44h
    JZ CMD_DUMP
    CPI 54h
    JZ CMD_TRANSFER
    CPI FFh
    JZ XFER_SEND
    CPI FEh
    JZ XFER_RECEIVE
L00BE:
    MVI A,3Fh
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    LXI SP,0FFFh
    JMP L0095
RST3_PUTS_HL_AT:
    MOV A,M
    CPI 40h
    RZ
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    INX HL
    JMP RST3_PUTS_HL_AT
CMD_GO:
    LXI HL,06A5h                                                      ; komenda monitora
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    RST 5                                                             ; RST5 - WCZYTAJ 4 HEX -> DE
    LXI HL,06B3h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    XCHG
    PCHL
CMD_DUMP:
    RST 5                                                             ; komenda monitora
    XCHG
L00DD:
    MVI D,10h
L00DF:
    PUSH HL
    MVI E,0Fh
    MVI A,0Ah
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    MVI A,0Dh
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    CALL L0165
L00EB:
    MVI A,20h
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    MOV A,M
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    INX HL
    DCR E
    JNZ L00EB
    POP HL
    PUSH HL
    MVI E,0Fh
    MVI A,09h
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
L00FC:
    MOV A,M
    CPI 20h
    JP L0104
L0102:
    MVI A,2Eh
L0104:
    CPI 7Eh
    JP L0102
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    INX HL
    DCR E
    JNZ L00FC
    DCR D
    JNZ L00DF
L0113:
    RST 2                                                             ; RST2 - WCZYTANIE ZNAKU DO A
    CPI 20h
    JZ L00DD
    JMP L0113
CMD_SETMEM:
    RST 5                                                             ; komenda monitora
    XCHG
    PUSH HL
    MVI D,10h
    LXI HL,0698h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    PUSH HL
L0127:
    CALL L0165
    MOV A,M
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    MVI A,0Ah
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    MVI A,0Dh
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    INX HL
    DCR D
    JNZ L0127
    LXI HL,0698h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
L013B:
    POP HL
    CALL L0165
    MOV A,M
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    MVI A,20h
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    RST 5                                                             ; RST5 - WCZYTAJ 4 HEX -> DE
    CPI 0Dh
    JNZ L014B
    MOV M,E
L014B:
    INX HL
    PUSH HL
    LXI HL,06A2h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    JMP L013B
RST4_PUTHEX2_A:
    PUSH BC
    MOV B,A
    RRC
    RRC
    RRC
    RRC
    CALL L0175
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    MOV A,B
    CALL L0175
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    POP BC
    RET
L0165:
    PUSH PSW
    MVI A,23h
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    MOV A,H
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    MOV A,L
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    PUSH HL
    LXI HL,069Eh
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    POP PSW
    RET
L0175:
    ANI 0Fh
    CPI 0Ah
    JM L0181
    SUI 09h
    ORI 40h
    RET
L0181:
    ORI 30h
    RET
RST6_PRINT_REGS:
    PUSH PSW
    PUSH BC
    PUSH DE
    PUSH HL
    CALL L0283
    PUSH HL
    LXI HL,079Ch
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    LXI BC,0007h
    CALL L028B
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    PUSH HL
    LXI HL,079Fh
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    LXI BC,0004h
    CALL L0296
    PUSH HL
    LXI HL,07A4h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    LXI BC,0002h
    CALL L0296
    PUSH HL
    LXI HL,07A9h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    LXI BC,0000h
    CALL L0296
    PUSH HL
    LXI HL,07AEh
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    CALL L02A7
    PUSH HL
    LXI HL,07B3h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    LXI BC,0008h
    CALL L0296
    PUSH HL
    LXI HL,07B8h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    LXI BC,0000h
    CALL L02BC
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    PUSH HL
    LXI HL,07BFh
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    LXI BC,000Ah
    CALL L0296
    PUSH HL
    LXI HL,07C6h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    LXI BC,0006h
    CALL L028B
    ANI 40h
    JZ L01FE
    JMP L0204
L01FE:
    CALL L02CF
    JMP L0207
L0204:
    CALL L02D5
L0207:
    PUSH HL
    LXI HL,07C9h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    LXI BC,0006h
    CALL L028B
    ANI 01h
    JZ L021B
    JMP L0221
L021B:
    CALL L02CF
    JMP L0224
L0221:
    CALL L02D5
L0224:
    PUSH HL
    LXI HL,07CDh
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    LXI BC,0006h
    CALL L028B
    ANI 04h
    JZ L0238
    JMP L023E
L0238:
    CALL L02CF
    JMP L0241
L023E:
    CALL L02D5
L0241:
    PUSH HL
    LXI HL,07D0h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    LXI BC,0006h
    CALL L028B
    ANI 80h
    JZ L0255
    JMP L025B
L0255:
    CALL L02CF
    JMP L025E
L025B:
    CALL L02D5
L025E:
    PUSH HL
    LXI HL,07D3h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    LXI BC,0006h
    CALL L028B
    ANI 10h
    JZ L0272
    JMP L0278
L0272:
    CALL L02CF
    JMP L027B
L0278:
    CALL L02D5
L027B:
    CALL L0287
    POP HL
    POP DE
    POP BC
    POP PSW
    RET
L0283:
    MVI A,FDh
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    RET
L0287:
    MVI A,FCh
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    RET
L028B:
    PUSH DE
    PUSH HL
    LXI HL,0006h
    DAD SP
    DAD BC
    MOV A,M
    POP HL
    POP DE
    RET
L0296:
    PUSH PSW
    PUSH DE
    PUSH HL
    LXI HL,0009h
    DAD SP
    DAD BC
    MOV A,M
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    DCX HL
    MOV A,M
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    POP HL
    POP DE
    POP PSW
    RET
L02A7:
    PUSH PSW
    PUSH DE
    PUSH HL
    LXI HL,0008h
    DAD SP
    DAD BC
    LXI BC,000Ah
    DAD BC
    XCHG
    MOV A,D
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    MOV A,E
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    POP HL
    POP DE
    POP PSW
    RET
L02BC:
    PUSH BC
    PUSH DE
    PUSH HL
    LXI HL,0008h
    DAD SP
    DAD BC
    MOV A,M
    MOV E,A
    INX HL
    MOV A,M
    MOV D,A
    XCHG
    MOV A,M
    POP HL
    POP DE
    POP BC
    RET
L02CF:
    PUSH PSW
    MVI A,2Dh
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    POP PSW
    RET
L02D5:
    PUSH PSW
    MVI A,2Bh
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    POP PSW
    RET
RST1_PUTCHAR_A:
    PUSH PSW
    MVI A,27h
    OUT USART0_STAT
    POP PSW
    OUT USART0_DATA
L02E3:
    IN USART0_STAT
    ANI 01h
    JZ L02E3
    RET
RST2_GETCHAR_A:
    MVI A,27h
    OUT USART0_STAT
L02EF:
    IN USART0_STAT
    ANI 02h
    JZ L02EF
    IN USART0_DATA
    CPI 2Eh
    JZ MONITOR_PROMPT
    RET
RST5_GETHEX4_DE:
    PUSH BC
    PUSH HL
    MVI B,00h
L0302:
    RST 2                                                             ; RST2 - WCZYTANIE ZNAKU DO A
    INR B
    CPI 58h
    JZ L0320
    CPI 20h
    JZ L0320
    CPI 0Dh
    JZ L0320
    CALL L035E
    PUSH PSW
    MOV A,B
    CPI 05h
    JZ L00BE
    JMP L0302
L0320:
    MOV H,A
    MOV A,B
    CPI 01h
    JNZ L032E
    MVI E,00h
L0329:
    MVI D,00h
    JMP L035A
L032E:
    CPI 02h
    JNZ L0340
    POP PSW
    MOV E,A
    JMP L0329
L0338:
    CPI 06h
    JZ L0358
    MOV L,E
    MVI B,06h
L0340:
    POP PSW
    MOV C,A
    POP PSW
    CALL L0372
    ADD C
    MOV E,A
    MOV A,B
    CPI 03h
    JZ L0329
    CPI 04h
    JNZ L0338
    POP PSW
    MOV D,A
    JMP L035A
L0358:
    MOV D,E
    MOV E,L
L035A:
    MOV A,H
    POP HL
    POP BC
    RET
L035E:
    PUSH BC
    MOV C,A
    ANI 40h
    JZ L036D
    MOV A,C
    ANI 0Fh
    ANI 09h
    JMP L0370
L036D:
    MOV A,C
    ANI 0Fh
L0370:
    POP BC
    RET
L0372:
    RLC
    RLC
    RLC
    RLC
    RET
L0377:
    PUSH PSW
    MVI A,27h
    OUT USART0_STAT
    POP PSW
    OUT USART0_DATA
L037F:
    IN USART0_STAT
    ANI 01h
    JZ L037F
    RET
L0387:
    MVI A,27h
    OUT USART0_STAT
L038B:
    IN USART0_STAT
    ANI 02h
    JZ L038B
    IN USART0_DATA
    RET
CMD_TRANSFER:
    LXI HL,06C5h                                                      ; komenda monitora
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    LXI HL,069Eh
L039C:
    RST 2                                                             ; RST2 - WCZYTANIE ZNAKU DO A
    CPI 57h
    JZ XFER_SEND
    CPI 50h
    JZ XFER_RECEIVE
    MVI A,3Fh
    RST 1                                                             ; RST1 - WYDRUK ZNAKU Z A
    JMP L039C
XFER_RECEIVE:
    LXI HL,0736h                                                      ; komenda monitora
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    CALL L0387
    MOV C,A
    CALL L0387
    MOV B,A
    CALL L0387
    MOV L,A
    CALL L0387
    MOV H,A
L03C1:
    CALL L0387
    MOV M,A
    INX HL
    DCX BC
    MOV A,C
    ORA B
    JZ L03CF
    JMP L03C1
L03CF:
    PUSH HL
    LXI HL,071Bh
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    POP HL
    MOV A,H
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    MOV A,L
    RST 4                                                             ; RST4 - WYDRUK HEX(A)
    LXI HL,0718h
    RST 3 					                                          ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    LXI HL,06B8h
    RST 3                                                             ; RST3 - WYDRUK LANCUCHA [HL]..'@'
    RST 2                                                             ; RST2 - WCZYTANIE ZNAKU DO A
    JMP MONITOR_PROMPT
XFER_SEND:
    PUSH DE                                                           ; komenda monitora
    PUSH HL
    PUSH PSW
    CALL L0387
    MOV L,A
    CALL L0387
    MOV H,A
    CALL L0387
    MOV E,A
    CALL L0387
    MOV D,A
    MOV A,M
    CALL L0377
    DCX DE
    INX HL
    MOV A,E
    ORA D

            ORG 0400h
; --- Data / screen texts (0400h-07FFh) ---
; RST3 drukuje od [HL] do terminatora '@' (40h). Dzielimy tez po CRLF.

    DB C2h, F8h, 03h, F1h, E1h, D1h, C3h, 80h, 00h, 1Bh, 48h, 1Bh, 4Ah, 0Dh, 0Ah    ; \xC2\xF8\x03\xF1\xE1\xD1\xC3\x80\0\eH\eJ\r\n
    DB 09h, 09h, 09h, 4Dh, 20h, 43h, 20h, 53h, 20h, 2Dh, 20h, 38h, 2Eh, 32h, 09h, 4Dh    ; \t\t\tM C S - 8.2\tM
    DB 20h, 45h, 20h, 4Eh, 20h, 55h, 0Dh, 0Ah    ;  E N U\r\n
    DB 0Ah, 41h, 44h, 52h, 45h, 53h, 59h, 20h, 57h, 45h, 2Fh, 57h, 59h, 3Ah, 09h, 09h    ; \nADRESY WE/WY:\t\t
    DB 09h, 09h, 44h, 59h, 52h, 45h, 4Bh, 54h, 59h, 57h, 59h, 3Ah, 0Dh, 0Ah    ; \t\tDYREKTYWY:\r\n
    DB 0Ah, 55h, 38h, 32h, 35h, 31h, 20h, 38h, 34h, 48h, 09h, 09h, 09h, 09h, 53h, 2Dh    ; \nU8251 84H\t\t\t\tS-
    DB 5Ah, 4Dh, 49h, 41h, 4Eh, 41h, 20h, 5Ah, 41h, 57h, 41h, 52h, 54h, 4Fh, 53h, 43h    ; ZMIANA ZAWARTOSC
    DB 49h, 20h, 50h, 41h, 4Dh, 49h, 45h, 43h, 49h, 0Dh, 0Ah    ; I PAMIECI\r\n
    DB 55h, 38h, 32h, 35h, 31h, 20h, 30h, 41h, 34h, 68h, 09h, 09h, 09h, 09h, 44h, 2Dh    ; U8251 0A4h\t\t\t\tD-
    DB 57h, 59h, 53h, 57h, 49h, 45h, 54h, 4Ch, 45h, 4Eh, 49h, 45h, 20h, 5Ah, 41h, 57h    ; WYSWIETLENIE ZAW
    DB 41h, 52h, 54h, 4Fh, 53h, 43h, 49h, 20h, 50h, 41h, 4Dh, 49h, 45h, 43h, 49h, 0Dh    ; ARTOSCI PAMIECI\r
    DB 0Ah, 55h, 38h, 32h, 35h, 35h, 20h, 30h, 41h, 30h, 68h, 09h, 09h, 09h, 09h, 47h    ; \nU8255 0A0h\t\t\t\tG
    DB 2Dh, 53h, 54h, 41h, 52h, 54h, 20h, 50h, 52h, 4Fh, 47h, 52h, 41h, 4Dh, 55h, 0Dh    ; -START PROGRAMU\r
    DB 0Ah, 55h, 38h, 32h, 35h, 33h, 20h, 38h, 38h, 48h, 09h, 09h, 09h, 09h, 54h, 2Dh    ; \nU8253 88H\t\t\t\tT-
    DB 50h, 52h, 4Fh, 47h, 52h, 41h, 4Dh, 20h, 54h, 52h, 41h, 4Eh, 53h, 4Dh, 49h, 53h    ; PROGRAM TRANSMIS
    DB 4Ah, 49h, 0Dh, 0Ah    ; JI\r\n
    DB 0Ah, 52h, 45h, 53h, 54h, 41h, 52h, 54h, 59h, 3Ah, 09h, 09h, 09h, 09h, 53h, 54h    ; \nRESTARTY:\t\t\t\tST
    DB 4Fh, 53h, 3Ah, 20h, 30h, 30h, 46h, 46h, 46h, 68h, 0Dh, 0Ah    ; OS: 00FFFh\r\n
    DB 52h, 53h, 54h, 20h, 30h, 20h, 2Dh, 20h, 53h, 54h, 41h, 52h, 54h, 20h, 53h, 59h    ; RST 0 - START SY
    DB 53h, 54h, 45h, 4Dh, 55h, 0Dh, 0Ah    ; STEMU\r\n
    DB 52h, 53h, 54h, 20h, 31h, 20h, 2Dh, 20h, 57h, 59h, 44h, 52h, 55h, 4Bh, 20h, 5Ah    ; RST 1 - WYDRUK Z
    DB 4Eh, 41h, 4Bh, 55h, 20h, 5Ah, 20h, 41h, 4Bh, 55h, 4Dh, 55h, 4Ch, 41h, 54h, 4Fh    ; NAKU Z AKUMULATO
    DB 52h, 41h, 20h, 4Eh, 41h, 20h, 4Dh, 4Fh, 4Eh, 49h, 54h, 4Fh, 52h, 0Dh, 0Ah    ; RA NA MONITOR\r\n
    DB 52h, 53h, 54h, 20h, 32h, 20h, 2Dh, 20h, 57h, 43h, 5Ah, 59h, 54h, 41h, 4Eh, 49h    ; RST 2 - WCZYTANI
    DB 45h, 20h, 5Ah, 4Eh, 41h, 4Bh, 55h, 20h, 5Ah, 20h, 4Bh, 4Ch, 41h, 57h, 49h, 41h    ; E ZNAKU Z KLAWIA
    DB 54h, 55h, 52h, 59h, 20h, 44h, 4Fh, 20h, 41h, 4Bh, 55h, 4Dh, 55h, 4Ch, 41h, 54h    ; TURY DO AKUMULAT
    DB 4Fh, 52h, 41h, 0Dh, 0Ah    ; ORA\r\n
    DB 52h, 53h, 54h, 20h, 33h, 20h, 2Dh, 20h, 57h, 59h, 44h, 52h, 55h, 4Bh, 20h, 4Ch    ; RST 3 - WYDRUK L
    DB 41h, 4Eh, 43h, 55h, 43h, 48h, 41h, 20h, 5Ah, 20h, 50h, 41h, 4Dh, 49h, 45h, 43h    ; ANCUCHA Z PAMIEC
    DB 49h, 20h, 4Fh, 44h, 20h, 28h, 48h, 4Ch, 29h, 20h, 44h, 4Fh, 20h, 22h, 40h    ; I OD (HL) DO "@
    DB 22h, 0Dh, 0Ah    ; "\r\n
    DB 52h, 53h, 54h, 20h, 34h, 20h, 2Dh, 20h, 57h, 59h, 44h, 52h, 55h, 4Bh, 20h, 44h    ; RST 4 - WYDRUK D
    DB 57h, 4Fh, 43h, 48h, 20h, 5Ah, 4Eh, 41h, 4Bh, 4Fh, 57h, 20h, 48h, 45h, 4Bh, 53h    ; WOCH ZNAKOW HEKS
    DB 41h, 44h, 45h, 43h, 59h, 4Dh, 41h, 4Ch, 4Eh, 59h, 43h, 48h, 20h, 5Ah, 20h, 41h    ; ADECYMALNYCH Z A
    DB 4Bh, 55h, 4Dh, 55h, 4Ch, 41h, 54h, 4Fh, 52h, 41h, 0Dh, 0Ah    ; KUMULATORA\r\n
    DB 52h, 53h, 54h, 20h, 35h, 20h, 2Dh, 20h, 57h, 43h, 5Ah, 59h, 54h, 41h, 4Eh, 49h    ; RST 5 - WCZYTANI
    DB 45h, 20h, 43h, 5Ah, 54h, 45h, 52h, 30h, 45h, 43h, 68h, 20h, 5Ah, 4Eh, 41h, 4Bh    ; E CZTER0ECh ZNAK
    DB 4Fh, 57h, 20h, 48h, 45h, 4Bh, 53h, 41h, 44h, 45h, 43h, 59h, 4Dh, 41h, 4Ch, 4Eh    ; OW HEKSADECYMALN
    DB 59h, 43h, 48h, 20h, 44h, 4Fh, 20h, 52h, 45h, 4Ah, 45h, 53h, 54h, 52h, 4Fh, 57h    ; YCH DO REJESTROW
    DB 0Dh, 0Ah    ; \r\n
    DB 52h, 53h, 54h, 20h, 36h, 20h, 2Dh, 20h, 57h, 59h, 4Bh, 4Fh, 4Eh, 41h, 4Eh, 49h    ; RST 6 - WYKONANI
    DB 45h, 20h, 50h, 52h, 4Fh, 47h, 52h, 41h, 4Dh, 55h, 20h, 5Ah, 20h, 50h, 41h, 4Dh    ; E PROGRAMU Z PAM
    DB 49h, 45h, 43h, 49h, 20h, 52h, 41h, 4Dh, 20h, 28h, 4Ah, 4Dh, 50h, 20h, 38h, 30h    ; IECI RAM (JMP 80
    DB 30h, 48h, 29h, 0Dh, 0Ah    ; 0H)\r\n
    DB 52h, 53h, 54h, 20h, 37h, 20h, 2Dh, 20h, 00h, 50h, 52h, 5Ah, 45h, 52h, 57h, 41h    ; RST 7 - \0PRZERWA
    DB 4Eh, 49h, 45h, 20h, 5Ah, 45h, 47h, 41h, 52h, 4Fh, 57h, 45h, 1Bh, 59h, 36h, 5Bh    ; NIE ZEGAROWE\eY6[
    DB 40h    ; @
    DB 0Dh, 1Bh, 48h, 0Eh, 18h, 4Dh, 43h, 53h, 2Dh, 38h, 1Bh, 4Ah, 40h    ; \r\eH\x0E\x18MCS-8\eJ@
    DB 11h, 1Bh, 0Ah, 0Ah, 0Dh, 23h, 40h    ; \x11\e\n\n\r#@
    DB 0Ah, 09h, 40h    ; \n\t@
    DB 1Bh, 48h, 0Ah, 0Ah, 0Dh, 40h    ; \eH\n\n\r@
    DB 20h, 3Eh, 20h, 40h    ;  > @
    DB 0Ah, 0Dh, 40h    ; \n\r@
    DB 1Bh, 48h, 1Bh, 4Ah, 0Ah, 0Dh, 23h, 47h, 20h, 50h, 43h, 3Dh, 20h, 40h    ; \eH\eJ\n\r#G PC= @
    DB 1Bh, 48h, 1Bh, 4Ah, 40h    ; \eH\eJ@
    DB 4Eh, 41h, 43h, 49h, 53h, 4Eh, 49h, 4Ah, 00h, 2Dh, 43h, 52h, 40h    ; NACISNIJ\0-CR@
    DB 1Bh, 48h, 1Bh, 4Ah, 09h, 09h, 50h, 52h, 4Fh, 47h, 52h, 41h, 4Dh, 20h, 20h, 4Bh    ; \eH\eJ\t\tPROGRAM  K
    DB 4Fh, 4Dh, 55h, 4Eh, 49h, 4Bh, 41h, 43h, 59h, 4Ah, 4Eh, 59h, 0Dh, 0Ah    ; OMUNIKACYJNY\r\n
    DB 0Dh, 0Ah    ; \r\n
    DB 57h, 20h, 2Dh, 20h, 57h, 59h, 53h, 4Ch, 49h, 4Ah, 20h, 44h, 41h, 4Eh, 45h, 0Dh    ; W - WYSLIJ DANE\r
    DB 0Ah, 50h, 20h, 2Dh, 20h, 50h, 52h, 5Ah, 59h, 4Ah, 4Dh, 49h, 4Ah, 20h, 44h, 41h    ; \nP - PRZYJMIJ DA
    DB 4Eh, 45h, 0Dh, 0Ah    ; NE\r\n
    DB 2Eh, 20h, 2Dh, 20h, 57h, 59h, 4Ah, 53h, 43h, 49h, 45h, 0Dh, 0Ah    ; . - WYJSCIE\r\n
    DB 0Ah, 40h    ; \n@
    DB 0Ah, 0Ah, 09h, 0Ah, 0Dh, 50h, 41h, 4Dh, 49h, 45h, 43h, 20h, 5Ah, 41h, 4Ah, 45h    ; \n\n\t\n\rPAMIEC ZAJE
    DB 54h, 41h, 20h, 44h, 4Fh, 20h, 41h, 44h, 52h, 45h, 53h, 55h, 20h, 40h    ; TA DO ADRESU @
    DB 1Bh, 48h, 1Bh, 4Ah, 4Dh, 43h, 53h, 2Dh, 47h, 4Fh, 54h, 4Fh, 57h, 59h, 20h, 07h    ; \eH\eJMCS-GOTOWY \a
    DB 44h, 4Fh, 20h, 4Fh, 44h, 42h, 49h, 4Fh, 52h, 55h, 0Dh, 0Ah    ; DO ODBIORU\r\n
    DB 40h    ; @
    DB 1Bh, 48h, 1Bh, 4Ah, 50h, 4Fh, 44h, 41h, 4Ah, 20h, 41h, 44h, 52h, 45h, 53h, 20h    ; \eH\eJPODAJ ADRES 
    DB 50h, 49h, 45h, 52h, 57h, 53h, 5Ah, 45h, 47h, 4Fh, 20h, 42h, 41h, 4Ah, 54h, 55h    ; PIERWSZEGO BAJTU
    DB 20h, 44h, 41h, 4Eh, 59h, 43h, 48h, 20h, 40h    ;  DANYCH @
    DB 0Ah, 0Dh, 0Dh, 50h, 4Fh, 44h, 41h, 4Ah, 20h, 44h, 4Ch, 55h, 47h, 4Fh, 53h, 43h    ; \n\r\rPODAJ DLUGOSC
    DB 20h, 42h, 4Ch, 4Fh, 4Bh, 55h, 20h, 44h, 41h, 4Eh, 59h, 43h, 48h, 20h, 40h    ;  BLOKU DANYCH @
    DB 00h, 41h, 3Dh, 40h    ; \0A=@
    DB 20h, 42h, 43h, 3Dh, 40h    ;  BC=@
    DB 20h, 44h, 45h, 3Dh, 40h    ;  DE=@
    DB 20h, 48h, 4Ch, 3Dh, 40h    ;  HL=@
    DB 20h, 53h, 50h, 3Dh, 40h    ;  SP=@
    DB 20h, 50h, 43h, 3Dh, 40h    ;  PC=@
    DB 20h, 28h, 48h, 4Ch, 29h, 3Dh, 40h    ;  (HL)=@
    DB 20h, 28h, 53h, 50h, 29h, 3Dh, 40h    ;  (SP)=@
    DB 20h, 5Ah, 40h    ;  Z@
    DB 20h, 43h, 59h, 40h    ;  CY@
    DB 20h, 50h, 40h    ;  P@
    DB 20h, 53h, 40h    ;  S@
    DB 20h, 41h, 43h, 40h    ;  AC@
    DB 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h    ; \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0
    DB 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h    ; \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0
    DB 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h, 00h    ; \0\0\0\0\0\0\0\0\0