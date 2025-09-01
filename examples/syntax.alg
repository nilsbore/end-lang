# import a library
#import file

# ends with close statement or end of file
#open Examples1

def answer: i32 = 42

def times(a: T, b: T) -> T for T: type = a * b

#def times_lambda: (T, T) -> T for T: type = fn(a, b) => a*b

def mmul(a: tensor[T, M, N], b: tensor[T, N, L]) -> tensor[T, M, L] for T: type, L: size, M: size, N: size =
    let C : tensor[T, M, L] = tensor[T, M, L].zeros()
    for i in range(0, N):
        pass

def type Person = {
    name: String,
    address: String,
    age: u8,
}

def type Vec3 for T: type = {
    x: T,
    y: T,
    z: T,
}

#def enum Option for T: type =
def type Option for T: type = # trying comments
    | None
    | Some(T)

def isEmpty(v: Option[i32]) -> bool = match v:
    | None => False
    | Some(val) => True

def isEmpty(v: Option[i32]) -> bool =
    match v:
    | None => False
    | Some(val) => True

def ifTest(val: String) -> () =
    if val == "one":
        print(1)
    elif val == "two":
        print(2)
    elif val == "three":
        print(3)
    else:
        print("invalid")

def assignTest() -> () =
    let a = 2
    a = 1

def whileTest(iters: u32) -> () =
    let i = 0
    while i < iters:
        i = i + 1

# NOTE: the trait definition just defines
# what members and member function a type
# should have
def trait Print for T: type = {
    print: (T) -> str,
}

def trait ScreenPrint for T: Print = {
    screen_print: (T) -> (),
}

open ScreenPrint^T for T: Print

def screen_print(t: T) -> () =
    print(t.print())

close ScreenPrint^T

def trait Add for T: type = {
    add: (T, T) -> T,
}

def double(a: T) -> T for T: Add =
    return add(a, a)

open Add^i32

def add(a: i32, b: i32) -> i32 =
    a + b

close Add^i32

# type of i32 is now i32^Add, so it is a subtype of i32 and Add

def Add^Vec[T].add(a: Vec[T], b: Vec[T]) -> Vec[T] for T: Add =
    let c = Vec.new()
    for i in zip(a, b):
        c.append(i+j)
    return c

# equivalent to
open Add^Vec[T] for T: Add

def add(a: Vec[T], b: Vec[T]) -> Vec[T] =
    let c = Vec.new()
    for i in zip(a, b):
        c.append(i+j)
    return c

close Add^Vec[T]

def Person.changeAddress(self: Person, address: String) -> () =
    pass

# equivalent to
open Person

def changeAddress(self: Person, address: String) -> () =
    #self.address = address
    pass

close Person

def let_examples() -> () =

    let v: Vec3[f32] = Vec3(x=1, y=2, z=3)
    # identical to
    # let v: Vec3 of f32 = Vec3(x=1, y=2, z=3)

    # TODO: let f = fn x => x + 1
    let i = 2 # let g = fn(x: i32) => x + 1
    let i = 3
    
    let l: List[i32] = [1, 2, 3, 4, 5]
    # TODO: let l: List of i32 = [2*i for i in range(0, 9) if i != 5]
    
    let p = Person(name="joe", address="Main str 1", age=42)
    let p2: Person = {
        name="joe",
        address="Main str 1",
        age=42,
    }
    #p.changeAddress("High road 2")
    let one = Vec3[f32].ones()

