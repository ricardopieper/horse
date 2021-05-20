class SomeClass:
    def __init__(self):
        self.x = 2

    def get_num(self, a):
        return self.x * a + 10

instance = SomeClass()
result = instance.get_num(2)
print(result)
print(2 * 2 + 10)