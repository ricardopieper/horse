def is_even(x):
    return x % 2 == 0

some_list = [1,2,3,4,5,6,7,8,9]
for item in filter(is_even, some_list):
    print(item)

materialized = list(filter(is_even, some_list))
print(materialized)
assert_eq([2,4,6,8], materialized)