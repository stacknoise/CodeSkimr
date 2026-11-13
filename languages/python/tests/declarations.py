"""Module docstring."""

import os

MAX_SIZE = 10
_cache = {}


class Shape(Base):
    """A shape.

    With a second paragraph.
    """

    sides: int = 0
    label = "shape"
    _hidden = 1
    __secret = 2

    def __init__(self, sides):
        """Create a shape."""
        self.sides = sides

    def area(self):
        """Return the area."""
        return 0

    def _helper(self):
        pass

    def __mangled(self):
        pass

    def __repr__(self):
        return "Shape"

    @staticmethod
    def create():
        """Make one."""
        return Shape(0)

    @classmethod
    def from_text(cls, text):
        return cls(len(text))

    @property
    def name(self):
        """The name."""
        return self.label

    @name.setter
    def name(self, value):
        self.label = value

    class Inner:
        def method(self):
            def local():
                return 1

            return local()


def top_level(a, b=1):
    """Top level function."""

    def nested():
        return a

    return nested()


async def fetch(url):
    return url


@decorator(arg=1)
def decorated():
    pass


if os.name == "nt":

    def platform():
        pass


def property(value):
    """A plain function that is named like a decorator."""

    def _inner():
        return value

    return _inner


def __init__():
    pass
