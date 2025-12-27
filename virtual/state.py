from dataclasses import *


class Ready:
    pass


class ReceiveHead:
    pass


@dataclass
class ReceiveData:
    command: int
    data: list = field(default_factory=list)
    escape: bool = False


@dataclass
class Process:
    command: int
    data: list
