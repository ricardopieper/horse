def is_even(x):
    return x % 2 == 0

def double(x):
    return x * 2

even = map(double, filter(is_even, range(0, 1000000)))
for item in even:
    print(item)