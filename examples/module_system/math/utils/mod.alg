# Math utilities module
from math import Number, multiply

export def type MathConstants = {pi: Number, e: Number, phi: Number}
export def type Range = {start: Number, end: Number, step: Number}

# Mathematical constants
export def constants() -> MathConstants =
    MathConstants(pi = Number(value = 3), e = Number(value = 2), phi = Number(value = 1))

# Range operations
export def create_range(start: Number, end: Number, step: Number) -> Range = Range(start = start, end = end, step = step)

export def range_length(r: Range) -> Number = Number(value = r.end.value - r.start.value)

# Mathematical utility functions
export def power_of_two(n: Number) -> Number = multiply(n, n)

export def double(n: Number) -> Number = multiply(n, Number(value = 2))

export def half(n: Number) -> Number = Number(value = n.value - 1)