slowpython
==========

This is a toy implementation of Python.

For now, the only Python-y thing it has is the runtime system, where I'm trying to implement something
similar to the Python data model.

So far, I only parse and execute expressions, like `1 + sin(cos(9.3)) / 2 + 2 * 4 / (5 / 1.2)`.
In actual, real-life, production-ready Python, it would be `math.sin` but I don't like it, 
so I put these functions in the `__builtin__` module. There is no method accessor syntax as of now.

The implemented types as of now are `NoneType`, `NotImplemented`, `int` and `float`.

The plan is to be able to run a reasonable Python program, therefore support for strings, 
classes, etc may come in the future.

The expected performance is to be even slower than Python. It will be enough if it works and computes 
stuff correctly.

Architecture
------------

This project implements the usual lexer -> parser -> interpreter architecture. However, it currently
also implements a bytecode and bytecode interpreter instead of executing the AST directly.

A small compiler produces the slowpython bytecode in `bytecode/compiler.rs`. This could possibly be serialized and run. 
The interpreter runtime in `runtime.rs` implements the data model, function calls, modules and such. This is not JIT-compiled to x86, 
it is interpreted right away.

I try to follow the python bytecode specification, but I do some stuff differently. For instance: I call the binary functions
for add, subtract, multiply and divide directly like `__add__`, `__subtract__`, etc. It also implements everything as an object.

    - Numbers are `PyObject` of type `int` or `float`
    - Functions are `PyObject` of type `function`
    - Built-in methods in numbers are `PyObject` of type `function`, but they are bounded to a `PyObject` instance

Does it work?
-------------

There are 77 tests so far, they test almost everything and they all pass. 

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

This is an example of the bytecode for the expression `cos(sin(-(5.0 / 9.0) * 32.0)) / tanh(cos(1.0) - (5.0 / 9.0))`:

    LoadFunction("cos")
    LoadFunction("sin")
    LoadConst(Float(5.0))
    LoadMethod("__truediv__")
    LoadConst(Float(9.0))
    CallMethod { number_arguments: 1 }
    LoadMethod("__neg__")
    CallMethod { number_arguments: 0 }
    LoadMethod("__mul__")
    LoadConst(Float(32.0))
    CallMethod { number_arguments: 1 }
    CallFunction { number_arguments: 1 }
    CallFunction { number_arguments: 1 }
    LoadMethod("__truediv__")
    LoadFunction("tanh")
    LoadFunction("cos")
    LoadConst(Float(1.0))
    CallFunction { number_arguments: 1 }
    LoadMethod("__sub__")
    LoadConst(Float(5.0))
    LoadMethod("__truediv__")
    LoadConst(Float(9.0))
    CallMethod { number_arguments: 1 }
    CallMethod { number_arguments: 1 }
    CallFunction { number_arguments: 1 }
    CallMethod { number_arguments: 1 }
