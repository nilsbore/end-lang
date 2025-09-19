# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **end-lang**, an experimental programming language that combines structural typing with nominal classes, featuring intersection types via the `^` operator. The language draws inspiration from Python syntax, OCaml's type system, Rust's traits, and MLscript's structural subtyping.

## Language Design Principles

- **Structural typing with nominal anchoring**: Types are compared structurally but retain nominal identity
- **Intersection types**: The `^` operator creates intersections like `i32^Add` (a type that is both i32 AND has the Add trait)
- **Retroactive trait implementation**: Types can gain new capabilities after definition through open/close blocks
- **Compositional type propagation**: When a base type implements a trait, structural types containing that base type automatically leverage that implementation
- **Full type annotations at function boundaries**: All function parameters and return types must be explicitly annotated - no type inference across function boundaries
- **Complete type information at module boundaries**: All top-level definitions (types, functions, constants, traits) require explicit type annotations, enabling a fully-typed module registry that supports forward references and retroactive trait implementation
- **Tuple-first collection literals**: `[1, 2, 3]` creates tuple expressions using square bracket syntax - there are no built-in list types

## Architecture Overview

### Core Components

1. **Lexer** (`src/lexer.rs`): Two-stage lexer that first creates token trees, then flattens them into a token stream
   - Handles delimited expressions (parentheses, brackets, braces, blocks)
   - Creates proper token hierarchy before parsing

2. **Parser** (`src/parser.rs`): Recursive descent parser using the `chumsky` crate
   - Builds AST from flattened token stream
   - Handles complex syntax like qualified types, generics, and open/close blocks

3. **AST** (`src/ast.rs`): Abstract Syntax Tree definitions
   - Contains all language constructs: expressions, statements, types, traits
   - Uses spanned types for error reporting

4. **Module Symbol Registry** (`src/checker.rs`): Complete fully-typed registry of all module symbols
   - Collects ALL module definitions: types, functions, constants, traits with complete type information
   - Enables forward references within modules and retroactive trait implementation
   - Three-phase architecture: symbol collection → enhancement application → type checking

5. **Type Checker** (`src/checker.rs`): Bidirectional type checking with contexts
   - Implements complex type inference for intersection types and structural subtyping
   - Uses ordered context for existential types and constraint solving
   - Works with enhanced types from the fully-populated registry

6. **Types** (`src/types.rs`): Core type system definitions
   - Basic types (Bool, Nat, Int, Real, Char, Str)
   - Composite types (Object, List, Dict, Tuple, Func)

7. **Module System** (`src/module_system.rs`): Complete multi-module compilation system
   - Module discovery and parsing from directory structures
   - Export/import resolution with Python-style import syntax
   - Three-phase compilation: Interface Collection → Dependency Resolution → Implementation Compilation
   - Cross-module type checking with enhanced intersection types

### Key Language Features

- **Type definitions**: `def type Person = { name: String, age: u8 }`
- **Generic types**: `def type Vec3 for T: type = { x: T, y: T, z: T }`
- **Sum types**: `def type Option for T: type = | None | Some(T)`
- **Traits**: `def trait Print for T: type = { print: (T) -> str }`
- **Trait implementation**: Open/close blocks for retroactive implementation
- **Intersection types**: `i32^Add` creates a subtype of both i32 and Add
- **Method syntax sugar**: `def Person.method(self: Person) -> ()` equivalent to open/close blocks
- **Tuple literals**: `[1, 2, 3]` creates tuple expressions (square brackets, not lists)
- **Module system**: Export-at-definition with `export def`, Python-style imports with `from module import item`
- **Cross-module type checking**: Full intersection type system works across module boundaries

### Tuple Design Philosophy

End-lang uses a distinctive approach to collection literals with different syntax for types vs expressions:

- **Tuple expressions**: `[1, 2, 3]` creates tuple values using square bracket syntax
- **Tuple types**: `(i32, i32, i32)` defines tuple types using parentheses syntax
- **No built-in lists**: There is no built-in `List[T]` type - collections must be user-defined
- **Type annotations**: Tuples have explicit arity - `[1, 2, 3]: (i32, i32, i32)` not `List[i32]`
- **Parser design**: Square brackets `[...]` map to `Expr::Tuple`, parentheses `(...)` map to `Type::Tuple`
- **Structural typing**: Tuples participate in the intersection type system like other types

**Key Distinction - Expression vs Type Syntax**:
```rust
# Type definitions use parentheses
def type Dataset = {values: (Number, Number, Number), count: i32}

# Expressions use square brackets
def dataset: Dataset = Dataset(values = [Number(value = 1), Number(value = 2), Number(value = 3)], count = 3)

# Complete example
def coordinates: (f64, f64, f64) = [1.0, 2.0, 3.0]  # Correct
def invalid: List[i32] = [1, 2, 3]                  # Error: List undefined
```

**Rationale**: This design emphasizes explicit typing and structural composition over convenient but potentially ambiguous collection types, while maintaining clear separation between type and expression syntax.

## Module System Architecture

End-lang features a complete multi-module compilation system that supports the full intersection type system across module boundaries.

### Module Organization

**Directory Structure**:
```
examples/module_system/
├── main.alg                    # Root module
├── math/
│   ├── mod.alg                 # math module
│   ├── geometry/
│   │   └── mod.alg            # math.geometry module
│   ├── statistics.alg          # math.statistics module
│   └── utils/
│       ├── mod.alg            # math.utils module
│       └── helpers.alg        # math.utils.helpers module
└── graphics/
    └── mod.alg                 # graphics module
```

### Export/Import Syntax

**Export-at-definition syntax**:
```rust
# math/mod.alg
export def type Number = {value: i32}
export def add(x: Number, y: Number) -> Number = Number(value = x.value + y.value)
export def multiply(x: Number, y: Number) -> Number = Number(value = x.value * y.value)
```

**Python-style import syntax**:
```rust
# main.alg
from math import Number, add, multiply, subtract
from graphics import Color, Shape, Drawable

def main() -> i32 =
    let num1 = Number(value = 10)
    let result = add(num1, multiply(num1, Number(value = 2)))
    result.value
```

### Three-Phase Compilation

The module system uses a three-phase compilation architecture:

1. **Phase 1: Interface Collection**
   - Collect module interfaces from all modules in parallel
   - Extract exports: types, functions, constants, traits
   - Build complete module registry without dependencies

2. **Phase 2: Dependency Resolution**
   - Resolve import statements across modules
   - Build dependency graph and topological ordering
   - Validate import/export compatibility

3. **Phase 3: Implementation Compilation**
   - Compile modules in dependency order (levels)
   - Type check with full cross-module symbol resolution
   - Apply retroactive trait implementations globally

### Key Features

**Cross-Module Type Checking**:
- Full intersection type system works across modules
- Enhanced types from one module are available in importing modules
- Retroactive trait implementations apply globally

**Forward References**:
- Modules can reference types/functions defined later in the same module
- Cross-module forward references work through interface collection

**Enhanced Type Propagation**:
- Trait implementations in one module enhance types visible to importing modules
- `open Add^Counter` in module A affects `Counter` usage in module B

### Important Implementation Details

**Preprocessing Consistency**:
- Both single-file and multi-module parsing use the same `preprocess_src()` function
- Located in `src/utils.rs` and applied consistently in `src/main.rs` and `src/module_system.rs`
- Critical for let statements in function bodies and proper block syntax handling

**Parser Features**:
- **Chained member access**: `r.end.value` fully supported through recursive parsing
- **Tuple syntax**: Square brackets `[]` for expressions, parentheses `()` for types
- **Import statement parsing**: Full AST support for `from module import item` syntax

**Testing**:
- Complete end-to-end module system testing with `examples/module_system/`
- All 8 modules parse and type check successfully
- Complex features: imports, exports, chained member access, let statements in function bodies

## Development Commands

### Build and Run
```bash
cargo build                    # Build the project
cargo run <file.alg>           # Parse and analyze a single source file
cargo run <directory>          # Parse and analyze a module collection
```

### Module System Examples
```bash
# Single file mode
cargo run examples/syntax.alg

# Module collection mode
cargo run examples/module_system    # Complete module system with 8 modules
```

### Testing
```bash
cargo test           # Run all tests
cargo check          # Fast syntax/type checking
```

### Example Files
The `examples/` directory contains various language examples:
- `syntax.alg` - Main syntax examples (most up-to-date single file)
- `types.alg`, `struct.alg`, `enum.alg` - Type system examples
- `trait.alg` - Trait definition and implementation examples
- `generic_*.alg` - Generic programming examples
- `module_system/` - Complete multi-module system with 8 modules demonstrating:
  - Cross-module imports and exports
  - Complex type definitions and trait implementations
  - Chained member access and let statements in function bodies
  - Real-world module organization patterns

## Development Notes

### Type Annotation Requirements

End-lang enforces a clear distinction between **module boundaries** and **local scope** for type annotations:

#### **Module Boundaries (Required Type Annotations)**
- **Functions**: `def func(x: T, y: U) -> V` - all parameters and return types must be explicit
- **Constants**: `def name: T = value` - type must be explicitly declared
- **Type definitions**: `def type A = {fields...}` - structural definition required
- **Trait definitions**: `def trait T = {members...}` - all member types must be explicit

#### **Local Scope (Optional Type Annotations)**
- **Let bindings**: `let x = value` or `let x: T = value` - type inference allowed within functions
- **Local variables**: Type inference provides flexibility within function bodies

**Rationale**: This design enables the fully-typed module registry while maintaining ergonomic local programming. Module boundaries require complete type information to support forward references, retroactive trait implementation, and cross-module imports.

### Parser Architecture
The parser uses a two-stage approach:
1. **Lexing**: Creates token trees with proper delimiter matching
2. **Parsing**: Flattens token trees and builds AST using recursive descent

### Type System Implementation
- **Bidirectional typing**: Uses contexts for constraint-based type inference
- **Structural subtyping**: Types are compared by structure while maintaining nominal identity
- **Intersection handling**: The `^` operator creates complex subtyping relationships

### Error Reporting
Uses the `ariadne` crate for rich error reporting with:
- Colored output and span highlighting
- Detailed error messages for parsing failures
- Recovery parsing to continue after errors

## Registry Architecture and Type Information

### Complete Type Information Storage

End-lang's registry stores **complete type information** for all module-boundary definitions, enabling the intersection type system's advanced features like forward references and retroactive trait implementation.

### ModuleSymbolTable Structure (`src/checker.rs:154-177`)

```rust
struct ModuleSymbolTable {
    type_definitions: HashMap<String, Type>,     // Complete intersection types
    function_signatures: HashMap<String, Type>,  // Full function type signatures  
    constants: HashMap<String, Type>,            // Declared constant types
    traits: HashMap<String, TraitDefinition>,    // Complete trait definitions with all members
    enhancements: HashMap<String, Vec<String>>,  // Retroactive trait enhancement registry
    // ... orphan rule and method tracking ...
}
```

### What Gets Stored

#### 1. **Types** → **Complete Intersection Types**
- **Source**: `def type Person = {name: String, age: i32}` (required type annotation)
- **Registry Entry**: `"Person" → Person^{name: String}^{age: i32}`
- **Why**: Automatic nominal+structural intersection creation enables seamless subtyping

#### 2. **Functions** → **Full Type Signatures**  
- **Source**: `def add(x: i32, y: i32) -> i32` (required parameter and return type annotations)
- **Registry Entry**: `"add" → (i32, i32) -> i32`
- **Why**: Complete signatures enable forward references and method dispatch

#### 3. **Constants** → **Declared Types**
- **Source**: `def answer: i32 = 42` (required type annotation) 
- **Registry Entry**: `"answer" → i32`
- **Why**: Explicit types prevent inference ambiguity at module boundaries

#### 4. **Traits** → **Complete Definitions with Member Types**
- **Source**: `def trait Add for T: type = { add: (T, T) -> T }`
- **Registry Entry**: Complete `TraitDefinition` with all member types and kinds
- **Why**: Full trait information enables retroactive implementation and method resolution

### Why Complete Type Information Matters

#### **Forward References Work**
```rust
def useCounter(c: Counter) -> i32 = c.value  // ✅ Works before Counter is defined
def type Counter = {value: i32}              // Defined later
```
**How**: Registry collects all type names in Pass 1, complete definitions in Pass 2

#### **Retroactive Enhancement Possible**
```rust
def type Counter = {value: i32}              // Initially: Counter^{value: i32}
open Add^Counter                             // Enhancement: Counter^{value: i32}^Add
def add(a: Counter, b: Counter) -> Counter   // Function can use enhanced type
close Add^Counter
```
**How**: Registry stores complete type information, then applies enhancements in-place

#### **Method Dispatch Ready (Phase 4)**
```rust
def counter: Counter = {value = 5}
def result: Counter = counter.add(other)     // Future: method call through enhanced type
```
**How**: Registry has complete trait member information for method resolution

### Module Boundary vs Local Scope Design

| **Context** | **Typing** | **Registry** | **Rationale** |
|-------------|------------|--------------|---------------|
| **Functions** | `def func(x: T) -> U` | Full signature | Clear interfaces, forward references |
| **Constants** | `def name: T = value` | Complete type | Module interface clarity |
| **Types** | `def type A = {fields...}` | Full intersection | Retroactive enhancement |
| **Traits** | `def trait T = {members...}` | All member types | Method dispatch capability |
| **Let bindings** | `let x = value` or `let x: T = value` | Not stored | Local inference flexibility |

### Three-Phase Architecture Benefits

1. **Phase 1 - Symbol Collection**: Complete registry enables forward references
2. **Phase 2 - Enhancement Application**: Full type information allows retroactive modification  
3. **Phase 3 - Type Checking**: Enhanced registry provides all types for bidirectional checking

This design makes end-lang's intersection type system both **powerful** (retroactive traits, forward references) and **safe** (complete type information, orphan rule compliance).

## Key Files to Understand

- `lang-design-summary.md` - Comprehensive language design document
- `examples/syntax.alg` - Most current syntax examples
- `src/main.rs` - Entry point showing parsing pipeline
- `src/ast.rs` - Complete AST definitions
- `src/parser.rs` - Parser implementation using chumsky
- `src/checker.rs` - Bidirectional type checker

## Type Checker Implementation Status

### Current Working Features ✅
- **Bidirectional type checking** with synthesis and checking modes
- **Basic types**: literals (Nat, Bool, Str), built-in types (i32, f64, bool, String, etc.)
- **Binary operations**: arithmetic (+, -, *, /) and comparisons (==, <, >, etc.) with proper type synthesis
- **Variable resolution**: function parameters and local variables
- **Intersection types**: `A^B` with proper subtyping rules (`A^B <: A` and `A^B <: B`)
- **Complete intersection-first type system**: ALL types are intersections internally ✅
  - **Parser desugaring**: `{x: i32, y: i32}` parses to `{x: i32}^{y: i32}` field intersections
  - **Automatic nominal intersections**: `def type Person = {name: String, age: i32}` creates `Person^{name: String}^{age: i32}`
  - **Field constraint subtyping**: Object literals `{name = "Alice", age = 30}` work with nominal types
  - **Complete field validation**: Missing or mismatched fields are properly caught
- **Complete module symbol registry**: Collects ALL module symbols (types, functions, constants, traits)
- **Forward reference support**: Variables and functions can reference later-defined types within same module
- **Retroactive trait implementation**: `open Add^Counter` makes ALL existing `Counter` variables become `Counter^{value: i32}^Add`
- **In-place type enhancement**: Enhancement registry directly modifies type definitions for true global transformation
- **Two-pass orphan rule validation**: Prevents implementation of foreign type intersections with correct timing
- **Enhanced type resolution**: Type checker uses complete registry for all lookups from start
- **Structural typing with nominal anchoring**: `Person^{name: String, age: i32}`
- **Generic functions**: basic support for `def func(x: T) -> T for T: type`
- **Function type checking**: parameters are properly added to context
- **Subtyping**: Nat literals can subtype to integer types (i32, u32, i64, etc.)
- **Member access**: Full field lookup for structs, intersection types, and nominal types (`person.name`, `obj.field`)
- **Object literals**: Complete synthesis working seamlessly with nominal types
- **Function calls**: Comprehensive support for nullary `func()`, single-argument `func(x)`, and multi-argument `func(a, b)` calls using tuple-based approach
- **Real-world program support**: Successfully type checks `examples/syntax.alg` with complex features

### Key Entry Points for Type Checker Work

If you want to work on the type checker, start here:

1. **Main entry point**: `check_program()` in `src/checker.rs:1521`
   - This is called from `main.rs` and processes entire programs
   - **Three-phase approach**: 
     - Phase 1: Complete module symbol collection (`collect_module_symbols()`)
     - Phase 1.5: In-place retroactive enhancement (`symbols.apply_all_enhancements()`)
     - Phase 2: Enhanced type checking with complete registry

2. **Module Symbol Registry**: `ModuleSymbolTable` struct at `src/checker.rs:154`
   - **Complete registry**: Collects ALL module symbols (types, functions, constants, traits)
   - **Two-pass collection**: Pass 1 collects type/trait names, Pass 2 processes definitions
   - **Enhancement registry**: Tracks trait implementations from open/close blocks
   - **Orphan rule validation**: Prevents implementation of foreign type intersections

3. **Core bidirectional functions**:
   - `synthesizes_to()` at `src/checker.rs:672` - infers types from expressions
   - `checks_against()` at `src/checker.rs:178` - checks expressions against expected types
   - `subtype()` at `src/checker.rs:494` - establishes subtyping relationships

4. **Context management**: `Context` struct at `src/checker.rs:31`
   - Ordered context with existentials, type variables, and solved types
   - Enhanced with complete registry information from Phase 1
   - Critical for constraint-based type inference

### Priority Areas for Development

The type checker now has **comprehensive support for intersection types with retroactive trait implementation**. The remaining development areas focus on advanced language constructs:

1. **Method Dispatch Through Enhanced Intersection Types** (Phase 4)
   - Method calls on enhanced types: `counter.add(counter)` where `counter: Counter^Add`
   - Trait method resolution using registry information
   - Integration with existing function call infrastructure

2. **Cross-Module Registry Support** (Phase 4)
   - Export/import complete module registries between modules
   - Cross-module type checking with enhanced intersection types
   - Module-scoped enhancement validation and propagation

3. **Let Expressions** (`TODO` at `src/checker.rs:732`)
   - AST support exists but type checker implementation needed
   - Local variable binding with type inference: `let x = expr in body`
   - Requires context management for locally scoped variables

4. **Control Flow Expressions**
   - If/else expressions with type checking for both branches
   - Match expressions with exhaustiveness checking for sum types
   - Loop expressions (for, while) with proper type analysis

5. **Named Function Arguments** (`TODO` at `src/checker.rs:907`)
   - Function calls with named parameters: `func(x: value, y: other)`
   - Parameter name validation and reordering
   - Integration with current tuple-based multi-argument system

6. **Advanced Generic Constraints**
   - Beyond basic `for T: type` constraints
   - Trait bounds: `for T: Trait` constraint checking
   - Higher-kinded types and complex generic relationships
   - Generic function instantiation with constraint solving

7. **Pattern Matching and Destructuring**
   - Tuple destructuring: `let (a, b) = tuple`
   - Struct destructuring: `let {name, age} = person`
   - Enum pattern matching with exhaustiveness checking

8. **Parser Enhancements**
   - Floating point literal support for `examples/syntax_complete.alg`
   - Complex control flow syntax parsing
   - Advanced pattern matching syntax

### Implementation Architecture

The type checker follows the **Complete and Easy Bidirectional Typechecking** paper:

**Reference**: Dunfield & Krishnaswami, "Complete and Easy Bidirectional Typechecking for Higher-Rank Polymorphism" (2019)
- arXiv: https://arxiv.org/abs/1908.05839
- Implements algorithmic bidirectional typing with existential types

**Key concepts implemented**:
- **Synthesis mode**: `expr ⇒ type` (infer type from expression)
- **Checking mode**: `expr ⇐ type` (check expression against expected type)  
- **Subtyping**: `type <: type` with constraint solving
- **Context**: ordered list of type variables, existentials, and solved types
- **Existential instantiation**: `instantiate_l()` and `instantiate_r()` functions

**Critical Design Decision**: End-lang requires **full type annotations at function boundaries**. This means:
- All function parameters must have explicit type annotations: `def func(x: i32, y: String) -> bool`
- All function return types must be explicitly declared  
- No type inference occurs across function calls - the type checker uses the declared types
- This simplifies the implementation significantly and provides clear interface contracts
- Local variables within functions can still use type inference

### Understanding Type Checker Output

When running `cargo run examples/syntax.alg`, you'll see debug output like:
```
check    Literal(Nat(42)) | i32    [context...]    Sub
synth    Literal(Nat(42)) |        [context...]    1I=>  
subtype  Nat | i32                 [context...]    <:NatToInt
```

This shows:
- **check**: Checking expression against expected type (checking mode)
- **synth**: Synthesizing type from expression (synthesis mode)
- **subtype**: Establishing subtyping relationship
- **Context**: `[...]` shows current typing context
- **Rules**: `Sub`, `1I=>`, `<:NatToInt` are type checking rules applied

### Debugging Tips

1. **Enable detailed output**: The checker already prints debug traces
2. **Context inspection**: Look at the `[context...]` to see available variables/types
3. **Rule tracing**: Each rule application is logged (e.g., `Binary`, `Var`, `Sub`)
4. **Panic locations**: Check the line numbers in `src/checker.rs` for unimplemented cases

### Testing the Type Checker

The type checker includes comprehensive test coverage. Run tests with:

```bash
cargo test           # Run all built-in tests (52+ test cases)
cargo run examples/syntax.alg         # ✅ Real-world end-lang program (type checks successfully)
cargo run examples/syntax_complete.alg # ❌ Has parsing issues with float literals (3.14159)
```

**Test Results**:
- **Unit tests**: 52+ test cases covering intersection types, retroactive enhancement, forward references, orphan rule validation
- **Real-world programs**: `examples/syntax.alg` with complex features type checks successfully
- **Integration tests**: Complete module registry, enhancement application, and type resolution working

**Implemented Test Cases Cover:**
- ✅ **Basic constants**: `def answer: i32 = 42`
- ✅ **Binary operations**: `def result: i32 = 2 + 3`, `def isEqual: bool = 5 == 3`
- ✅ **Struct definitions**: `def type Person = { name: String, age: i32 }`
- ✅ **Member access**: `def getName(p: Person) -> String = p.name`
- ✅ **Object literals**: `def person: PersonType = { name = "Alice", age = 30 }`
- ✅ **Function calls**: `def result: i32 = add(5, 3)` (multi-argument support)
- ✅ **Generic functions**: `def times(a: T, b: T) -> T for T: type = a * b`
- ✅ **Intersection types**: Nominal types get proper `Person^{fields}` intersection types
- ✅ **Complete module registry**: All symbols (types, functions, constants, traits) collected
- ✅ **Forward references**: `useCounter(c: Counter) -> i32` works before `Counter` is defined
- ✅ **Retroactive trait implementation**: `open Add^Counter` enhances ALL existing `Counter` variables
- ✅ **In-place type enhancement**: Registry directly modifies type definitions
- ✅ **Two-pass orphan rule validation**: Prevents implementation of foreign type intersections
- ✅ **Enhanced type resolution**: Context uses complete registry information
- ✅ **Real-world program support**: `examples/syntax.alg` with complex features type checks successfully
- ✅ **Trait definitions and open/close blocks**: Full support for retroactive trait implementation
- ✅ **Multiple enhancements**: Types can have multiple traits like `Counter^Add^Display`
- ✅ **Subtyping**: Nat literals subtype to integer types (`42` → `i32`)
- ✅ **Error handling**: Proper failures for missing fields, type mismatches, orphan rule violations
- ✅ **Field intersection parsing**: `{x: i32, y: i32}` creates field constraint intersections
- ✅ **Automatic nominal intersections**: All type definitions become intersections internally
- ✅ **Field constraint validation**: Complete checking of field requirements and types
- ✅ **Object literal → nominal subtybing**: `{name = "Alice"}: Person` type checking works
- ✅ **Error detection**: Field mismatches, missing fields, and orphan rule violations properly caught

**Example Working Programs:**
```rust
// Multi-parameter functions with tuple arguments
def add(x: i32, y: i32) -> i32 = x + y
def result: i32 = add(5, 3)

// Object literals with computed expressions  
def computedObject: ComputedType = {
    sum = 5 + 3,
    comparison = 10 > 5,
    value = 42,
}

// Member access on intersection types
def type Address = { street: String, city: String }
def type Person = { name: String, address: Address }
def getCity(p: Person) -> String = p.address.city
```

### Related Academic Work

- **MLsub**: Stephen Dolan's work on algebraic subtyping
- **Effekt**: Effects system with structural typing
- **TypeScript**: Structural typing in practice
- **OCaml**: Nominal typing with structural comparison
- **Rust traits**: Retroactive implementation concepts

## Dependencies

- `chumsky` - Parser combinator library
- `ariadne` - Error reporting and diagnostics  
- `internment` - String interning (partially used)

The type checker now provides **comprehensive support for end-lang's intersection-first type system**. **Major milestone achieved**: intersection types are now the fundamental primitive throughout the parser and type checker. 

## Current Implementation Status (2024)

### Phase 1-3 Complete ✅ - Major Milestone Achieved

The type checker has achieved a **major milestone** with complete intersection-first type system and retroactive trait implementation:

**Key achievements:**
- **Complete module symbol registry**: ALL symbols (types, functions, constants, traits) collected with forward reference support
- **Retroactive trait implementation**: `open Add^Counter` makes ALL existing `Counter` variables become `Counter^{value: i32}^Add`
- **In-place type enhancement**: Registry directly modifies type definitions for true global transformation
- **Two-pass orphan rule validation**: Prevents implementation of foreign type intersections with correct timing
- **Enhanced type resolution**: Type checker uses complete registry for all lookups from start
- **Real-world program support**: Successfully type checks `examples/syntax.alg` with complex features
- **52+ passing tests**: Full coverage of intersection types, retroactive enhancement, and forward references

### Technical Evidence

**Debug Output from `examples/syntax.alg`**:
```
Type definitions: ["Vec3", "Option", "Person"]
Enhancement registry: {"ScreenPrint": ["T"], "i32": ["Add"], "Vec": ["Add"]...}
Applying retroactive enhancements...
[Vec3: Vec3^{z: T}^{y: T}^{x: T}, Person: Person^{age: u8}^{address: String}^{name: String}, ...]
Type checking completed successfully!
```

This shows the complete intersection-first type system working with:
- Complete registry collecting all module symbols
- Enhancement registry tracking trait implementations  
- In-place retroactive enhancement applied
- Enhanced intersection types throughout the context
- Successful type checking of complex real-world programs

### Architecture Achievement

The implementation demonstrates that **intersection types can serve as the core primitive** for a practical programming language while maintaining ergonomic syntax. The type checker successfully handles:
- Complex structural typing scenarios with proper bidirectional inference
- Retroactive trait implementation across entire module scope
- Forward references within modules
- Orphan rule compliance preventing conflicting implementations
- Real-world program complexity with multiple types, traits, and enhancements

### Phase 4 Complete ✅ - Multi-Module System Achieved

The compiler has achieved **complete multi-module compilation** with full intersection type system support:

**Phase 4 achievements:**
- **Complete module system**: 8-module examples/module_system fully working
- **Three-phase compilation**: Interface Collection → Dependency Resolution → Implementation Compilation
- **Cross-module type checking**: Full intersection types work across module boundaries
- **Export/import resolution**: Python-style `from module import item` syntax fully implemented
- **Enhanced type propagation**: Retroactive trait implementations apply globally across modules
- **Directory-based module discovery**: Automatic module path resolution from filesystem
- **Preprocessing consistency**: Single-file and multi-module parsing use identical preprocessing
- **Parser enhancements**: Chained member access (`r.end.value`) and correct tuple syntax
- **End-to-end validation**: All parsing, type checking, and compilation phases working

**Technical Evidence from `examples/module_system`**:
```
Module collection mode: parsing directory "examples/module_system"
Found 8 modules
Successfully parsed 8 modules
=== Phase 1: Interface Collection ===
Phase 1 complete: Collected 8 module interfaces
=== Phase 2: Dependency Resolution ===
Phase 2 complete: Resolved dependencies for 8 modules
=== Phase 3: Implementation Compilation ===
Phase 3 complete: All modules compiled successfully
Module collection compiled successfully!
```

**Current Status**: End-lang now has a **complete multi-module intersection type system** with full parsing, type checking, and compilation support. The implementation successfully demonstrates intersection types as a practical foundation for a modern programming language with advanced module system capabilities.

**Next priorities**: Method dispatch through enhanced intersection types, advanced pattern matching, and performance optimizations.

## Contributor Guidelines

### Module System Development

**Critical Implementation Details for Contributors**:

1. **Preprocessing Consistency**: Any changes to parsing logic MUST maintain consistency between single-file and multi-module modes:
   - Single file: `parse()` in `src/main.rs` uses `preprocess_src()` from `src/utils.rs`
   - Multi-module: `parse_module_source()` in `src/module_system.rs` uses the same `preprocess_src()`
   - **Never** modify parsing without ensuring both paths use identical preprocessing

2. **Tuple Syntax Rules**:
   - **Type definitions**: Always use parentheses `(T, U, V)` for tuple types
   - **Expressions**: Always use square brackets `[value1, value2, value3]` for tuple values
   - **Parser mapping**: `[...]` → `Expr::Tuple`, `(...)` → `Type::Tuple`

3. **Member Access Parsing**:
   - Chained access like `obj.field.subfield` is supported through recursive `.repeated()` parsing
   - Located in `src/parser.rs` around lines 107-125
   - Avoid breaking the chain by ensuring left-recursion is handled properly

4. **Module System Architecture**:
   - **Three phases must remain separate**: Interface Collection, Dependency Resolution, Implementation Compilation
   - **Cross-module types**: Enhanced intersection types from one module affect all importing modules
   - **Registry consistency**: `ModuleSymbolTable` must contain complete type information for cross-module resolution

5. **Testing Requirements**:
   - Any parser changes must pass both `cargo run examples/syntax.alg` (single file) and `cargo run examples/module_system` (multi-module)
   - New language features should be tested in both modes
   - Module system tests are integration tests - they validate the entire compilation pipeline

### Common Pitfalls

- **Forgetting preprocessing**: Adding new parsing logic without applying `preprocess_src()` will break let statements in function bodies
- **Mixing tuple syntax**: Using `[T, U, V]` in type definitions or `(value1, value2, value3)` in expressions will fail parsing
- **Breaking member access chains**: Modifying member access parsing without considering recursion will break `obj.field.subfield` syntax
- **Incomplete registry information**: The module system requires complete type information at module boundaries - partial type information will break cross-module resolution