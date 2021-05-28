
main:
    psh &'\0'
    psh &'\n'
    psh &'d'
    psh &'l'
    psh &'r'
    psh &'o'
    psh &'w'
    psh &' '
    psh &','
    psh &'o'
    psh &'l'
    psh &'l'
    psh &'e'
    psh &'h'

.loop:
    pop 'a
    ceq 'a &'\0' 'b
    beq 'b &'hlt
    put 'a
    jmp &'.loop

hlt:
    hlt


a: 0
b: 0
sp: 255
