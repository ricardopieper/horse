def try_hack(y):
    y = 10
    return y


x = 9
val = try_hack(x)
assert_eq(9, x)
assert_eq(10, val)