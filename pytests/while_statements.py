print("Testing simple loop with 1 statement inside")
x = 0
while x < 1000:
    x = x + 1
assert_eq(x, 1000)


print("Testing simple loop setting some more variables")
x = 0
y = 0
z = 0
while x < 1000:
    x = x + 1
    y = y + 1
    z = z + 1

assert_eq(x, 1000)
assert_eq(y, 1000)
assert_eq(z, 1000)



print("Testing break statement up to 5000 instead of 10000")
x = 0
y = 0
while x < 10000:
    y = y + 1
    x = x + 1
    if x == 5000:
        break


assert_eq(x, 5000)
assert_eq(y, 5000)
