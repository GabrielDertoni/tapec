main:
.loop:
    cpy 'ptr '.cpy_arg     ; Efetivamente dereferencia o ponteiro e coloca o valor em a
    cpy <.cpy_arg> 'a

    ceq &0 'a 'tmp         ; Compara se o resgistrado a possui o valor 0 e coloca o resultado em tmp
    beq 'tmp &'.loop_end   ; Se tmp for 1, termina o loop. Senão, continue
    put 'a                 ; Imprime o conteúdo de a.
    add &1 'ptr 'ptr       ; Incrementa o ponteiro.
    jmp &'.loop            ; Volta ao início do loop.

.loop_end:
    hlt

tmp: 0                     ; Armazena o resultado de comparações.
a: 0                       ; Registrador A.
ptr: 'string               ; Aponta para o próximo caractere a ser impresso.
string: "Hello, world!\n\0"
