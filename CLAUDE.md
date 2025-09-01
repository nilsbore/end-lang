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

4. **Type Checker** (`src/checker.rs`): Bidirectional type checking with contexts
   - Implements complex type inference for intersection types and structural subtyping
   - Uses ordered context for existential types and constraint solving

5. **Types** (`src/types.rs`): Core type system definitions
   - Basic types (Bool, Nat, Int, Real, Char, Str)
   - Composite types (Object, List, Dict, Tuple, Func)

### Key Language Features

- **Type definitions**: `def type Person = { name: String, age: u8 }`
- **Generic types**: `def type Vec3 for T: type = { x: T, y: T, z: T }`
- **Sum types**: `def type Option for T: type = | None | Some(T)`
- **Traits**: `def trait Print for T: type = { print: (T) -> str }`
- **Trait implementation**: Open/close blocks for retroactive implementation
- **Intersection types**: `i32^Add` creates a subtype of both i32 and Add
- **Method syntax sugar**: `def Person.method(self: Person) -> ()` equivalent to open/close blocks

## Development Commands

### Build and Run
```bash
cargo build          # Build the project
cargo run <file.alg>  # Parse and analyze a source file
```

### Testing
```bash
cargo test           # Run all tests
cargo check          # Fast syntax/type checking
```

### Example Files
The `examples/` directory contains various language examples:
- `syntax.alg` - Main syntax examples (most up-to-date)
- `types.alg`, `struct.alg`, `enum.alg` - Type system examples
- `trait.alg` - Trait definition and implementation examples
- `generic_*.alg` - Generic programming examples

## Development Notes

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
- **Binary operations**: arithmetic (+, -, *, /) and comparisons (==, <, >, etc.)
- **Variable resolution**: function parameters and local variables
- **Intersection types**: `A^B` with proper subtyping rules (`A^B <: A` and `A^B <: B`)
- **Structural typing with nominal anchoring**: `Person^{name: String, age: i32}`
- **Generic functions**: basic support for `def func(x: T) -> T for T: type`
- **Function type checking**: parameters are properly added to context
- **Subtyping**: Nat literals can subtype to integer types

### Key Entry Points for Type Checker Work

If you want to work on the type checker, start here:

1. **Main entry point**: `check_program()` in `src/checker.rs:925`
   - This is called from `main.rs:114` and processes entire programs
   - Two-pass approach: collect definitions, then check function bodies

2. **Core bidirectional functions**:
   - `synthesizes_to()` at `src/checker.rs:529` - infers types from expressions
   - `checks_against()` at `src/checker.rs:176` - checks expressions against expected types
   - `subtype()` at `src/checker.rs:427` - establishes subtyping relationships

3. **Context management**: `Context` struct at `src/checker.rs:48`
   - Ordered context with existentials, type variables, and solved types
   - Critical for constraint-based type inference

### Priority Areas for Development

1. **Member Access** (`Expr::Member` at `src/checker.rs:698`)
   - Currently returns `Type::Unknown`
   - Essential for structural typing: `person.name`, `obj.field`
   - Should look up field types from struct/object types

2. **Object Literals** (`Expr::Object` at `src/checker.rs:656`)
   - Currently returns `Type::Unknown` 
   - Need to synthesize structural types: `{name: "Alice", age: 30}`
   - Should create proper `Type::Struct` with inferred field types

3. **Function Calls** (`Expr::Call` at `src/checker.rs:613`)
   - Basic implementation exists but limited
   - Need better multi-argument support and type application
   - Generic function instantiation needs work

4. **Trait System** (Open/Close blocks)
   - Currently skipped in `check_program()` (see "Skipping definition type" output)
   - Need to implement trait constraint checking
   - Handle retroactive trait implementation for intersection types

5. **Pattern Matching and Control Flow**
   - Match expressions, if/else, loops
   - Exhaustiveness checking for sum types
   - Type checking for complex control structures

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

```bash
# Test basic functionality
cargo run test_simple.alg    # Basic constant: def x: i32 = 42
cargo run test_binary.alg    # Binary ops: def result: i32 = 2 + 3  
cargo run test_struct.alg    # Struct types: def type Person = { name: String }

# Test complex examples
cargo run examples/syntax.alg        # Full syntax (some features not yet supported)
cargo run examples/syntax_complete.alg  # Comprehensive examples showcasing end-lang features
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

The type checker is now functionally working for basic programs with intersection types and structural/nominal typing - a significant milestone for end-lang's unique type system.