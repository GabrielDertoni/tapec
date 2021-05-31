jmp &'main

n: 10               ; Entrada para fib(n)
res: 0              ; Armazena o resultado de fib(n)

main:
    psh 'n
    cal &'fib
    ptn 'res
    put &'\n'

    hlt

fib:
    add 'sp &2 'b     ; 'b aponta para o argumento
    cpy *'b 'arg      ; Pega o argumento

    cle 'arg &2 'a    ; if ('arg < 2) goto .final;
    beq 'a &'.final

    add 'arg &-1 'a   ; 'a = 'arg - 1

    ; fib('arg - 1)
    psh 'a            ; Coloca o argumento para a chamada recursiva
    cal &'fib         ; Chamada recursiva
    add 'sp &1 'sp    ; Tira o argumento 'a do topo da stack

    add 'sp &2 'b     ; Carrega argumento novamente
    cpy *'b 'arg

    psh 'res          ; Salva 'res na stack

    add 'arg &-2 'a   ; 'a = 'arg - 2

    ; fib('arg - 2)
    psh 'a
    cal &'fib
    add 'sp &1 'sp

    pop 'a            ; 'a = fib('arg - 1)
    add 'a 'res 'res  ; 'res = fib('arg - 1) + fib('arg - 2)
    ret
    
.final:
    cpy 'arg 'res     ; Retorna 'arg
    ret

a: 0
b: 0
arg: 0
__tmp: 0
sp: '__end

