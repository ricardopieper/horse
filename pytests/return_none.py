def perform_test():
    traceback()
    print(21)
    traceback()
    print(22)
    traceback()

returned = perform_test()
assert_eq(None, returned)