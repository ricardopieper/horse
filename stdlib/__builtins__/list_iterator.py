class list_iterator:
    def __init__(self, list):
        self.list = list
        self.index_cur = 0

    def __next__(self):
        if self.index_cur >= len(self.list):
            raise StopIteration
        else:
            result = self.list[self.index_cur]
            self.index_cur = self.index_cur + 1
            return result