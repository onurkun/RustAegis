use aegis_vm::vm_protect;

// Native implementation
#[inline(never)]
fn check_license_native(key: u64) -> bool {
    // Simulated license check logic
    if key > 10000 {
        if key.is_multiple_of(2) {
            if (key & 0xFF) == 0xAA {
                return true;
            }
        } else {
            // Odd numbers > 10000
            if (key & 0xF) == 0x3 {
                return true;
            }
        }
    }
    false
}

// VM-Protected implementation
#[vm_protect(level = "standard")]
fn check_license_vm(key: u64) -> bool {
    // VM executes this logic via bytecode interpretation
    // Encrypted control flow graph
    if key > 10000 {
        if (key % 2) == 0 {
            if (key & 0xFF) == 0xAA {
                return true;
            }
        } else {
            if (key & 0xF) == 0x3 {
                return true;
            }
        }
    }
    false
}

fn main() {
    let valid_key_1 = 10240 + 0xAA; // Even path
    let valid_key_2 = 10003; // Odd path
    let invalid_key = 500;

    println!("--- Control Flow Protection Test ---");

    println!("Checking Key 1 (Native): {}", check_license_native(valid_key_1));
    println!("Checking Key 1 (VM):     {}", check_license_vm(valid_key_1));

    println!("Checking Key 2 (Native): {}", check_license_native(valid_key_2));
    println!("Checking Key 2 (VM):     {}", check_license_vm(valid_key_2));

    println!("Checking Invalid (Native): {}", check_license_native(invalid_key));
    println!("Checking Invalid (VM):     {}", check_license_vm(invalid_key));
}
