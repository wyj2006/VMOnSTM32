class Memory:
    def __init__(self):
        self.data = []

    def read(self, address: int):
        while address >= len(self.data):
            self.data.append(0)
        return self.data[address]

    def write(self, address: int, value: int):
        while address >= len(self.data):
            self.data.append(0)
        self.data[address] = value
