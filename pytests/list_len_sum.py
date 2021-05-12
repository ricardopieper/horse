def assert_eq(expected, actual):
    if expected == actual:
        print("Success")
    else:
        panic("Error: expected = "+str(expected)+ " not equal to "+str(actual))

list = [1, 2]
result = len(list) + 1

assert_eq(3, result)