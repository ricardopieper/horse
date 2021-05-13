print("Computing the # of numbers divisible by 5")

x = 0
y = 0
mod5 = 0
while x < 10000:
    y = y + 1
    x = x + 1
    if x % 5 == 0:
        mod5 = mod5 + 1

assert_eq(2000, mod5)