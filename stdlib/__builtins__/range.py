class range:
    def __init__(self, max):
        self.current = 0
        self.max = max

    def __next__(self):
        if self.current >= self.max:
            raise StopIteration
        else:
            cur = self.current
            self.current = self.current + 1
            return cur

    def __iter__(self):
        return self