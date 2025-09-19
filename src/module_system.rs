use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use crate::ast::Program;
use crate::checker::ModulePath;
use crate::lexer::{lexer, tts_to_stream};
use crate::parser::program_parser;
use crate::utils::preprocess_src;
use chumsky::prelude::*;

/// Represents a discovered module with its source code and metadata
#[derive(Debug, Clone)]
pub struct DiscoveredModule {
    /// The module path (e.g., "math.geometry")
    pub module_path: ModulePath,

    /// The file system path to the source file
    pub file_path: PathBuf,

    /// The source code content
    pub source_code: String,
}

/// Module discovery engine that maps file system to module hierarchy
pub struct ModuleDiscovery {
    /// Root directory for module discovery
    root_path: PathBuf,
}

impl ModuleDiscovery {
    /// Create a new module discovery engine
    pub fn new<P: AsRef<Path>>(root_path: P) -> Self {
        ModuleDiscovery {
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    /// Discover all modules in the project
    pub fn discover_modules(&self) -> Result<Vec<DiscoveredModule>, String> {
        let mut modules = Vec::new();

        // Discover all modules recursively (including main.alg)
        self.discover_modules_recursive(&self.root_path, &[], &mut modules)?;

        Ok(modules)
    }

    /// Recursively discover modules in a directory
    fn discover_modules_recursive(
        &self,
        current_path: &Path,
        current_module_path: &[String],
        modules: &mut Vec<DiscoveredModule>,
    ) -> Result<(), String> {
        let entries = fs::read_dir(current_path)
            .map_err(|e| format!("Failed to read directory {:?}: {}", current_path, e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            if path.is_file() && file_name.ends_with(".alg") {
                // Handle .alg files
                if file_name == "mod.alg" {
                    // mod.alg represents the current module
                    if !current_module_path.is_empty() {
                        let source = fs::read_to_string(&path)
                            .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;

                        modules.push(DiscoveredModule {
                            module_path: ModulePath::new(current_module_path.to_vec()),
                            file_path: path,
                            source_code: source,
                        });
                    }
                } else if file_name == "main.alg" && current_module_path.is_empty() {
                    // main.alg in root directory represents the root module
                    let source = fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;

                    modules.push(DiscoveredModule {
                        module_path: ModulePath::new(vec![]), // Root module
                        file_path: path,
                        source_code: source,
                    });
                } else {
                    // Regular .alg file represents a submodule
                    let module_name = file_name.strip_suffix(".alg").unwrap().to_string();
                    let mut submodule_path = current_module_path.to_vec();
                    submodule_path.push(module_name);

                    let source = fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read {:?}: {}", path, e))?;

                    modules.push(DiscoveredModule {
                        module_path: ModulePath::new(submodule_path),
                        file_path: path,
                        source_code: source,
                    });
                }
            } else if path.is_dir() && !file_name.starts_with('.') {
                // Recurse into subdirectories
                let mut submodule_path = current_module_path.to_vec();
                submodule_path.push(file_name);

                self.discover_modules_recursive(&path, &submodule_path, modules)?;
            }
        }

        Ok(())
    }

    /// Convert a file path to a module path
    pub fn file_path_to_module_path(&self, file_path: &Path) -> Result<ModulePath, String> {
        let relative_path = file_path.strip_prefix(&self.root_path)
            .map_err(|_| format!("File path {:?} is not within root path {:?}", file_path, self.root_path))?;

        let components: Vec<String> = relative_path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();

        if components.is_empty() {
            return Err("Empty module path".to_string());
        }

        // Handle different cases
        if components.len() == 1 && components[0] == "main.alg" {
            // Root module
            Ok(ModulePath::new(vec![]))
        } else if components.last() == Some(&"mod.alg".to_string()) {
            // Directory module: math/geometry/mod.alg -> math.geometry
            let path = &components[..components.len() - 1];
            Ok(ModulePath::new(path.to_vec()))
        } else if let Some(file) = components.last() {
            if file.ends_with(".alg") {
                // File module: math/statistics.alg -> math.statistics
                let path = &components[..components.len() - 1];
                let mut module_path = path.to_vec();
                let module_name = file.strip_suffix(".alg").unwrap().to_string();
                module_path.push(module_name);
                Ok(ModulePath::new(module_path))
            } else {
                Err(format!("Invalid module file path: {:?}", file_path))
            }
        } else {
            Err(format!("Invalid module file path: {:?}", file_path))
        }
    }
}

/// Error that occurred while parsing a module
#[derive(Debug)]
pub struct ModuleParseError {
    pub module_path: ModulePath,
    pub file_path: PathBuf,
    pub errors: Vec<String>,
}

/// Parse a single module's source code into an AST
pub fn parse_module_source(source_code: &str) -> Result<Program, Vec<String>> {
    // Apply the same preprocessing as single file mode
    let preprocessed_source = preprocess_src(source_code);
    let (tts, lex_errors) = lexer().parse_recovery(preprocessed_source.as_str());

    let mut all_errors = Vec::new();

    // Collect lexing errors
    for error in lex_errors {
        all_errors.push(format!("Lexing error: {:?}", error));
    }

    let program = if let Some(tts) = tts {
        // Flatten token trees into token stream
        let eoi = 0..preprocessed_source.chars().count();
        let token_stream = tts_to_stream(eoi, tts);

        // Parse the token stream
        let (ast, parse_errors) = program_parser().parse_recovery(token_stream);

        // Collect parsing errors
        for error in parse_errors {
            all_errors.push(format!("Parse error: {:?}", error));
        }

        ast
    } else {
        None
    };

    if all_errors.is_empty() {
        if let Some(program) = program {
            Ok(program)
        } else {
            Err(vec!["Failed to parse program".to_string()])
        }
    } else {
        Err(all_errors)
    }
}

/// Collection of parsed modules ready for type checking
#[derive(Debug)]
pub struct ModuleCollection {
    /// Mapping from module path to parsed AST
    pub modules: HashMap<ModulePath, Program>,

    /// Mapping from module path to source file path (for error reporting)
    pub source_files: HashMap<ModulePath, PathBuf>,
}

impl ModuleCollection {
    /// Create a new empty module collection
    pub fn new() -> Self {
        ModuleCollection {
            modules: HashMap::new(),
            source_files: HashMap::new(),
        }
    }

    /// Parse a collection of discovered modules into AST
    pub fn from_discovered_modules(discovered_modules: Vec<DiscoveredModule>) -> Result<Self, Vec<ModuleParseError>> {
        let mut collection = ModuleCollection::new();
        let mut errors = Vec::new();

        for discovered_module in discovered_modules {
            match parse_module_source(&discovered_module.source_code) {
                Ok(program) => {
                    collection.add_module(
                        discovered_module.module_path.clone(),
                        program,
                        discovered_module.file_path.clone(),
                    );
                }
                Err(parse_errors) => {
                    errors.push(ModuleParseError {
                        module_path: discovered_module.module_path,
                        file_path: discovered_module.file_path,
                        errors: parse_errors,
                    });
                }
            }
        }

        if errors.is_empty() {
            Ok(collection)
        } else {
            Err(errors)
        }
    }

    /// Add a parsed module to the collection
    pub fn add_module(&mut self, module_path: ModulePath, program: Program, source_file: PathBuf) {
        self.source_files.insert(module_path.clone(), source_file);
        self.modules.insert(module_path, program);
    }

    /// Get a module by path
    pub fn get_module(&self, module_path: &ModulePath) -> Option<&Program> {
        self.modules.get(module_path)
    }

    /// Get the source file path for a module
    pub fn get_source_file(&self, module_path: &ModulePath) -> Option<&PathBuf> {
        self.source_files.get(module_path)
    }

    /// Get all module paths
    pub fn get_module_paths(&self) -> Vec<ModulePath> {
        self.modules.keys().cloned().collect()
    }

    /// Check if a module exists
    pub fn has_module(&self, module_path: &ModulePath) -> bool {
        self.modules.contains_key(module_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create main.alg - imports from multiple modules
        let main_content = r#"
from math import Number, add
from math.geometry import Circle, area
from graphics import Color, Renderer

def main() -> i32 =
    let num1 = Number(value = 5.0)
    let num2 = Number(value = 3.0)
    let result = add(num1, num2)

    let circle = Circle(center = Vector(x = 0.0, y = 0.0), radius = result)
    let circle_area = area(circle)

    42
"#;
        fs::write(root.join("main.alg"), main_content).unwrap();

        // Create math/mod.alg - core math types and functions
        fs::create_dir_all(root.join("math")).unwrap();
        let math_content = r#"
# Core math module with basic types and operations
export def type Number = {value: f64}
export def type Vector = {x: f64, y: f64}

# Basic arithmetic
export def add(a: Number, b: Number) -> Number =
    Number(value = a.value + b.value)

export def multiply(a: Number, b: Number) -> Number =
    Number(value = a.value * b.value)

# Export enhancement for printing numbers
export open Print^Number
def print(n: Number) -> String =
    "Number(" + n.value.toString() + ")"
close Print^Number
"#;
        fs::write(root.join("math/mod.alg"), math_content).unwrap();

        // Create math/geometry/mod.alg - geometric shapes using math types
        fs::create_dir_all(root.join("math/geometry")).unwrap();
        let geometry_content = r#"
# Geometry module - imports from parent math module
from math import Number, Vector, multiply
from math import Print^Number  # Import enhancement

export def type Circle = {center: Vector, radius: Number}
export def type Rectangle = {width: Number, height: Number}

export def area(c: Circle) -> Number =
    let pi = Number(value = 3.14159)
    let radius_squared = multiply(c.radius, c.radius)
    multiply(pi, radius_squared)

export def rect_area(r: Rectangle) -> Number =
    multiply(r.width, r.height)

# Export enhancement for circles
export open Display^Circle
def display(c: Circle) -> String =
    "Circle(center=(" + c.center.x.toString() + "," + c.center.y.toString() + "), radius=" + c.radius.print() + ")"
close Display^Circle
"#;
        fs::write(root.join("math/geometry/mod.alg"), geometry_content).unwrap();

        // Create math/statistics.alg - statistical functions
        let statistics_content = r#"
# Statistics module - single file submodule
from math import Number, add

export def type Dataset = {values: [Number]}

export def mean(d: Dataset) -> Number =
    let sum = Number(value = 0.0)
    let count = 0
    for value in d.values:
        sum = add(sum, value)
        count = count + 1
    Number(value = sum.value / count)

export def variance(d: Dataset) -> Number =
    let mean_val = mean(d)
    let sum_squares = Number(value = 0.0)
    for value in d.values:
        let diff = Number(value = value.value - mean_val.value)
        sum_squares = add(sum_squares, Number(value = diff.value * diff.value))
    Number(value = sum_squares.value / d.values.length)
"#;
        fs::write(root.join("math/statistics.alg"), statistics_content).unwrap();

        // Create math/utils/mod.alg - utility types
        fs::create_dir_all(root.join("math/utils")).unwrap();
        let utils_content = r#"
# Math utilities
export def type MathConstants = {pi: Number, e: Number}

export def constants() -> MathConstants =
    MathConstants(
        pi = Number(value = 3.14159265359),
        e = Number(value = 2.71828182846)
    )
"#;
        fs::write(root.join("math/utils/mod.alg"), utils_content).unwrap();

        // Create math/utils/helpers.alg - helper functions
        let helpers_content = r#"
# Math helper functions
from math import Number

export def clamp(value: Number, min_val: Number, max_val: Number) -> Number =
    if value.value < min_val.value:
        min_val
    else if value.value > max_val.value:
        max_val
    else:
        value

export def abs(value: Number) -> Number =
    if value.value < 0.0:
        Number(value = -value.value)
    else:
        value
"#;
        fs::write(root.join("math/utils/helpers.alg"), helpers_content).unwrap();

        // Create graphics/mod.alg - graphics module using math types
        fs::create_dir_all(root.join("graphics")).unwrap();
        let graphics_content = r#"
# Graphics module - depends on math module
from math import Vector, Number
from math.geometry import Circle, Rectangle

export def type Color = {r: u8, g: u8, b: u8, a: u8}
export def type Renderer = {width: i32, height: i32}

export def rgb(r: u8, g: u8, b: u8) -> Color =
    Color(r = r, g = g, b = b, a = 255)

export def create_renderer(width: i32, height: i32) -> Renderer =
    Renderer(width = width, height = height)

# Functions that use imported geometric types
export def draw_circle(renderer: Renderer, circle: Circle, color: Color) -> () =
    pass  # Implementation would draw the circle

export def draw_rectangle(renderer: Renderer, rect: Rectangle, color: Color) -> () =
    pass  # Implementation would draw the rectangle
"#;
        fs::write(root.join("graphics/mod.alg"), graphics_content).unwrap();

        temp_dir
    }

    #[test]
    fn test_module_discovery() {
        let temp_dir = create_test_project();
        let discovery = ModuleDiscovery::new(temp_dir.path());

        let modules = discovery.discover_modules().unwrap();

        // Should find all modules: main, math, math.geometry, math.statistics, math.utils, math.utils.helpers, graphics
        assert_eq!(modules.len(), 7);

        let module_paths: Vec<String> = modules
            .iter()
            .map(|m| m.module_path.to_string())
            .collect();

        assert!(module_paths.contains(&"".to_string())); // Root (main.alg)
        assert!(module_paths.contains(&"math".to_string()));
        assert!(module_paths.contains(&"math.geometry".to_string()));
        assert!(module_paths.contains(&"math.statistics".to_string()));
        assert!(module_paths.contains(&"math.utils".to_string()));
        assert!(module_paths.contains(&"math.utils.helpers".to_string()));
        assert!(module_paths.contains(&"graphics".to_string()));
    }

    #[test]
    fn test_file_path_to_module_path() {
        let temp_dir = create_test_project();
        let discovery = ModuleDiscovery::new(temp_dir.path());

        // Test various file path mappings
        let test_cases = vec![
            ("main.alg", ""),
            ("math/mod.alg", "math"),
            ("math/geometry/mod.alg", "math.geometry"),
            ("math/statistics.alg", "math.statistics"),
            ("math/utils/helpers.alg", "math.utils.helpers"),
        ];

        for (file_path, expected_module_path) in test_cases {
            let full_path = temp_dir.path().join(file_path);
            let module_path = discovery.file_path_to_module_path(&full_path).unwrap();
            assert_eq!(module_path.to_string(), expected_module_path,
                      "File path {} should map to module path {}", file_path, expected_module_path);
        }
    }

    #[test]
    fn test_module_collection() {
        let mut collection = ModuleCollection::new();

        let math_path = ModulePath::from_string("math");
        let geometry_path = ModulePath::from_string("math.geometry");

        // Create dummy programs
        let math_program = Program { defs: vec![] };
        let geometry_program = Program { defs: vec![] };

        // Add modules
        collection.add_module(math_path.clone(), math_program, PathBuf::from("math/mod.alg"));
        collection.add_module(geometry_path.clone(), geometry_program, PathBuf::from("math/geometry/mod.alg"));

        // Test retrieval
        assert!(collection.has_module(&math_path));
        assert!(collection.has_module(&geometry_path));
        assert!(!collection.has_module(&ModulePath::from_string("nonexistent")));

        assert!(collection.get_module(&math_path).is_some());
        assert_eq!(collection.get_source_file(&math_path), Some(&PathBuf::from("math/mod.alg")));

        let module_paths = collection.get_module_paths();
        assert_eq!(module_paths.len(), 2);
        assert!(module_paths.contains(&math_path));
        assert!(module_paths.contains(&geometry_path));
    }

    #[test]
    fn test_parse_module_source_success() {
        let source = r#"
export def type Number = {value: f64}

export def add(a: Number, b: Number) -> Number =
    Number(value = a.value + b.value)
"#;

        let result = parse_module_source(source);
        assert!(result.is_ok(), "Should successfully parse valid module source");

        let program = result.unwrap();
        assert_eq!(program.defs.len(), 2); // Type definition and function definition
    }

    #[test]
    fn test_parse_module_source_with_imports() {
        let source = r#"
from math import Number, Vector
from graphics import Color

export def type Circle = {center: Vector, radius: Number}

export def draw(circle: Circle, color: Color) -> () =
    pass
"#;

        let result = parse_module_source(source);
        assert!(result.is_ok(), "Should successfully parse module with imports");

        let program = result.unwrap();
        // Should have 2 import statements + 1 type definition + 1 function
        assert!(!program.defs.is_empty());
    }

    #[test]
    fn test_parse_module_source_syntax_error() {
        let source = r#"
export def type Invalid = {
    # Missing closing brace and other syntax errors
    value: i32
    another_field
"#;

        let result = parse_module_source(source);
        assert!(result.is_err(), "Should fail to parse invalid syntax");

        let errors = result.unwrap_err();
        assert!(!errors.is_empty(), "Should have parsing errors");
    }

    #[test]
    fn test_module_collection_from_discovered_modules_success() {
        // Use the real working example files
        let examples_path = std::env::current_dir().unwrap().join("examples/module_system");
        let discovery = ModuleDiscovery::new(&examples_path);
        let discovered_modules = discovery.discover_modules().unwrap();

        let result = ModuleCollection::from_discovered_modules(discovered_modules);
        assert!(result.is_ok(), "Should successfully parse all discovered modules");

        let collection = result.unwrap();
        assert_eq!(collection.modules.len(), 7); // All 7 modules should parse successfully

        // Verify specific modules exist
        assert!(collection.has_module(&ModulePath::from_string(""))); // Root
        assert!(collection.has_module(&ModulePath::from_string("math")));
        assert!(collection.has_module(&ModulePath::from_string("math.geometry")));
        assert!(collection.has_module(&ModulePath::from_string("graphics")));
    }

    #[test]
    fn test_module_collection_from_discovered_modules_with_errors() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a valid module
        fs::write(root.join("main.alg"), "def main() -> i32 = 42").unwrap();

        // Create an invalid module
        fs::create_dir_all(root.join("bad")).unwrap();
        fs::write(root.join("bad/mod.alg"), "def invalid syntax {{{ missing everything").unwrap();

        let discovery = ModuleDiscovery::new(root);
        let discovered_modules = discovery.discover_modules().unwrap();

        let result = ModuleCollection::from_discovered_modules(discovered_modules);
        assert!(result.is_err(), "Should fail when some modules have parse errors");

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1); // Only the bad module should have errors
        assert_eq!(errors[0].module_path, ModulePath::from_string("bad"));
    }

    #[test]
    fn test_module_collection_operations_extended() {
        let mut collection = ModuleCollection::new();

        let math_path = ModulePath::from_string("math");
        let program = Program { defs: vec![] };
        let file_path = PathBuf::from("math/mod.alg");

        collection.add_module(math_path.clone(), program, file_path.clone());

        assert!(collection.has_module(&math_path));
        assert!(collection.get_module(&math_path).is_some());
        assert_eq!(collection.get_source_file(&math_path), Some(&file_path));

        let module_paths = collection.get_module_paths();
        assert_eq!(module_paths.len(), 1);
        assert!(module_paths.contains(&math_path));
    }

    #[test]
    fn test_end_to_end_module_discovery_and_parsing() {
        // This test verifies the complete pipeline from file system to parsed modules
        // Using the real example files in examples/module_system
        let examples_path = std::env::current_dir().unwrap().join("examples/module_system");
        let discovery = ModuleDiscovery::new(&examples_path);

        // Step 1: Discover modules
        let discovered_modules = discovery.discover_modules().unwrap();
        assert_eq!(discovered_modules.len(), 7);

        // Step 2: Parse all modules
        let collection = ModuleCollection::from_discovered_modules(discovered_modules).unwrap();
        assert_eq!(collection.modules.len(), 7);

        // Step 3: Verify specific modules are parsed correctly
        let math_module = collection.get_module(&ModulePath::from_string("math"));
        assert!(math_module.is_some(), "Math module should be parsed");

        let main_module = collection.get_module(&ModulePath::from_string(""));
        assert!(main_module.is_some(), "Main module should be parsed");

        // Step 4: Verify source file tracking
        for module_path in collection.get_module_paths() {
            let source_file = collection.get_source_file(&module_path);
            assert!(source_file.is_some(), "Every module should have a source file tracked");

            // Verify the file actually exists
            assert!(source_file.unwrap().exists(), "Source file should exist on filesystem");
        }
    }
}