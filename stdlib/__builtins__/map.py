class map:
    def __init__(self, mapping_function, iterable):
        self.mapping_function = mapping_function
        self.iterator = iterable.__iter__()

    def __next__(self):
        next = self.iterator.__next__()
        return self.mapping_function(next)

    def __iter__(self):
        return self