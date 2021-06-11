class range:
    def __init__(self, param1, param2 = None):
        if param2 == None:
            self.current = 0
            self.max = param1
        else:
            self.current = param1
            self.max = param2

    def __next__(self):
        if self.current >= self.max:
            raise StopIteration
        else:
            cur = self.current
            self.current = self.current + 1
            return cur

    def __iter__(self):
        return self