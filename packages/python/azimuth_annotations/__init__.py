"""Runtime-neutral Azimuth decorators for Python source extraction."""

from collections.abc import Callable
from typing import ParamSpec, TypeVar

P = ParamSpec("P")
R = TypeVar("R")


def _marker(*_values: str) -> Callable[[Callable[P, R]], Callable[P, R]]:
    def decorate(target: Callable[P, R]) -> Callable[P, R]:
        return target

    return decorate


realizes = _marker
implements_check = _marker
implements_mechanism = _marker
