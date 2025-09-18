# Comprehensive module system demonstration
from math import Number, Vector, add, multiply, subtract, vector_add, vector_scale, vector_magnitude_squared, create_unit_vector
from graphics import Color, ColorName, Shape, DrawStyle, Drawable, rgb, rgba, color_from_name, describe_color, calculate_shape_area, create_drawable, blend_colors, create_renderer

# Comprehensive main function demonstrating cross-module usage
def main() -> i32 =
    # Basic math operations with multiline processing
    let num1 = Number(value = 10)
    let num2 = Number(value = 5)
    let sum_result = add(num1, num2)
    let product_result = multiply(sum_result, num2)
    let difference = subtract(product_result, num1)

    # Vector operations demonstration
    let origin = Vector(x = Number(value = 0), y = Number(value = 0))
    let point1 = Vector(x = num1, y = num2)
    let point2 = Vector(x = sum_result, y = difference)
    let combined_vector = vector_add(point1, point2)
    let scaled_vector = vector_scale(combined_vector, Number(value = 2))
    let magnitude_sq = vector_magnitude_squared(scaled_vector)
    let unit_vec = create_unit_vector(Number(value = 1))

    # Color and enum pattern matching demonstration
    let red_color = color_from_name(Red)
    let green_color = color_from_name(Green)
    let custom_purple = color_from_name(Custom(rgb(128, 0, 128)))
    let yellow_name = Yellow
    let described_yellow = describe_color(yellow_name)
    let blended_color = blend_colors(red_color, green_color)

    # Complex shape creation and area calculation
    let circle_shape = Circle(Number(value = 5))
    let rect_shape = Rectangle(Number(value = 4), Number(value = 6))
    let triangle_shape = Triangle(origin, point1, point2)
    let line_shape = Line(origin, combined_vector)

    # Test
    let circle_area = calculate_shape_area(circle_shape)
    let rect_area = calculate_shape_area(rect_shape)
    let triangle_area = calculate_shape_area(triangle_shape)
    let line_area = calculate_shape_area(line_shape)

    # Drawing style and drawable creation
    let filled_style = Filled(red_color)
    let outlined_style = Outlined(green_color, Number(value = 2))
    let gradient_style = Gradient(red_color, blended_color)
    let dashed_style = Dashed(custom_purple, Number(value = 3))

    let drawable1 = create_drawable(circle_shape, filled_style, origin)
    let drawable2 = create_drawable(rect_shape, outlined_style, point1)
    let drawable3 = create_drawable(triangle_shape, gradient_style, point2)
    let drawable4 = create_drawable(line_shape, dashed_style, scaled_vector)

    # Renderer creation and final computation
    let renderer = create_renderer(800, 600)
    let final_result = add(magnitude_sq, circle_area)

    # Return final computed value
    final_result.value

# Demonstration function for enum pattern matching
def demonstrate_color_matching(color_name: ColorName) -> Number =
    let color_info = match color_name:
        | Red => Number(value = 1)
        | Green => Number(value = 2)
        | Blue => Number(value = 3)
        | White => Number(value = 4)
        | Black => Number(value = 5)
        | Yellow => Number(value = 6)
        | Purple => Number(value = 7)
        | Orange => Number(value = 8)
        | Custom(custom_color) => Number(value = 9)
    color_info

# Advanced multiline function demonstrating shape pattern matching
def analyze_shape_properties(shape: Shape) -> Vector =
    let analysis_result = match shape:
        | Circle(radius) =>
            let diameter = multiply(radius, Number(value = 2))
            Vector(x = radius, y = diameter)
        | Rectangle(width, height) =>
            let area_as_number = multiply(width, height)
            Vector(x = width, y = height)
        | Triangle(p1, p2, p3) =>
            let avg_x = add(p1.x, p2.x)
            let avg_y = add(p1.y, p2.y)
            Vector(x = avg_x, y = avg_y)
        | Line(start, end) =>
            let delta_x = subtract(end.x, start.x)
            let delta_y = subtract(end.y, start.y)
            Vector(x = delta_x, y = delta_y)
    analysis_result

# Complex multiline function with nested pattern matching
def process_drawable_style(drawable: Drawable) -> Color =
    let processed_color = match drawable.style:
        | Filled(fill_color) => fill_color
        | Outlined(outline_color, thickness) => outline_color
        | Gradient(start_color, end_color) => blend_colors(start_color, end_color)
        | Dashed(dash_color, spacing) => dash_color
    processed_color

