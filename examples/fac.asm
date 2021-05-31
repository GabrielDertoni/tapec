jmp &'main

n: 7                ; Entrada para fac(n)
res: 0              ; Armazena o resultado de fac(n)

main:
    psh &'mgs_start
    cal &'print_str
    add 'sp &1 'sp

    ptn 'n

    psh &'msg_end
    cal &'print_str
    add 'sp &1 'sp

    psh 'n
    cal &'fac
    add 'sp &1 'sp

    ptn 'res
    put &'\n'

    hlt

fac:
    add 'sp &2 'b   ; 'b aponta para o argumento
    cpy *'b 'b      ; Pega o argumento

    cle 'b &2 'a    ; Se menor que 2, retorna 1
    beq 'a &'.final

    add 'b &-1 'a   ; Armazena n - 1 em 'a
    psh 'a          ; Coloca o argumento para a chamada recursiva

    cal &'fac       ; Chamada recursiva

    pop 'a          ; Tira o argumento 'a do topo da stack

    add 'sp &2 'b   ; 'b aponta para o argumento
    cpy *'b 'b      ; Pega o argumento

    mul 'b 'res 'res
    ret
    
.final:
    cpy &1 'res     ; Retorna 1
    ret

print_str:
    add 'sp &2 'b
    cpy *'b 'ptr

.loop:
    cpy *'ptr 'a
    ceq &0 'a 'b
    beq 'b &'.loop_end
    put 'a
    add &1 'ptr 'ptr
    jmp &'.loop

.loop_end:
    ret


ptr: 0
a: 0
b: 0
__tmp: 0
sp: '__end

mgs_start: "fac(\0"
msg_end:   ") = \0"
