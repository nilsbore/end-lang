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
    return C

#TODO: def mmul(a: [M, N: T], b: [N, L: T]) -> [M, L: T] for T: type, L, M, N: size =
#    pass

#end namespace Examples1

# makes functions above available for rest of file
#open namespace Examples1

def struct Person =
    name: String
    address: String
    age: u8

def struct Vec3 for T: type =
    x: T
    y: T
    z: T
# next version altearnative syntax:
# def struct Vec3 for T: type = x: T, y: T, z: T

def enum Option for T: type =
    | None
    | Some(T)
# next version altearnative syntax:
# def enum Option for T: type = None | Some(T)


def isEmpty(v: Option of i32) -> bool = match v:
    | None => False
    | Some(_) => True

def trait Print for T: type =
    print: (T) -> ()

def trait Add for T: type = # this is all good
    add: (T, T) -> T

def impl Add for i32 =
    def add(a: i32, b: i32) -> i32 =
        a + b

# return last expression is disabled in void functions
def changeAddress(self: Person, address: String) =
    #self.address = address
    pass

def let_examples() -> () =

    # TODO: let f = fn x => x + 1
    # let g = fn(x: i32) => x + 1

    let v: Vec3[f32] = Vec3(x=1, y=2, z=3)
    # identical to
    # let v: Vec3 of f32 = Vec3(x=1, y=2, z=3)

    let l: List[i32] = [1, 2, 3, 4, 5]
    # TODO: let l: List of i32 = [2*i for i in range(0, 9) if i != 5]

    let one = Vec3[f32]::ones()

    let p = Person(name="joe", address="Main str 1", age=42)
    #p.changeAddress("High str 2")
