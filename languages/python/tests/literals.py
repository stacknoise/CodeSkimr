"""Cases of the no-literals rule (ADR 0058): one function per case."""


def clean(items):
    """A docstring is no literal."""
    total = 0
    for item in items:
        total += 1
    return total + item.count


def numbers():
    return 42 + 0 + 1 + 2.5


def strings(name):
    label = "x" + 'y'
    return f"{name}!" + label


def annotated(first: "Order", second: int = 7) -> "Order":
    return first


def flags():
    return True or False or None


class Holder:
    FIELD = 99

    def method(self):
        """Doc."""
        return [1, 2, 3]
