# Hierarchical Module System Design (Revised)

## Overview

The language uses a hierarchical module system with **explicit imports** and **export-at-definition** syntax. Modules form compilation units with clear interface boundaries, enabling parallel compilation while maintaining end-lang's intersection type system with retroactive trait implementation.

## Core Design Principles

### 1. **Export-at-Definition**
- `export def type`, `export def`, `export open` - mark items for export at definition site
- Only exported items are visible to other modules
- Clean separation between public interface and private implementation

### 2. **Explicit Python-Style Imports**
- `from module.path import item1, item2` - import specific items
- `import module.path.item` - import with qualified access
- No automatic inheritance - modules explicitly declare dependencies

### 3. **Orphan Rule Prevents Enhancement Conflicts**
- `open Enhancement^Type` requires either `Enhancement` OR `Type` locally defined
- Two modules **cannot** enhance the same foreign type
- Only type-defining modules can enhance with foreign traits
- Only trait-defining modules can apply to foreign types

### 4. **Registry-Based Compilation Units**
- Module interfaces serve as compilation boundaries
- Three-phase compilation enables parallelization
- Incremental compilation with stable interface contracts

## File Structure Convention

```
project/
├── main.alg
└── math/
    ├── mod.alg                    # Root math module
    ├── geometry/
    │   ├── mod.alg                # geometry submodule
    │   └── shapes/
    │       └── mod.alg            # deep submodule
    ├── statistics.alg             # Single file submodule
    └── utils/
        ├── mod.alg                # Directory module
        └── helpers.alg            # Single file submodule
```

**Module Path Mapping**:
- `math/mod.alg` → `math`
- `math/geometry/mod.alg` → `math.geometry`
- `math/statistics.alg` → `math.statistics`
- `math/utils/helpers.alg` → `math.utils.helpers`

## Export Syntax

### Type Definitions
```rust
# Private type - only visible within current module
def type LocalType = {value: i32}

# Exported type - available for import by other modules
export def type Number = {value: f64}
export def type Vector = {x: f64, y: f64}
```

### Function Definitions
```rust
# Private function
def localHelper(x: i32) -> i32 = x + 1

# Exported function
export def add(a: Number, b: Number) -> Number =
    Number(value = a.value + b.value)
```

### Enhancement Definitions
```rust
# Private enhancement - only available in current module
open LocalTrait^Number
def localMethod(n: Number) -> String = "local"
close LocalTrait^Number

# Exported enhancement - available for import
export open Print^Number
def print(n: Number) -> String =
    "Number(" + n.value.toString() + ")"
close Print^Number
```

## Import Syntax

### Specific Item Imports
```rust
# Import specific items from a module
from math import Number, Vector
from math import Print^Number

# Use imported items directly
def example(n: Number) -> String = n.print()
```

### Qualified Imports
```rust
# Import with module qualification
import math.Number
import math.geometry.Circle

# Use with full qualification
def example(n: math.Number) -> math.geometry.Circle = ...
```

### Import Lists
```rust
# Multiple imports from same module
from math.geometry import Circle, Rectangle, Triangle
from math import Number, Vector, Print^Number, Scale^Vector
```

## Subtype Enhancement Inheritance

Complex types automatically inherit enhancements from simpler constituent types:

```rust
# Module A: math/mod.alg
export def type Point = {x: f64, y: f64}
export open Print^Point
def print(p: Point) -> String =
    "Point(" + p.x.toString() + ", " + p.y.toString() + ")"
close Print^Point

# Module B: graphics/mod.alg
from math import Point, Print^Point

# ColoredPoint automatically inherits Print^Point
def type ColoredPoint = Point^{color: String}

def example(cp: ColoredPoint) -> String =
    cp.print()  # ✅ Works - inherited from Point^Print
```

**Inheritance Rules**:
- If `A <: B` and `B` has enhancement `E`, then `A` automatically gets `E`
- Complex intersection types inherit from all constituent types
- More specific enhancements take precedence over general ones

## Example Implementation

### Root Module: `math/mod.alg`

```rust
# Core type definitions with exports
export def type Number = {value: f64}
export def type Vector = {x: f64, y: f64}

# Private enhancement (not exported)
open LocalAdd^Number
def localAdd(a: Number, b: Number) -> Number =
    Number(value = a.value + b.value)
close LocalAdd^Number

# Exported enhancements
export open Print^Number
def print(n: Number) -> String =
    "Number(" + n.value.toString() + ")"
close Print^Number

export open Scale^Vector
def scale(v: Vector, factor: f64) -> Vector =
    Vector(x = v.x * factor, y = v.y * factor)
close Scale^Vector

export open BasicMath^Number
def multiply(a: Number, b: Number) -> Number =
    Number(value = a.value * b.value)
close BasicMath^Number

# Function using private enhancement
def addTwoNumbers(a: Number, b: Number) -> Number =
    a.localAdd(b)  # ✅ Works - LocalAdd^Number available locally
```

### Submodule: `math/geometry/mod.alg`

```rust
# Explicit imports from parent module
from math import Number, Vector
from math import Print^Number, Scale^Vector, BasicMath^Number

def type Circle = {center: Vector, radius: Number}

# Export enhancement for descendants
export open Rotate^Vector
def rotate(v: Vector, angle: f64) -> Vector =
    let cos_a = cos(angle)
    let sin_a = sin(angle)
    Vector(
        x = v.x * cos_a - v.y * sin_a,
        y = v.x * sin_a + v.y * cos_a
    )
close Rotate^Vector

export open Area^Circle
def area(c: Circle) -> Number =
    let pi = Number(value = 3.14159)
    let r_squared = c.radius.multiply(c.radius)  # ✅ BasicMath^Number imported
    pi.multiply(r_squared)
close Area^Circle

def distance(v1: Vector, v2: Vector) -> Number =
    let dx = Number(value = v1.x - v2.x)
    let dy = Number(value = v1.y - v2.y)
    # ❌ Cannot use LocalAdd^Number - not exported from math
    let dx_squared = dx.multiply(dx)  # ✅ BasicMath^Number imported
    let dy_squared = dy.multiply(dy)
    Number(value = sqrt(dx_squared.value + dy_squared.value))
```

### Single File Submodule: `math/statistics.alg`

```rust
# Explicit imports - only gets what it asks for
from math import Number, BasicMath^Number

def type Dataset = {values: [Number]}

export open Stats^Dataset
def mean(d: Dataset) -> Number =
    let sum = Number(value = 0.0)
    let count = 0
    for value in d.values:
        # ❌ Cannot use LocalAdd^Number - not exported from math
        sum = Number(value = sum.value + value.value)
        count = count + 1
    Number(value = sum.value / count)

def variance(d: Dataset) -> Number =
    let mean_val = d.mean()
    let sum_squares = Number(value = 0.0)
    for value in d.values:
        let diff = Number(value = value.value - mean_val.value)
        let square = diff.multiply(diff)  # ✅ BasicMath^Number imported
        sum_squares = Number(value = sum_squares.value + square.value)
    Number(value = sum_squares.value / d.values.length)
close Stats^Dataset

# ❌ Cannot use Rotate^Vector - not imported from geometry sibling
def processVectors(vectors: [Vector]) -> Number =
    # ❌ Error: Vector not imported from math
    # Need: from math import Vector, Scale^Vector
```

## Three-Phase Compilation Architecture

### Phase 1: Interface Collection (Fully Parallel)
```rust
// Collect exports from all modules without processing bodies
ModuleInterface {
    module_path: "math.geometry",
    exports: {
        types: ["Circle"],
        functions: ["distance"],
        enhancements: ["Rotate^Vector", "Area^Circle"]
    },
    imports: {
        from_math: ["Number", "Vector", "Print^Number", "Scale^Vector", "BasicMath^Number"]
    }
}
```

### Phase 2: Dependency Resolution (Fast Sequential)
```rust
// Build dependency graph and validate all imports
DependencyGraph {
    "math": [],                           // No dependencies
    "math.geometry": ["math"],            // Depends on math
    "math.statistics": ["math"],          // Depends on math
    "graphics": ["math", "math.geometry"] // Depends on both
}
```

### Phase 3: Implementation Compilation (Parallel by Level)
```rust
// Compile in dependency order with parallelization within levels
Level 0: ["math"]                    // Compile first (no dependencies)
Level 1: ["math.geometry", "math.statistics"]  // Parallel (both depend only on math)
Level 2: ["graphics"]                // Depends on Level 0 and 1
```

## Benefits for Compilation Units

### **Parallel Compilation**
- Interface phase: All modules processed simultaneously
- Implementation phase: Parallel within dependency levels
- Much faster than sequential compilation

### **Incremental Builds**
- Interface changes only affect dependent modules
- Implementation changes don't affect other modules
- Stable interfaces enable aggressive caching

### **Clear Boundaries**
- Registry interfaces serve as compilation contracts
- Type checking uses pre-resolved registries
- Clean separation between signature and implementation

### **Orphan Rule Compliance**
- Enhancement conflicts impossible due to orphan rule
- Clear ownership of type/trait combinations
- Predictable enhancement resolution

## Registry Interface Format

```rust
struct ModuleInterface {
    module_path: ModulePath,              // "math.geometry"
    exports: ExportRegistry,              // What this module provides
    imports: ImportRequirements,          // What this module needs
    interface_version: Version,           // For compatibility checking
    enhancement_inheritance: EnhancementMap, // Subtype inheritance rules
}

struct ExportRegistry {
    types: HashMap<String, Type>,         // Complete type signatures
    functions: HashMap<String, Type>,     // Function type signatures
    constants: HashMap<String, Type>,     // Constant types
    enhancements: HashSet<String>,        // "Enhancement^Type" strings
}
```

This design provides **clean module boundaries**, **parallel compilation**, and **stable interfaces** while maintaining end-lang's powerful intersection type system with retroactive trait implementation.