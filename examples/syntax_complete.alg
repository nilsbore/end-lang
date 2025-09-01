# End-Lang Complete Syntax Examples
# Showcasing structural typing with nominal anchoring and intersection types

# ============================================================================
# BASIC TYPES AND LITERALS
# ============================================================================

def answer: i32 = 42
def pi: f64 = 3.14159
def greeting: String = "Hello, World!"
def is_ready: bool = true

# ============================================================================
# STRUCTURAL TYPING WITH NOMINAL ANCHORING
# ============================================================================

# Define a structural type with nominal identity
# This creates Person^{name: String, age: i32, address: String}
def type Person = {
    name: String,
    age: i32,
    address: String,
}

# Define another type with similar structure
def type Employee = {
    name: String,
    age: i32,
    id: i32,
    department: String,
}

# Structural compatibility - can use Person where {name: String, age: i32} is expected
def getName(p: {name: String, age: i32}) -> String = p.name

# Both Person and Employee can be passed to getName due to structural subtyping
def example1() -> String =
    let person: Person = {name="Alice", age=30, address="123 Main St"}
    let employee: Employee = {name="Bob", age=25, id=1001, department="Engineering"}
    
    let name1 = getName(person)    # Person <: {name: String, age: i32}
    let name2 = getName(employee)  # Employee <: {name: String, age: i32}
    
    return name1

# ============================================================================
# INTERSECTION TYPES AND TRAIT SYSTEM
# ============================================================================

# Define traits that describe capabilities
def trait Printable for T: type = {
    toString: (T) -> String,
}

def trait Comparable for T: type = {
    compare: (T, T) -> i32,
}

# Retroactive trait implementation using open/close blocks
open Printable^Person

def toString(p: Person) -> String =
    "Person(name=" + p.name + ", age=" + p.age + ")"

close Printable^Person

# Now Person has type Person^{name: String, age: i32, address: String}^Printable
# This means Person is a subtype of:
# - Person (nominal)
# - {name: String, age: i32, address: String} (structural)  
# - Printable (trait)

# Functions can require intersection types
def printAndReturn(x: T) -> T for T: Printable =
    print(x.toString())
    return x

# ============================================================================
# GENERIC TYPES WITH STRUCTURAL CONSTRAINTS
# ============================================================================

# Generic type with structural constraint
def type Point for T: type = {
    x: T,
    y: T,
}

# Specialized types
def type Point2D = Point[f64]
def type Point3D for T: type = {
    x: T,
    y: T, 
    z: T,
}

# Point3D^T is a subtype of Point[T] due to structural subtyping
def distance2D(p: Point[f64]) -> f64 =
    sqrt(p.x * p.x + p.y * p.y)

def example2() -> f64 =
    let p3d: Point3D[f64] = {x=3.0, y=4.0, z=5.0}
    return distance2D(p3d)  # Point3D[f64] <: Point[f64]

# ============================================================================
# INTERSECTION TYPES WITH MULTIPLE TRAITS
# ============================================================================

def trait Add for T: type = {
    add: (T, T) -> T,
}

def trait Multiply for T: type = {
    multiply: (T, T) -> T,
}

# Function requiring intersection of two traits
def quadratic(a: T, b: T, c: T, x: T) -> T for T: Add^Multiply =
    # T must implement both Add AND Multiply
    let ax2 = a.multiply(x.multiply(x))
    let bx = b.multiply(x)
    return ax2.add(bx).add(c)

# Implement both traits for f64
open Add^f64
def add(a: f64, b: f64) -> f64 = a + b
close Add^f64

open Multiply^f64  
def multiply(a: f64, b: f64) -> f64 = a * b
close Multiply^f64

# Now f64 has type f64^Add^Multiply

# ============================================================================
# ENUM TYPES WITH INTERSECTION SUPPORT
# ============================================================================

def type Option for T: type = 
    | None
    | Some(T)

def type Result for T: type, E: type =
    | Ok(T)
    | Err(E)

# Pattern matching with intersection types
def unwrapOrDefault(opt: Option[T], default: T) -> T for T: type =
    match opt:
    | None => default
    | Some(value) => value

# ============================================================================
# COMPLEX STRUCTURAL SUBTYPING
# ============================================================================

# Define a type that extends another structurally
def type NamedEntity = {
    name: String,
}

def type TimestampedEntity = {
    name: String,
    created_at: i64,
}

def type FullEntity = {
    name: String,
    created_at: i64,
    id: i32,
    metadata: String,
}

# Chain of structural subtyping:
# FullEntity <: TimestampedEntity <: NamedEntity

def processNamed(entity: NamedEntity) -> String = entity.name
def processTimestamped(entity: TimestampedEntity) -> i64 = entity.created_at

def example3() -> i64 =
    let full: FullEntity = {
        name="MyEntity", 
        created_at=1234567890,
        id=42,
        metadata="Important"
    }
    
    # All of these work due to structural subtyping
    let name = processNamed(full)           # FullEntity <: NamedEntity  
    let timestamp = processTimestamped(full) # FullEntity <: TimestampedEntity
    
    return timestamp

# ============================================================================
# METHOD SYNTAX SUGAR WITH INTERSECTION TYPES  
# ============================================================================

# Method syntax is sugar for open/close blocks
def Person.getDisplayName(self: Person) -> String =
    self.name + " (" + self.age + " years old)"

# Equivalent to:
# open Person
# def getDisplayName(self: Person) -> String = ...
# close Person

# Methods work with intersection types
def Person.greet(self: T) -> String for T: Person^Printable =
    "Hello, I am " + self.toString()

# ============================================================================
# COMPOSITIONAL TYPE PROPAGATION
# ============================================================================

# When a base type implements a trait, structural types containing 
# that base automatically benefit

def type Wrapper for T: type = {
    value: T,
    id: i32,
}

# If T implements Add, then Wrapper[T] can leverage that implementation
def Wrapper[T].addValues(self: Wrapper[T], other: Wrapper[T]) -> T for T: Add =
    self.value.add(other.value)

def example4() -> f64 =
    let w1: Wrapper[f64] = {value=3.14, id=1}
    let w2: Wrapper[f64] = {value=2.71, id=2}
    
    # This works because f64 implements Add, and Wrapper[f64] can use that
    return w1.addValues(w2)

# ============================================================================
# ADVANCED INTERSECTION COMBINATIONS
# ============================================================================

# Type that is the intersection of multiple structural and nominal types
def type RichEntity = Person^Employee^Printable^Comparable

# Functions can work with complex intersection requirements
def processRichEntity(entity: Person^Printable) -> String =
    # entity has both Person properties AND Printable behavior
    "Processing: " + entity.toString() + " at " + entity.address

# ============================================================================
# BINARY OPERATIONS AND TYPE INFERENCE
# ============================================================================

def arithmetic_examples() -> i32 =
    let a = 10
    let b = 5
    
    let sum = a + b       # Type: i32
    let diff = a - b      # Type: i32  
    let product = a * b   # Type: i32
    let quotient = a / b  # Type: i32
    
    let is_equal = a == b     # Type: bool
    let is_greater = a > b    # Type: bool
    let is_less_eq = a <= b   # Type: bool
    
    return sum

# ============================================================================
# DEMONSTRATING TYPE SAFETY WITH INTERSECTION TYPES
# ============================================================================

def type SecureData = {
    data: String,
    access_level: i32,
}

def trait Encrypted for T: type = {
    encrypt: (T) -> T,
    decrypt: (T) -> T,
}

def trait Logged for T: type = {
    log_access: (T) -> (),
}

# Function requiring data that is both encrypted and logged
def processSecureData(data: T) -> T for T: SecureData^Encrypted^Logged =
    data.log_access()
    let encrypted = data.encrypt()
    let processed = encrypted  # Do processing
    return processed.decrypt()

# This demonstrates end-lang's unique approach:
# - Structural typing for flexibility
# - Nominal anchoring for type identity  
# - Intersection types for capability composition
# - Retroactive trait implementation for extensibility