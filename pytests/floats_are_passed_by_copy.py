def try_hack(y):
    assert_eq(9.5, y)
    y = 10.5
    return y


x = 9.5
val = try_hack(x)
assert_eq(9.5, x)
assert_eq(10.5, val)