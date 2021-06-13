def double(x):
    return x * 2

some_list = [1,2,3,4,5]
doubled = map(double, some_list)
for item in doubled:
    print(item)