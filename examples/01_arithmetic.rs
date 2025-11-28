use aegis_vm::vm_protect;

// Native implementation (for reference/IDA comparison)
#[inline(never)]
#[allow(clippy::manual_rotate)]
fn derive_key_native(seed: u64, salt: u64) -> u64 {
    let mut k = seed;
    k = (k ^ salt).wrapping_add(0x1234567890ABCDEF);
    k = (k << 5) | (k >> 59);
    k = k.wrapping_mul(0x5DEECE66D);
    k ^ 0xCAFEBABE
}

// VM-Protected implementation
// Use "paranoid" level to enable heavy MBA transformations
#[vm_protect(level = "paranoid")]
fn derive_key_vm(seed: u64, salt: u64) -> u64 {
    let mut k = seed;
    // In VM context, standard operators usually behave as wrapping in our implementation
    // but let's match the logic exactly.
    k = (k ^ salt) + 0x1234567890ABCDEF;
    k = (k << 5) | (k >> 59);
    k = k * 0x5DEECE66D;
    k ^ 0xCAFEBABE
}

fn main() {
    let seed = 0xDEADBEEF;
    let salt = 0xAABBCCDD;

    println!("--- Arithmetic Obfuscation Test ---");
    println!("Seed: 0x{:X}, Salt: 0x{:X}", seed, salt);

    let native = derive_key_native(seed, salt);
    println!("Native Result: 0x{:X}", native);

    let vm = derive_key_vm(seed, salt);
    println!("VM Result:     0x{:X}", vm);

    if native == vm {
        println!("[SUCCESS] Results match!");
    } else {
        println!("[FAILURE] Results differ!");
    }
}
