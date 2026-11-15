# Fixture for the cognitive complexity and parameter counts (ADR 0048). The expected values are in
# crates/adapters/parsing/tests/python_metrics.rs.


def straight(a):
    return a + 1


def nested(a, b):
    if a > 0:
        for _ in range(b):
            if a > b:
                return 1
    return 0


def chain(a):
    if a == 1:
        return 1
    elif a == 2:
        return 2
    else:
        return 3


def runs(a, b, c, d):
    return a and b and c or d


def handler(x):
    try:
        return int(x)
    except ValueError:
        if x:
            return 1
    return 0


def lambdas(a):
    f = lambda x: x if x else a
    return f(a)


class Shop:
    def method(self, a, b: int, c=3, *rest, **named):
        match a:
            case 1:
                return b
            case _:
                return c


def looped(a):
    while a:
        a -= 1
    return a


class Maker:
    @classmethod
    def make(cls, a):
        return a
