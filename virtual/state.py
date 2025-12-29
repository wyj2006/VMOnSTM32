from dataclasses import *

from command import *


@dataclass
class Ready:
    pass


@dataclass
class ReceiveHead:
    pass


@dataclass
class ReceiveData:
    command: Command
    data: list = field(default_factory=list)
    escape: bool = False


@dataclass
class Process:
    command: Command
    data: list
