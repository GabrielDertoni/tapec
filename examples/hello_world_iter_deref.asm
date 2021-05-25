
main:
    cpy &'string 'ptr

.loop:
    cpy *'ptr 'a
    ceq &0 'a 'tmp
    beq 'tmp &'.loop_end
    put 'a
    add &1 'ptr 'ptr
    jmp &'.loop

.loop_end:
    hlt

tmp: 0
a: 0
ptr: 0
string: "Hello, world!\n\0"

