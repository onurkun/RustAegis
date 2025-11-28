use aegis_vm::vm_protect;

// Native implementation
#[inline(never)]
fn fibonacci_native(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    let mut a = 0;
    let mut b = 1;

    for _ in 2..n + 1 {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}

// VM-Protected implementation
#[vm_protect(level = "standard")]
fn fibonacci_vm(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    let mut a = 0;
    let mut b = 1;

    // The VM macro handles range-based for loops
    for _i in 2..n + 1 {
        let temp = a + b;
        a = b;
        b = temp;
    }
    b
}

fn main() {
    let n = 20;

    println!("--- Loop Virtualization Test ---");
    println!("Calculating Fibonacci({})", n);

    let native = fibonacci_native(n);
    println!("Native Result: {}", native);

    let vm = fibonacci_vm(n);
    println!("VM Result:     {}", vm);
}
