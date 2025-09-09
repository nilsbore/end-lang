# Intersection Type System Overhaul Plan

## Understanding the Real Mechanism

From examples/syntax.alg lines 88-95:
```python
open Add^i32
def add(a: i32, b: i32) -> i32 = a + b
close Add^i32
# Comment: "type of i32 is now i32^Add, so it is a subtype of i32 and Add"
```

**Key Insight:** `open Trait^Type` creates an intersection `Type^Trait` that applies globally and retroactively to ALL variables of that type.

## Core Architecture Changes

### All Nominal Types Are Growing Intersections

Every type annotation in end-lang becomes an intersection that accumulates constraints:

- `def type A = {x: i32}` creates `A` with initial constraint `{x: i32}` → internally `A^{x: i32}`
- `a: A` means `a: A^{x: i32}` in ALL type checking contexts  
- `open Add^A` makes ALL existing `x: A` become `x: A^{x: i32}^Add` retroactively
- Field constraints: `{x: i32, y: i32}` → `{x: i32}^{y: i32}` (intersection of single-field constraints)

## Phase 1: Open/Close Block Infrastructure

### 1.1 AST Extensions
- `Open` and `Close` AST nodes already exist (ast.rs:233-241)
- Add intersection type tracking in `Open { ty: Spanned<Type> }`  
- Support `open Add^i32` syntax parsing

### 1.2 Parser Updates  
- Ensure parser handles intersection types in open/close: `Add^Vec[T] for T: Add`
- Parse method definitions within open/close blocks
- Handle generic constraints: `for T: Add` in open blocks

### 1.3 Global Type State Management
```rust
struct TypeEnhancementRegistry {
    // Track which types have gained which traits: i32 -> [Add, Print, ...]
    type_enhancements: HashMap<String, Vec<String>>,
    // Store method implementations: (Add, i32) -> [add_function_def]
    trait_implementations: HashMap<(String, String), Vec<FuncDef>>,
}
```

## Phase 2: Retroactive Type Enhancement

### 2.1 Multi-Pass Program Analysis
- **Pass 1:** Parse all definitions, collect trait definitions
- **Pass 2:** Process all `open Trait^Type` blocks → populate enhancement registry  
- **Pass 3:** Resolve all type annotations with enhanced intersections
- **Pass 4:** Type check expressions with fully enhanced types

### 2.2 Dynamic Type Resolution
- When type checker encounters `x: i32`, check enhancement registry
- If registry shows `i32` has `Add`, resolve as `x: i32^Add`
- All type operations use enhanced intersection types automatically

### 2.3 Context Enhancement
- Modify `Context::TypedVariable(name, type)` resolution
- Before type checking, enhance all types in context with registry data
- `x: i32` becomes `x: i32^Add` if enhancement registry shows `i32^Add` exists

## Phase 3: Structural Type Desugaring  

### 3.1 Field-Level Intersection Implementation
- `def type Person = {name: String, age: i32}` creates `Person^{name: String}^{age: i32}`
- Parser converts struct syntax to intersection of field constraints
- Each `{field: Type}` becomes separate intersection component

### 3.2 Structural Constraint Types
```rust
enum Type {
    // Existing...
    FieldConstraint(String, Box<Type>),  // {field: Type}
    // Keep existing Struct for compatibility during transition
}
```

### 3.3 Member Access Through Intersections
- `person.name` searches intersection for `{name: Type}` constraint
- Works on any type with appropriate field constraint in intersection
- Enables structural typing: `{name: String}^{age: i32}` has `.name` and `.age`

## Phase 4: Open/Close Block Processing

### 4.1 Enhancement Registry Population
- `open Add^i32 ... close Add^i32` adds `i32 -> Add` to enhancement registry
- Store function definitions from within the block
- Support generic open blocks: `open Add^Vec[T] for T: Add`

### 4.2 Retroactive Application
- After processing open/close block, ALL existing `x: i32` become `x: i32^Add`
- Update type checking context with enhanced types
- No need to re-declare variables - enhancement is automatic

### 4.3 Method Resolution Integration
- When calling `x.add(y)` where `x: i32^Add`, look up trait implementation
- Use stored function definitions from open/close blocks
- Support method dispatch through intersection components

## Phase 5: Type Checking Integration

### 5.1 Subtyping with Enhanced Types
- `i32^Add <: Add` (intersection subtyping rule)
- `i32^Add <: i32` (intersection subtyping rule)  
- Update `subtype()` (checker.rs:427) to handle enhanced intersections

### 5.2 Variable Resolution Enhancement
- `synthesizes_to()` for `QualifiedIdent(i32_var)` returns enhanced type `i32^Add`
- Context lookup automatically provides enhanced types
- No changes needed to expression type synthesis - works through enhanced resolution

### 5.3 Field Constraint Satisfaction
- `{x: i32^Add}` satisfies `{x: Add}` through intersection subtyping
- Implement compositional trait propagation through structure
- Enable structural method dispatch on enhanced field types

## Implementation Strategy

### Week 1: Enhancement Registry
1. Implement `TypeEnhancementRegistry` with HashMap storage
2. Add registry population from `open Trait^Type` blocks  
3. Implement type resolution with registry lookup

### Week 2: Retroactive Enhancement
1. Multi-pass program analysis with enhancement application
2. Context enhancement with registry-resolved intersection types
3. Update all variable type resolution to use enhanced types

### Week 3: Structural Desugaring  
1. Convert struct types to field constraint intersections
2. Update member access to search intersection components
3. Implement field constraint subtyping rules

### Week 4: Integration & Testing
1. Full integration of enhanced types throughout type checker
2. Test retroactive enhancement behavior
3. Verify structural typing works with enhanced intersections

## Key Technical Insights

### Insight 1: Global Type Enhancement
`open Add^i32` doesn't create new variables - it enhances the MEANING of existing type `i32` everywhere in the program.

### Insight 2: Intersection is the Core Type System
Every type becomes an intersection: `i32`, `Person^{name:String}^{age:i32}`, `Vec[T]^Add^Print`.

### Insight 3: Retroactive Application  
Enhancement happens after variable declaration but before type checking - it's a global transformation.

### Insight 4: Structural + Nominal Unification
Field constraints `{name: String}` and trait constraints `Add` are both intersection components with same semantics.

## Success Criteria

✅ **Retroactive Enhancement:** `open Add^i32` makes ALL `x: i32` become `x: i32^Add`
✅ **Intersection Subtyping:** `i32^Add <: i32` and `i32^Add <: Add` both work  
✅ **Structural Fields:** `Person^{name: String}^{age: i32}` supports `person.name` access
✅ **Method Dispatch:** `x.add(y)` works where `x: i32^Add` using trait implementation
✅ **Compositional Typing:** `{x: i32^Add}` automatically satisfies `{x: Add}`

This plan correctly implements the open/close block system from the spec - where trait implementations create global, retroactive type enhancements through intersection types.

## Current State Analysis

**What Works Now:**
- Basic intersection types `A^B` with subtyping rules
- Struct types with field access and object literal synthesis
- Function calls (nullary, single-arg, multi-arg) with tuple-based approach
- Comprehensive test coverage for current structural typing

**What Needs Implementation:**
- Open/Close block processing (currently skipped in `check_program()`)
- Field-level intersection desugaring (`{x: i32, y: i32}` → `{x: i32}^{y: i32}`)
- Dynamic type enhancement registry
- Retroactive type annotation updating
- Trait implementation method storage and dispatch

## Implementation Files to Modify

- **src/ast.rs**: Add `Type::FieldConstraint`, enhance `Open`/`Close` nodes
- **src/parser.rs**: Update struct parsing to create field intersections, handle open/close syntax
- **src/checker.rs**: Multi-pass program analysis, enhancement registry, enhanced type resolution
- **tests**: Comprehensive intersection type test suite with retroactive enhancement

## References

- **lang-design-spec.md**: Complete specification for intersection-based type system
- **examples/syntax.alg**: Shows actual open/close block syntax and semantics
- **CLAUDE.md**: Current type checker status and architecture