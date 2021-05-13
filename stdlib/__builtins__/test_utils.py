def assert_eq(expected, actual):
    if expected == actual:
        print("Success")
    else:
        panic("Error: expected = "+str(expected)+ " not equal to "+str(actual))