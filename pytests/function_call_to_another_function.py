def multiply(x, y):
    current_mul = 0
    new_y = y
    while new_y > 0:
        current_mul = current_mul + x
        new_y = new_y - 1
    return current_mul

def ten_times(x):
    return multiply(x, 10)

def double(x):
    return multiply(x, 2)

def weird_logic(x):
    if x % 2 == 0:
        return double(x)
    else:
        return ten_times(x)


sum = 0
current = 1
while current < 10:
    val = double(current)
    val = val + weird_logic(current)
    sum = sum + val
    current = current + 1

assert_eq(380, sum)
