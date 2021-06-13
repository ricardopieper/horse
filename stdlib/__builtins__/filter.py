class filter:
    def __init__(self, mapping_function, iterable):
        self.mapping_function = mapping_function
        self.iterator = iterable.__iter__()

    def __next__(self):
        while True:
            next = self.iterator.__next__()
            if self.mapping_function(next):
                return next

    def __iter__(self):
        return self