main:
    cpy &'.ip1 'ret
    jmp &'procedure
.ip1:
    put 'a
    hlt

procedure:
    cpy &'B' 'a
    jmp 'ret
    put &'C'
    hlt

a: 'A'
ret: 0
