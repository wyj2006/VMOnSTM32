class Memory:
    def __init__(self):
        self.data = {}

    def read(self, address: int):
        return self.data.setdefault(address, 0)

    def write(self, address: int, value: int):
        self.data[address] = value
