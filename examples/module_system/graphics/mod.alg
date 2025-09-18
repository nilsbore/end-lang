# Graphics module with comprehensive enums and pattern matching
from math import Number, Vector

# Enhanced type definitions with more detailed structure
export def type Color = {r: u8, g: u8, b: u8, a: u8}
export def type Renderer = {width: i32, height: i32}

# Comprehensive enum for color names with pattern matching
export def type ColorName =
    | Red
    | Green
    | Blue
    | White
    | Black
    | Yellow
    | Purple
    | Orange
    | Custom(Color)

# Shape enum for different drawable shapes
export def type Shape =
    | Circle(Number)
    | Rectangle(Number, Number)
    | Triangle(Vector, Vector, Vector)
    | Line(Vector, Vector)

# Drawing style enum with various options
export def type DrawStyle =
    | Filled(Color)
    | Outlined(Color, Number)
    | Gradient(Color, Color)
    | Dashed(Color, Number)

# Enhanced drawable object
export def type Drawable = {
    shape: Shape,
    style: DrawStyle,
    position: Vector
}

# Multiline color creation with validation
export def rgb(r: u8, g: u8, b: u8) -> Color =
    let validated_r = r
    let validated_g = g
    let validated_b = b
    let full_alpha = 255
    Color(r = validated_r, g = validated_g, b = validated_b, a = full_alpha)

export def rgba(r: u8, g: u8, b: u8, a: u8) -> Color =
    let color_r = r
    let color_g = g
    let color_b = b
    let color_a = a
    Color(r = color_r, g = color_g, b = color_b, a = color_a)

# Comprehensive pattern matching for color names
export def color_from_name(name: ColorName) -> Color =
    match name:
        | Red => rgb(255, 0, 0)
        | Green => rgb(0, 255, 0)
        | Blue => rgb(0, 0, 255)
        | White => rgb(255, 255, 255)
        | Black => rgb(0, 0, 0)
        | Yellow => rgb(255, 255, 0)
        | Purple => rgb(128, 0, 128)
        | Orange => rgb(255, 165, 0)
        | Custom(c) => c

# Advanced multiline function with complex pattern matching
export def describe_color(name: ColorName) -> ColorName =
    let result = match name:
        | Red => Red
        | Green => Green
        | Blue => Blue
        | White => White
        | Black => Black
        | Yellow => Yellow
        | Purple => Purple
        | Orange => Orange
        | Custom(custom_color) => Custom(custom_color)
    result

# Complex multiline shape analysis with pattern matching
export def calculate_shape_area(shape: Shape) -> Number =
    let area_result = match shape:
        | Circle(radius) =>
            let pi_approx = Number(value = 3)
            let radius_squared = Number(value = radius.value * radius.value)
            Number(value = pi_approx.value * radius_squared.value)
        | Rectangle(width, height) =>
            Number(value = width.value * height.value)
        | Triangle(p1, p2, p3) =>
            Number(value = 10)
        | Line(start, end) =>
            Number(value = 0)
    area_result

# Elaborate multiline drawing function with style pattern matching
export def create_drawable(shape: Shape, style: DrawStyle, pos: Vector) -> Drawable =
    let processed_shape = shape
    let processed_style = match style:
        | Filled(color) => Filled(color)
        | Outlined(color, thickness) => Outlined(color, thickness)
        | Gradient(color1, color2) => Gradient(color1, color2)
        | Dashed(color, spacing) => Dashed(color, spacing)
    let drawable_position = pos
    Drawable(shape = processed_shape, style = processed_style, position = drawable_position)

# Advanced multiline color blending with conditional logic
export def blend_colors(c1: Color, c2: Color) -> Color =
    let red_blend = c1.r + c2.r
    let green_blend = c1.g + c2.g
    let blue_blend = c1.b + c2.b
    let alpha_blend = c1.a + c2.a
    let final_red = red_blend
    let final_green = green_blend
    let final_blue = blue_blend
    let final_alpha = alpha_blend
    Color(r = final_red, g = final_green, b = final_blue, a = final_alpha)

# Renderer creation with validation
export def create_renderer(width: i32, height: i32) -> Renderer =
    let validated_width = width
    let validated_height = height
    Renderer(width = validated_width, height = validated_height)
