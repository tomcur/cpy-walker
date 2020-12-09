#!/usr/bin/env python2

import sys


class Something:
    def __init__(self, anything):
        self.anything = anything


if __name__ == "__main__":
    entry = ["hello world", 42, Something("I'm here!")]
    print(id(entry))

    sys.stdout.flush()
    input()
