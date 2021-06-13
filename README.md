horse
==========

Horse is a toy implementation of Python in Rust.

Since this is a toy implementation, not every python program will run in it.
However, the goal is to support a good amount of Python functionality.

This interpreter has the following parts:

 - Parser
 - Compiler (AST to bytecode)
 - Virtual Machine


Currently, the following features are supported:

 - Basic syntax: `if`, `else`, `while`
 - Literal syntax for lists. Dictionaries are not supported yet.
 - You can raise exceptions but you can't use `try/except`.
 - Function and class definition with default parameters. This implementation is incomplete: there is no support for inheritance yet, or named parameters.
 - Iterator protocol. Some built-in classes like `list_iterator` and `range` are implemented using the language itself (not a Rust native function). This might be slower, but it is cool :)


If you want a better implementation of Python written in Rust, check out https://github.com/RustPython/RustPython. They even have `pip` working.


Robustness
----------

We have a bunch of regression tests to check if things are still working. However, we do not test for things that shouldn't work, so it is easy to get yourself shot in your own foot.
For instance: default parameters in function declarations must be placed after the normal parameters. If you interleave normal and ddefault parameters, the interpreter will crash.

It's a toy project. I just want things to run.

Is it stable?
-------------

No, any error (like syntax errors and type errors or lookup failures) causes the program to panic, and in some cases it does not report what exactly went wrong. 

However, more and more features of this interpreter will be implemented using the language itself (like the standard library), so eventually this interpreter should report better errors.

About classes
-------------

When a class is declared, we just run the code inside the class declaration. Every declaration (store name)
inside the class definition is declared in the scope of the class object (the type) itself.

Classes are actually functions with bounded properties. 
The class declaration itself is a function that, when called, creates a new `type` PyObject with bounded functions and attributes that can be changed from the outside. In some sense it is kinda like Javascript. 

It also stores a function in the module with the same name as the class. This function creates a new object
of the defined type, and then calls the `__init__` method on that object. 

This strategy is based off Python's strategy.
