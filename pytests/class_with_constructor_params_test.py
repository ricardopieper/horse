class SomeClass:
    def __init__(self, val):
        self.x = val

    def get_num(self, a):
        return self.x * a + 10

instance = SomeClass(10)
result = instance.get_num(2)
print(result)
assert_eq(30, result)