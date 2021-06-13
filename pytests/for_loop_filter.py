def is_even(x):
    return x % 2 == 0

some_list = [1,2,3,4,5,6,7,8,9]
even = filter(is_even, some_list)
for item in even:
    print(item)