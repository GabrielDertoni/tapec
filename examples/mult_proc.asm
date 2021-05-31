
main:
    cal &'proc1

    hlt


proc1:
    put &'P'
    put &'r'
    put &'o'
    put &'c'
    put &' '
    put &'1'
    put &'\n'
    
    cal &'proc2

    put &'P'
    put &'r'
    put &'o'
    put &'c'
    put &' '
    put &'1'
    put &'\n'

    ret


proc2:
    put &'P'
    put &'r'
    put &'o'
    put &'c'
    put &' '
    put &'2'
    put &'\n'
    ret


__tmp: 0
sp: 255

