"""Runtime-neutral package markers extracted from source imports."""
from collections.abc import Callable
from typing import TypeVar
T = TypeVar("T")
def implements_probe(identity: str) -> Callable[[T], T]:
    def decorate(target: T) -> T:
        return target
    return decorate
