main:                   ; Inicia um contexto
.loop:                  ; Label local de um escopo "main"
    cpy 'char_A 'tmp    ; Copia 'A' para a "variável" tmp.
    put 'tmp            ; Imprime o valor de tmp. No caos, 'A'.
    jmp &'.loop         ; Cria um loop infinito. A necessidade desse '&' se
                        ; tornará aparente mais afrente.

outro:
.loop:                  ; Não interfere com outro label .loop.
    cpy 'char_B 'tmp
    put 'tmp
    jmp &'.loop

tmp: 0
char_A: 'A'
char_B: 'B'
