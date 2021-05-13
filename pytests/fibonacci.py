def fib(x):
    if x == 0:
        return 0
    
    if x == 1:
        return 1

    fib2 = fib(x - 2)
    fib1 = fib(x - 1)
    return fib2 + fib1

def fib_style2(x):
    if x == 0:
        return 0
    
    if x == 1:
        return 1

    return fib_style2(x - 2) + fib_style2(x - 1)
    
def assert_both_methods_work():
    assert_eq(21, fib(8))
    assert_eq(21, fib_style2(8))

def assert_both_methods_work2(x, y):
    assert_eq(x, fib(y))
    assert_eq(x, fib_style2(y))

assert_both_methods_work()
assert_both_methods_work2(21, 8)