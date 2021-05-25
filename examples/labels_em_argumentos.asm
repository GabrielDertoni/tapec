main:
    cpy 'ptr '.arg           ; Copia o valor de ponteiro para .arg.
    put <.arg>               ; Agora .arg possui 'string e pode imprimir o 'H'.

ptr: 'string                 ; Ponteiro para o endereço de string.
string: "Hello, World\n\0"
