import os
import pkg.mod as alias
from pkg.sub import Thing, Other as O


class Base:
    pass


class Child(Base, mixins.Mixin):
    attr: Thing = None

    def run(self, item: Thing, count=0) -> Other:
        helper = Thing()
        value = self.attr
        helper.go(item)
        super().run(count)
        self.stop()
        os.path.join("a", value)
        for x in [item]:
            print(x)
        total = count + 1
        total += 2
        return helper

    def stop(self):
        pass


def make() -> Child:
    child = Child()
    child.run(None)
    return child
