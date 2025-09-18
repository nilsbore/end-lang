# Core math module with basic types and operations
export def type Number = {value: i32}
export def type Vector = {x: Number, y: Number}

# Basic arithmetic operations with multiline implementations
export def add(a: Number, b: Number) -> Number =
    let result_value = a.value + b.value
    Number(value = result_value)

export def multiply(a: Number, b: Number) -> Number =
    let product = a.value * b.value
    Number(value = product)

export def subtract(a: Number, b: Number) -> Number =
    let difference = a.value - b.value
    Number(value = difference)

# Complex multiline vector operations
export def vector_add(v1: Vector, v2: Vector) -> Vector =
    let new_x = add(v1.x, v2.x)
    let new_y = add(v1.y, v2.y)
    Vector(x = new_x, y = new_y)

export def vector_scale(v: Vector, factor: Number) -> Vector =
    let scaled_x = multiply(v.x, factor)
    let scaled_y = multiply(v.y, factor)
    Vector(x = scaled_x, y = scaled_y)

# Advanced multiline function with conditional logic
export def vector_magnitude_squared(v: Vector) -> Number =
    let x_squared = multiply(v.x, v.x)
    let y_squared = multiply(v.y, v.y)
    let mag_sq = add(x_squared, y_squared)
    mag_sq

# Utility function with multiple operations
export def create_unit_vector(direction: Number) -> Vector =
    let zero = Number(value = 0)
    let one = Number(value = 1)
    let x_coord = one
    let y_coord = zero
    Vector(x = x_coord, y = y_coord)
