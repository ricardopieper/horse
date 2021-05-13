def try_hack(y):
    assert_eq("abc", y)
    y = "def"
    return y


x = "abc"
val = try_hack(x)
assert_eq("abc", x)
assert_eq("def", val)