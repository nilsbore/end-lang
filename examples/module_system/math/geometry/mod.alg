# Geometry module - imports from parent math module
from math import Number, Vector, multiply, add
from math import vector_add, vector_scale

export def type Circle = {center: Vector, radius: Number}
export def type Rectangle = {width: Number, height: Number}
export def type Point = {position: Vector}

# Circle operations
export def area(c: Circle) -> Number = multiply(Number(value = 3), multiply(c.radius, c.radius))

export def circumference(c: Circle) -> Number = multiply(multiply(Number(value = 2), Number(value = 3)), c.radius)

# Rectangle operations
export def rect_area(r: Rectangle) -> Number = multiply(r.width, r.height)

export def rect_perimeter(r: Rectangle) -> Number = add(multiply(r.width, Number(value = 2)), multiply(r.height, Number(value = 2)))

# Point operations
export def translate_point(p: Point, offset: Vector) -> Point = Point(position = vector_add(p.position, offset))

export def scale_point(p: Point, factor: Number) -> Point = Point(position = vector_scale(p.position, factor))