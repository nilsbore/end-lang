# import a library
#import file

# ends with End statement or end of file
#begin namespace Examples1

def answer: i32 = 42

def times(a: T, b: T) -> T for T: type = a * b

#def times_lambda: (T, T) -> T for T: type = fn(a, b) => a*b

def mmul(a: tensor[T, M, N], b: tensor[T, N, L]) -> tensor[T, M, L] for T: type, L: size, M: size, N: size =
    let C : tensor[T, M, L] = tensor[T, M, L]::zeros()
    for i in range(0, N):
        pass

def struct Person =
    name: String
    address: String
    age: u8

def struct Vec for T: type =
    x: T
    y: T
    z: T

def enum Option for T: type =
    | None
    | Some(T)

def isEmpty(v: Option[i32]) -> bool = match v:
    | None => False
    | Some(val) => True

def isEmpty(v: Option[i32]) -> bool =
    match v:
        | None => False
        | Some(val) => True
    pass

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

def whileTest(iters: uint32) -> () =
    let i = 0
    while i < iters:
        i = i + 1

def trait Print for T: type =
    print: (T) -> ()

def trait Add for T: type =
    add: (T, T) -> T

def impl Add for i32 =
    def add(a: i32, b: i32) -> i32 =
        a + b

def changeAddress(self: Person, address: String) -> () =
    #self.address = address
    pass

def let_examples() -> () =

    let v: Vec3[f32] = Vec3(x=1, y=2, z=3)
    # identical to
    # let v: Vec3 of f32 = Vec3(x=1, y=2, z=3)

    # TODO: let f = fn x => x + 1
    # let g = fn(x: i32) => x + 1

    let l: List[i32] = [1, 2, 3, 4, 5]
    # TODO: let l: List of i32 = [2*i for i in range(0, 9) if i != 5]

    let p = Person(name="joe", address="Main str 1", age=42)
    #p.changeAddress("High road 2")
    let one = Vec3[f32]::ones()

