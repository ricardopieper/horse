def double(x):
    return x * 2

def weird_logic(x):
    if x % 2 == 0:
        return x * 2
    else:
        return x * 10


sum = 0
current = 1
while current < 1000:
    val = double(current)
    val = val + weird_logic(current)
    sum = sum + val
    current = current + 1
    printstack()

assert_eq(3998000, sum)
assert_eq(1000, current)
