def perform_test():
    print(21)
    print(22)

returned = perform_test()
assert_eq(None, returned)