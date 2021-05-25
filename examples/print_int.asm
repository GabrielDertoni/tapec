main:
.loop:
    cle &100 'i 'tmp      ; while (i < 100) {
    beq 'tmp &'.loop_end
    ptn 'i                ;     printf("%d%%\n", i);
    put &'%'
    put &'\n'
    add &1 'i 'i          ;     i++;
    jmp &'.loop
.loop_end:                ; }
    hlt

tmp: 0
i: 0
