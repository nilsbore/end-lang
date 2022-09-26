# ends with End statement or end of file
begin namespace Examples1

def answer: i32 = 42

def times(a: T, b: T) -> T for T: type =
    a * b

def mmul(a: tensor<T, M, N>, b: tensor<T, N, L>) -> tensor<T, M, L> for T: type, L, M, N: size =
    let C : tensor<T, M, L> = tensor<T, M, L>. zeros()
    for i in range(0, N):
        ...
    return C

def mmul(a: [M, N: T], b: [N, L: T]) -> [M, L: T] for T: type, L, M, N: size =
    ...

end namespace Examples1

# makes functions above available for rest of file
open namespace Examples1

def struct Person =
    name: String
    address: String
    age: u8

def struct Vec3 for T: type =
    x: T
    y: T
    z: T

def enum Option for T: type =
    | None
    | Some T

def isEmpty(v: Option of i32) -> bool =
    match v:
    | None => False
    | Some _ => True    

let v: Vec3<f32> = Vec3(x=1, y=2, z=3)
# identical to
let v: Vec3 of f32 = Vec3(x=1, y=2, z=3)

let l: List of i32 = [2*i for i in range(0, 9) if i != 5]

let one = Vec3<f32>.ones()

def trait Print for T: type =
    print: (T) -> ()

def trait Add for T: type =
    add: (T, T) -> T

def impl Add for i32 =
    def add(a: i32, b: i32) =
        a + b

let f = fn x => x + 1
let g = fn(x: i32) => x + 1

# return last expression is disabled in void functions
def changeAddress(inout self: Person, address: String) =
    self.address = address

let p = Person(name="joe", address="Main str 1", age=42)
p.changeAddress("High str 2")
