def plain():
    return 1


def branches(a, b):
    if a > 0 and b > 0:
        return 1
    elif a < 0 or b < 0:
        return 2
    else:
        return 3


def conditional(a):
    return 1 if a else 2


def loops(xs):
    for x in xs:
        pass
    while xs:
        break
    else:
        pass
    return [y for y in xs if y > 0]


def cases(k):
    match k:
        case 1:
            return "a"
        case 2 | 3:
            return "b"
        case _:
            return "c"


def handlers():
    try:
        pass
    except ValueError:
        pass
    except (KeyError, TypeError):
        pass
    else:
        pass
    finally:
        pass


def outer(items):
    f = lambda v: v if v else 0
    def local(z):
        if z:
            return z
        return 0
    return f(items) + local(1)


def calls(d):
    # a comment
    d["k"] = len(d) * 2
    assert d
    return d.get("k", None)
