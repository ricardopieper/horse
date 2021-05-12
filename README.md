slowpython
==========

This is a toy implementation of Python in Rust.

Currently, stuff like this should work:

    def double(x):
        return x * 2

    sum = 0
    current = 1
    while current < 10:
        val = double(current)
        print(val)
        sum = sum + val
        current = current + 1

    print("sum should be 90, is " + str(sum))

    proof = 2 + 4 + 6 + 8 + 10 + 12 + 14 + 16 + 18
    print("proof: "+str(proof))

The math functions in the `math` module were added to the `__builtins__` module for convenience. There is also a 
`print` function, which takes 1 argument.

The implemented types as of now are `NoneType`, `NotImplemented`, `int` and `float`, `bool` and `str`. They are not complete.

The plan is to be able to run a reasonable Python program. It still needs to suport list operations, maps, sets, literal sintaxes, etc.

The expected performance is to be even slower than Python. It will be enough if it works and computes stuff correctly. However I spent some time
optimizing some stuff.

Architecture
------------

This project implements the usual lexer -> parser -> interpreter architecture. However, it currently
also implements a bytecode and bytecode interpreter instead of executing the AST directly (tree).

A small compiler produces the slowpython bytecode in `bytecode/compiler.rs`. This could possibly be serialized and run. 
The interpreter runtime in `runtime.rs` implements the data model, function calls, modules and such. This is not JIT-compiled to x86, 
it is interpreted right away.

I try to follow the python bytecode specification, but I do some stuff differently depending on the current limitations of this interpreter.
There are optimizations for common arithmetic operations (like + - * / %) which fall back to calling the `__add__`, `__mul__`, etc methods 
if the types aren't compatible (ex: string + string calls the `__add__` method).

Does it work?
-------------

There are a bunch of tests so far, they test almost everything and they all pass. 

Is it stable?
-------------

No, any error (like syntax errors and type errors or lookup failures) causes the program to panic, 
and in some cases it does not report what exactly went wrong. There is no support for exceptions.

Is the code any good?
---------------------

Expect a lot of duplicated code while I'm figuring out the data model. There is some `unsafe` and `UnsafeCell`
in some locations, which is not ideal, but Python doesn't care about data races :p it doesn't affect the state
of the interpreter though, just the user data in memory.

When things are more figured out, the code in the interpreter will probably shrink significantly.

Also, some stuff may not be idiomatic Rust.

Bytecode
--------

The bytecode is based on python's own bytecode. However we won't be able to load a python `.pyc` bytecode file and just run it. If this interpreter ever
saves a .pyc file, it would also not be loadable by cpython interpreter. The .pyc bytecode doen't really seem designed to be compatible across python versions, so I don't bother either.

    x = 0
    y = 0
    mod5 = 0
    while x < 900000:
        y = y + 1
        x = x + 1
        if x % 5 == 0:
            mod5 = mod5 + 1
    print(mod5)

The bytecode for the code above is:

    LoadConst(0)
    StoreName(0)
    LoadConst(0)
    StoreName(1)
    LoadConst(0)
    StoreName(2)
    LoadName(0)
    LoadConst(1)
    CompareLessThan
    JumpIfFalseAndPopStack(29)
    LoadName(1)
    LoadConst(2)
    BinaryAdd
    StoreName(1)
    LoadName(0)
    LoadConst(2)
    BinaryAdd
    StoreName(0)
    LoadName(0)
    LoadConst(3)
    BinaryModulus
    LoadConst(0)
    CompareEquals
    JumpIfFalseAndPopStack(28)
    LoadName(2)
    LoadConst(2)
    BinaryAdd
    StoreName(2)
    JumpUnconditional(6)
    LoadFunction("print")
    LoadName(2)
    CallFunction { number_arguments: 1 }

We store consts in a "data" section of the compiled program as well:

    pub struct Program {
        pub version: u64,
        pub data: Vec<Const>,
        pub code: Vec<Instruction>,
    }
