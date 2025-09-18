use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::prelude::*;
// use chumsky::stream::Stream;
use std::{env, fs};
use std::path::Path;
use std::collections::HashMap;

mod ast;
mod checker;
mod lexer;
mod parser;
mod utils;
mod module_system;
use crate::lexer::{lexer, tts_to_stream};
use crate::parser::program_parser;
use crate::checker::{check_program, ModulePath, ExportRegistry, ImportRequirements, ModuleInterface};
use crate::module_system::{ModuleDiscovery, ModuleCollection};
use crate::ast::{Program, Def, Type, Func};

/// Preprocess source to handle empty lines in indented blocks and ensure newline ending
fn preprocess_src(src: &str) -> String {
    let lines: Vec<&str> = src.lines().collect();
    let mut preprocessed_lines = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if line.trim().is_empty() && i > 0 && i < lines.len() - 1 {
            // This is an empty line between other lines
            let prev_line = lines[i - 1];
            let next_line = lines[i + 1];

            // Get indentation of previous and next lines
            let prev_indent = prev_line.len() - prev_line.trim_start().len();
            let next_indent = next_line.len() - next_line.trim_start().len();

            // Check if both previous and next lines are indented and have content
            let prev_indented = prev_indent > 0 && !prev_line.trim().is_empty();
            let next_indented = next_indent > 0 && !next_line.trim().is_empty();

            if prev_indented && next_indented {
                // Insert a dummy comment with the same indentation as the next line
                let indent_spaces = " ".repeat(next_indent);
                preprocessed_lines.push(format!("{}# dummy", indent_spaces));
            } else {
                preprocessed_lines.push(line.to_string());
            }
        } else {
            preprocessed_lines.push(line.to_string());
        }
    }

    let mut result = preprocessed_lines.join("\n");

    // Ensure input ends with newline for semantic_indentation parser
    if !result.ends_with('\n') {
        result.push('\n');
    }

    result
}

fn parse(src: &str) -> Option<ast::Program> {
    let src_with_newline = preprocess_src(src);

    //println!("=== Preprocessed Source ===");
    //println!("{}", src_with_newline);
    //println!("=== End Preprocessed Source ===");

    let (tts, errs) = lexer().parse_recovery(src_with_newline.as_str());

    let (ast, parse_errs) = if let Some(tts) = tts {
        //println!("--- Token Trees ---\n{:#?}", tts);

        // Next, flatten
        let eoi = 0..src_with_newline.chars().count();
        let token_stream = tts_to_stream(eoi, tts);

        // At this point, we have a token stream that can be fed into the main parser! Because this is just an example,
        // we're instead going to just collect the token stream into a vector and print it.

        //let flattened_trees = token_stream.fetch_tokens().collect::<Vec<_>>();

        //println!("--- Flattened Token Trees ---\n{:?}", flattened_trees);

        //let len = src.chars().count();
        let (ast, parse_errs) = program_parser().parse_recovery(token_stream);
        println!("{:#?}", ast);
        (ast, parse_errs)
    } else {
        (None, Vec::new())
    };

    errs.into_iter()
        .map(|e| e.map(|c| c.to_string()))
        .chain(parse_errs.into_iter().map(|e| e.map(|tok| tok.to_string())))
        .for_each(|e| {
            let report = Report::build(ReportKind::Error, (), e.span().start);

            let report = match e.reason() {
                chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                    .with_message(format!(
                        "Unclosed delimiter {}",
                        delimiter.fg(Color::Yellow)
                    ))
                    .with_label(
                        Label::new(span.clone())
                            .with_message(format!(
                                "Unclosed delimiter {}",
                                delimiter.fg(Color::Yellow)
                            ))
                            .with_color(Color::Yellow),
                    )
                    .with_label(
                        Label::new(e.span())
                            .with_message(format!(
                                "Must be closed before this {}",
                                e.found()
                                    .unwrap_or(&"end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                chumsky::error::SimpleReason::Unexpected => report
                    .with_message(format!(
                        "{}, expected {}",
                        if e.found().is_some() {
                            "Unexpected token in input"
                        } else {
                            "Unexpected end of input"
                        },
                        if e.expected().len() == 0 {
                            "something else".to_string()
                        } else {
                            e.expected()
                                .map(|expected| match expected {
                                    Some(expected) => expected.to_string(),
                                    None => "end of input".to_string(),
                                })
                                .collect::<Vec<_>>()
                                .join(", ")
                        }
                    ))
                    .with_label(
                        Label::new(e.span())
                            .with_message(format!(
                                "Unexpected token {}",
                                e.found()
                                    .unwrap_or(&"end of file".to_string())
                                    .fg(Color::Red)
                            ))
                            .with_color(Color::Red),
                    ),
                chumsky::error::SimpleReason::Custom(msg) => report.with_message(msg).with_label(
                    Label::new(e.span())
                        .with_message(format!("{}", msg.fg(Color::Red)))
                        .with_color(Color::Red),
                ),
            };

            report.finish().print(Source::from(&src)).unwrap();
        });

    ast
}

fn check(ast: ast::Program) -> Option<ast::Program> {
    match check_program(&ast) {
        Ok(()) => {
            println!("Type checking completed successfully!");
            Some(ast)
        }
        Err(error) => {
            eprintln!("Type checking failed: {}", error);
            None
        }
    }
}

/// Dependency graph for module compilation
#[derive(Debug)]
struct DependencyGraph {
    /// Modules in the collection
    modules: Vec<ModulePath>,
    /// Dependencies: module -> list of modules it depends on
    dependencies: HashMap<ModulePath, Vec<ModulePath>>,
}

impl DependencyGraph {
    fn new() -> Self {
        Self {
            modules: Vec::new(),
            dependencies: HashMap::new(),
        }
    }

    fn add_module(&mut self, module_path: ModulePath) {
        if !self.modules.contains(&module_path) {
            self.modules.push(module_path.clone());
            self.dependencies.insert(module_path, Vec::new());
        }
    }

    fn add_dependency(&mut self, module: ModulePath, dependency: ModulePath) -> Result<(), String> {
        self.dependencies.entry(module)
            .or_insert_with(Vec::new)
            .push(dependency);
        Ok(())
    }

    fn validate_no_cycles(&self) -> Result<(), String> {
        // Simple cycle detection - for now just check for direct cycles
        for (module, deps) in &self.dependencies {
            for dep in deps {
                if let Some(dep_deps) = self.dependencies.get(dep) {
                    if dep_deps.contains(module) {
                        return Err(format!("Circular dependency detected between {} and {}",
                            module.to_string(), dep.to_string()));
                    }
                }
            }
        }
        Ok(())
    }

    fn topological_levels(&self) -> Vec<DependencyLevel> {
        let mut levels = Vec::new();
        let mut processed = std::collections::HashSet::new();

        // Start with modules that have no dependencies
        let mut current_level = 0;
        let mut level_modules = Vec::new();

        for module in &self.modules {
            let empty_deps = Vec::new();
            let deps = self.dependencies.get(module).unwrap_or(&empty_deps);
            if deps.is_empty() {
                level_modules.push(module.clone());
                processed.insert(module.clone());
            }
        }

        if !level_modules.is_empty() {
            levels.push(DependencyLevel {
                level: current_level,
                modules: level_modules,
            });
        }

        // Add remaining modules (simplified topological sort)
        current_level += 1;
        let remaining_modules: Vec<ModulePath> = self.modules.iter()
            .filter(|m| !processed.contains(m))
            .cloned()
            .collect();

        if !remaining_modules.is_empty() {
            levels.push(DependencyLevel {
                level: current_level,
                modules: remaining_modules,
            });
        }

        levels
    }
}

/// A level in the dependency graph compilation order
#[derive(Debug)]
struct DependencyLevel {
    level: usize,
    modules: Vec<ModulePath>,
}

/// Registry with resolved imports for a module
#[derive(Debug)]
struct ResolvedRegistry {
    /// Local types available in this module
    types: HashMap<String, Type>,
    /// Local functions available in this module
    functions: HashMap<String, Type>,
    /// Local constants available in this module
    constants: HashMap<String, Type>,
    /// Enhancements available in this module
    enhancements: std::collections::HashSet<String>,
}

impl ResolvedRegistry {
    fn new() -> Self {
        Self {
            types: HashMap::new(),
            functions: HashMap::new(),
            constants: HashMap::new(),
            enhancements: std::collections::HashSet::new(),
        }
    }

    fn merge_exports(&mut self, exports: &ExportRegistry) {
        self.types.extend(exports.types.clone());
        self.functions.extend(exports.functions.clone());
        self.constants.extend(exports.constants.clone());
        self.enhancements.extend(exports.enhancements.clone());
    }

    fn add_imports(&mut self, _source_module: ModulePath, _imported_items: Vec<String>) -> Result<(), String> {
        // TODO: Implement proper import resolution
        Ok(())
    }
}

/// Parse an entire module collection from a directory
fn parse_module_collection<P: AsRef<Path>>(root_path: P) -> Result<ModuleCollection, String> {
    println!("Discovering modules in {:?}...", root_path.as_ref());

    // Step 1: Discover all modules in the directory
    let discovery = ModuleDiscovery::new(root_path);
    let discovered_modules = discovery.discover_modules()
        .map_err(|e| format!("Module discovery failed: {}", e))?;

    println!("Found {} modules", discovered_modules.len());
    for module in &discovered_modules {
        println!("  {} -> {:?}", module.module_path.to_string(), module.file_path);
    }

    // Step 2: Parse all discovered modules
    println!("Parsing modules...");
    let collection = ModuleCollection::from_discovered_modules(discovered_modules)
        .map_err(|errors| {
            let mut error_message = format!("Failed to parse {} modules:\n", errors.len());
            for error in errors {
                error_message.push_str(&format!(
                    "  Module {}: {} errors\n",
                    error.module_path.to_string(),
                    error.errors.len()
                ));
                for err in &error.errors {
                    error_message.push_str(&format!("    - {}\n", err));
                }
            }
            error_message
        })?;

    println!("Successfully parsed {} modules", collection.modules.len());
    Ok(collection)
}

/// Three-phase compilation architecture for module collections
fn compile_module_collection(collection: &ModuleCollection) -> Result<(), String> {
    println!("=== Phase 1: Interface Collection ===");

    // Phase 1: Collect module interfaces from all modules (fully parallel in principle)
    let mut module_interfaces = HashMap::new();
    for (module_path, program) in &collection.modules {
        println!("Collecting interface for module: {}", module_path.to_string());

        let interface = collect_module_interface(module_path.clone(), program)?;
        module_interfaces.insert(module_path.clone(), interface);
    }

    println!("Phase 1 complete: Collected {} module interfaces", module_interfaces.len());

    println!("=== Phase 2: Dependency Resolution ===");

    // Phase 2: Build dependency graph and validate imports
    let dependency_graph = build_dependency_graph(&module_interfaces)?;
    let resolved_registries = resolve_all_imports(&module_interfaces, &dependency_graph)?;

    println!("Phase 2 complete: Resolved dependencies for {} modules", resolved_registries.len());

    println!("=== Phase 3: Implementation Compilation ===");

    // Phase 3: Type check implementations with resolved registries
    // (parallel by dependency level in principle)
    for level in dependency_graph.topological_levels() {
        println!("Compiling dependency level {} with {} modules", level.level, level.modules.len());

        for module_path in &level.modules {
            let program = collection.modules.get(module_path)
                .ok_or_else(|| format!("Module not found: {}", module_path.to_string()))?;
            let registry = resolved_registries.get(module_path)
                .ok_or_else(|| format!("Registry not found: {}", module_path.to_string()))?;

            println!("Type checking module: {}", module_path.to_string());
            compile_module_implementation(program, registry)?;
        }
    }

    println!("Phase 3 complete: All modules compiled successfully");
    Ok(())
}

/// Phase 1: Collect exports and imports from a single module
fn collect_module_interface(module_path: ModulePath, program: &Program) -> Result<ModuleInterface, String> {
    // Extract exports and imports without processing function bodies
    let exports = extract_exports(program)?;
    let imports = extract_imports(program)?;

    Ok(ModuleInterface {
        module_path,
        exports,
        imports,
        interface_version: 1, // Default version
        enhancement_inheritance: HashMap::new(), // TODO: Implement enhancement inheritance
    })
}

/// Extract export information from a module's AST
fn extract_exports(program: &Program) -> Result<ExportRegistry, String> {
    let mut exports = ExportRegistry::new();

    for def in &program.defs {
        match def {
            Def::Type(typedef) => {
                if typedef.exported {
                    let type_signature = extract_type_signature(&typedef.ty.0)?;
                    exports.types.insert(typedef.name.0.clone(), type_signature);
                }
            }
            Def::Func(funcdef) => {
                if funcdef.exported {
                    let func_signature = extract_function_signature(funcdef)?;
                    exports.functions.insert(funcdef.ident.clone(), func_signature);
                }
            }
            Def::Const(constdef) => {
                if constdef.exported {
                    exports.constants.insert(constdef.name.0.clone(), constdef.ty.0.clone());
                }
            }
            Def::Open(openblock) => {
                if openblock.exported {
                    let enhancement_name = format_enhancement(&openblock.ty.0)?;
                    exports.enhancements.insert(enhancement_name);
                }
            }
            _ => {} // Skip other defs for interface collection
        }
    }

    Ok(exports)
}

/// Extract import requirements from a module's AST
fn extract_imports(program: &Program) -> Result<ImportRequirements, String> {
    let imports = ImportRequirements::new();

    for def in &program.defs {
        match def {
            Def::Import(_import_stmt) => {
                // TODO: Parse import statements when import AST nodes are available
                // For now, return empty requirements
            }
            _ => {}
        }
    }

    Ok(imports)
}

/// Phase 2: Build dependency graph from module interfaces
fn build_dependency_graph(interfaces: &HashMap<ModulePath, ModuleInterface>) -> Result<DependencyGraph, String> {
    let mut graph = DependencyGraph::new();

    // Add all modules as nodes
    for module_path in interfaces.keys() {
        graph.add_module(module_path.clone());
    }

    // Add dependencies based on imports
    for (module_path, interface) in interfaces {
        for dependency in interface.imports.get_dependencies() {
            graph.add_dependency(module_path.clone(), dependency)?;
        }
    }

    // Check for circular dependencies
    graph.validate_no_cycles()?;

    Ok(graph)
}

/// Phase 2: Resolve all imports and merge with local symbols
fn resolve_all_imports(
    interfaces: &HashMap<ModulePath, ModuleInterface>,
    dependency_graph: &DependencyGraph,
) -> Result<HashMap<ModulePath, ResolvedRegistry>, String> {
    let mut resolved = HashMap::new();

    // Process modules in dependency order
    for level in dependency_graph.topological_levels() {
        for module_path in &level.modules {
            let interface = interfaces.get(module_path)
                .ok_or_else(|| format!("Interface not found: {}", module_path.to_string()))?;

            let registry = resolve_module_imports(interface, interfaces, &resolved)?;
            resolved.insert(module_path.clone(), registry);
        }
    }

    Ok(resolved)
}

/// Resolve imports for a single module
fn resolve_module_imports(
    interface: &ModuleInterface,
    all_interfaces: &HashMap<ModulePath, ModuleInterface>,
    _resolved_dependencies: &HashMap<ModulePath, ResolvedRegistry>,
) -> Result<ResolvedRegistry, String> {
    let mut registry = ResolvedRegistry::new();

    // Add local exports
    registry.merge_exports(&interface.exports);

    // Resolve each import requirement
    for (source_module, imported_items) in interface.imports.from_imports.iter() {
        let source_interface = all_interfaces.get(source_module)
            .ok_or_else(|| format!("Source module not found: {}", source_module.to_string()))?;

        // Validate that all imported items are actually exported
        for item in imported_items {
            if !source_interface.exports.contains_item(item) {
                return Err(format!(
                    "Module {} does not export item: {}",
                    source_module.to_string(),
                    item
                ));
            }
        }

        // Add imported items to registry
        registry.add_imports(source_module.clone(), imported_items.clone())?;
    }

    Ok(registry)
}

/// Phase 3: Type check a single module implementation
fn compile_module_implementation(program: &Program, _registry: &ResolvedRegistry) -> Result<(), String> {
    // Convert ResolvedRegistry to the format expected by existing type checker
    // For now, use the existing single-module type checker

    match check_program(program) {
        Ok(()) => Ok(()),
        Err(error) => {
            Err(format!("Type error: {}", error))
        }
    }
}

/// Helper function to extract type signature from type definition
fn extract_type_signature(type_def: &Type) -> Result<Type, String> {
    // Return the type definition as-is
    // The existing type checker already handles intersection type creation
    Ok(type_def.clone())
}

/// Helper function to extract function signature
fn extract_function_signature(funcdef: &Func) -> Result<Type, String> {
    // Create function type from parameters and return type
    let param_types: Vec<Type> = funcdef.args.iter()
        .map(|param| param.ty.0.clone())
        .collect();

    let func_type = if param_types.is_empty() {
        // Nullary function: () -> ReturnType
        let unit_type = Type::Tuple(Vec::new()); // Empty tuple for unit type
        Type::Func(Box::new((unit_type, 0..0)), Box::new(funcdef.rtn.clone()))
    } else if param_types.len() == 1 {
        // Unary function: ParamType -> ReturnType
        Type::Func(Box::new((param_types[0].clone(), 0..0)), Box::new(funcdef.rtn.clone()))
    } else {
        // Multi-parameter function: (Param1, Param2, ...) -> ReturnType
        let param_tuple = Type::Tuple(param_types.into_iter().map(|t| (t, 0..0)).collect());
        Type::Func(Box::new((param_tuple, 0..0)), Box::new(funcdef.rtn.clone()))
    };

    // For now, ignore generics since Type::Quantified doesn't exist
    // TODO: Add proper generic support when quantified types are implemented
    Ok(func_type)
}

/// Helper function to format enhancement name
fn format_enhancement(type_annotation: &Type) -> Result<String, String> {
    // Extract enhancement string from intersection type
    match type_annotation {
        Type::Intersection(left, right) => {
            Ok(format!("{}^{}", type_to_string(left), type_to_string(right)))
        }
        _ => Err(format!("Invalid enhancement type: expected intersection, got {}", type_to_string(type_annotation)))
    }
}

/// Helper function to convert type to string representation
fn type_to_string(typ: &Type) -> String {
    match typ {
        Type::Ident(name) => {
            if name.type_args.is_empty() {
                name.path[0].0.clone()
            } else {
                format!("{}[{}]",
                    name.path[0].0,
                    name.type_args.iter()
                        .map(|arg| type_to_string(&arg.0))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        Type::Intersection(left, right) => {
            format!("{}^{}", type_to_string(left), type_to_string(right))
        }
        _ => format!("{:?}", typ) // Fallback for other types
    }
}

/// Check if the provided path is a directory (module collection) or single file
fn is_directory_mode(path: &Path) -> bool {
    path.is_dir() || (!path.exists() && !path.to_string_lossy().ends_with(".alg"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Program, Def, DefType, Func, Type, Item};
    use std::collections::HashMap;

    #[test]
    fn test_three_phase_compilation_architecture() {
        // Create a simple test module collection
        let mut module_collection = ModuleCollection::new();

        // Create a simple main module
        let main_program = Program {
            defs: vec![
                Def::Func(Func {
                    ty: None,
                    ident: "main".to_string(),
                    args: vec![],
                    rtn: (Type::Ident(crate::ast::QualifiedType {
                        path: vec![("i32".to_string(), 0..3)],
                        type_args: vec![]
                    }), 0..3),
                    generics: None,
                    body: Box::new(crate::ast::AssignExpr::Expr(Box::new((
                        crate::ast::Expr::Literal(crate::ast::Literal::Nat(42)),
                        0..2
                    )))),
                    exported: false,
                })
            ]
        };

        module_collection.modules.insert(
            ModulePath::new(vec![]),
            main_program
        );

        // Test three-phase compilation
        let result = compile_module_collection(&module_collection);
        assert!(result.is_ok(), "Three-phase compilation should succeed: {:?}", result);
    }

    #[test]
    fn test_interface_collection_phase() {
        // Test Phase 1: Interface Collection
        let program = Program {
            defs: vec![
                Def::Type(DefType {
                    name: ("TestType".to_string(), 0..8),
                    generics: None,
                    ty: (Type::Ident(crate::ast::QualifiedType {
                        path: vec![("i32".to_string(), 0..3)],
                        type_args: vec![]
                    }), 0..3),
                    exported: true,
                }),
                Def::Func(Func {
                    ty: None,
                    ident: "testFunc".to_string(),
                    args: vec![],
                    rtn: (Type::Ident(crate::ast::QualifiedType {
                        path: vec![("i32".to_string(), 0..3)],
                        type_args: vec![]
                    }), 0..3),
                    generics: None,
                    body: Box::new(crate::ast::AssignExpr::Expr(Box::new((
                        crate::ast::Expr::Literal(crate::ast::Literal::Nat(42)),
                        0..2
                    )))),
                    exported: true,
                })
            ]
        };

        let module_path = ModulePath::new(vec!["test".to_string()]);
        let interface = collect_module_interface(module_path.clone(), &program).unwrap();

        // Verify interface collection
        assert_eq!(interface.module_path, module_path);
        assert!(interface.exports.types.contains_key("TestType"));
        assert!(interface.exports.functions.contains_key("testFunc"));
    }

    #[test]
    fn test_dependency_graph_creation() {
        // Test Phase 2: Dependency Resolution
        let mut interfaces = HashMap::new();

        // Create modules with no dependencies for simplicity
        let main_interface = ModuleInterface {
            module_path: ModulePath::new(vec![]),
            exports: ExportRegistry::new(),
            imports: ImportRequirements::new(),
            interface_version: 1,
            enhancement_inheritance: HashMap::new(),
        };

        let math_interface = ModuleInterface {
            module_path: ModulePath::new(vec!["math".to_string()]),
            exports: ExportRegistry::new(),
            imports: ImportRequirements::new(),
            interface_version: 1,
            enhancement_inheritance: HashMap::new(),
        };

        interfaces.insert(ModulePath::new(vec![]), main_interface);
        interfaces.insert(ModulePath::new(vec!["math".to_string()]), math_interface);

        let dependency_graph = build_dependency_graph(&interfaces).unwrap();

        // Verify dependency graph
        assert_eq!(dependency_graph.modules.len(), 2);
        assert!(dependency_graph.dependencies.contains_key(&ModulePath::new(vec![])));
        assert!(dependency_graph.dependencies.contains_key(&ModulePath::new(vec!["math".to_string()])));
    }

    #[test]
    fn test_export_extraction() {
        // Test export extraction from AST
        let program = Program {
            defs: vec![
                Def::Type(DefType {
                    name: ("ExportedType".to_string(), 0..12),
                    generics: None,
                    ty: (Type::Ident(crate::ast::QualifiedType {
                        path: vec![("i32".to_string(), 0..3)],
                        type_args: vec![]
                    }), 0..3),
                    exported: true,
                }),
                Def::Type(DefType {
                    name: ("PrivateType".to_string(), 0..11),
                    generics: None,
                    ty: (Type::Ident(crate::ast::QualifiedType {
                        path: vec![("i32".to_string(), 0..3)],
                        type_args: vec![]
                    }), 0..3),
                    exported: false,
                })
            ]
        };

        let exports = extract_exports(&program).unwrap();

        // Verify only exported items are included
        assert!(exports.types.contains_key("ExportedType"));
        assert!(!exports.types.contains_key("PrivateType"));
    }

    #[test]
    fn test_function_signature_extraction() {
        // Test function signature extraction
        let func = Func {
            ty: None,
            ident: "testFunc".to_string(),
            args: vec![
                Item {
                    name: "x".to_string(),
                    ty: (Type::Ident(crate::ast::QualifiedType {
                        path: vec![("i32".to_string(), 0..3)],
                        type_args: vec![]
                    }), 0..3),
                }
            ],
            rtn: (Type::Ident(crate::ast::QualifiedType {
                path: vec![("i32".to_string(), 0..3)],
                type_args: vec![]
            }), 0..3),
            generics: None,
            body: Box::new(crate::ast::AssignExpr::Expr(Box::new((
                crate::ast::Expr::Literal(crate::ast::Literal::Nat(42)),
                0..2
            )))),
            exported: true,
        };

        let signature = extract_function_signature(&func).unwrap();

        // Verify function signature is correct
        match signature {
            Type::Func(param_type, ret_type) => {
                // Single parameter function should have direct parameter type
                assert!(matches!(param_type.0, Type::Ident(_)));
                assert!(matches!(ret_type.0, Type::Ident(_)));
            }
            _ => panic!("Expected function type"),
        }
    }

    #[test]
    fn test_three_phase_compilation_with_exports() {
        // Test complete three-phase compilation with actual exports
        let mut module_collection = ModuleCollection::new();

        // Create a module with exports
        let lib_program = Program {
            defs: vec![
                Def::Type(DefType {
                    name: ("Number".to_string(), 0..6),
                    generics: None,
                    ty: (Type::Struct(vec![
                        Item {
                            name: "value".to_string(),
                            ty: (Type::Ident(crate::ast::QualifiedType {
                                path: vec![("i32".to_string(), 0..3)],
                                type_args: vec![]
                            }), 0..3),
                        }
                    ]), 0..10),
                    exported: true,
                }),
                Def::Func(Func {
                    ty: None,
                    ident: "add".to_string(),
                    args: vec![
                        Item {
                            name: "x".to_string(),
                            ty: (Type::Ident(crate::ast::QualifiedType {
                                path: vec![("Number".to_string(), 0..6)],
                                type_args: vec![]
                            }), 0..6),
                        }
                    ],
                    rtn: (Type::Ident(crate::ast::QualifiedType {
                        path: vec![("Number".to_string(), 0..6)],
                        type_args: vec![]
                    }), 0..6),
                    generics: None,
                    body: Box::new(crate::ast::AssignExpr::Expr(Box::new((
                        crate::ast::Expr::QualifiedIdent(crate::ast::QualifiedType {
                            path: vec![("x".to_string(), 0..1)],
                            type_args: vec![]
                        }),
                        0..1
                    )))),
                    exported: true,
                })
            ]
        };

        module_collection.modules.insert(
            ModulePath::new(vec!["lib".to_string()]),
            lib_program
        );

        // Test three-phase compilation
        let result = compile_module_collection(&module_collection);
        assert!(result.is_ok(), "Three-phase compilation with exports should succeed: {:?}", result);
    }
}

fn main() {
    let path_str = env::args().nth(1).expect("Expected file or directory argument");
    let path = Path::new(&path_str);

    if is_directory_mode(path) {
        // Module collection mode - parse entire directory
        println!("Module collection mode: parsing directory {:?}", path);

        match parse_module_collection(path) {
            Ok(collection) => {
                println!("Module collection parsed successfully!");
                println!("Modules: {:?}", collection.get_module_paths());

                // Three-phase compilation
                match compile_module_collection(&collection) {
                    Ok(_) => println!("Module collection compiled successfully!"),
                    Err(error) => {
                        eprintln!("Module compilation failed:\n{}", error);
                        std::process::exit(1);
                    }
                }
            }
            Err(error) => {
                eprintln!("Module collection parsing failed:\n{}", error);
                std::process::exit(1);
            }
        }
    } else {
        // Single file mode - existing behavior
        println!("Single file mode: parsing {:?}", path);

        let src = fs::read_to_string(path)
            .expect("Failed to read file");
        let ast = parse(&src);
        let Some(ast) = ast else { return };
        check(ast);
    }
}
