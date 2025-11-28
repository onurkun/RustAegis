//! Build script for anticheat-vm
//!
//! Generates compile-time constants including:
//! - BUILD_SEED: Unique per-build encryption key seed
//! - BUILD_ID: Derived from BUILD_SEED for watermarking
//! - BUILD_TIMESTAMP: Unix timestamp of build
//! - CUSTOMER_ID: Customer identifier for build tracking
//! - WATERMARK: 128-bit steganographic watermark
//! - OPCODE_TABLE: Shuffled opcode mapping for polymorphic VM

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Generate build configuration
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("build_config.rs");
    let mut f = File::create(&dest_path).expect("Could not create build_config.rs");

    // Get build timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    // Generate BUILD_SEED
    // In production: Use ANTICHEAT_BUILD_KEY environment variable
    // In dev: Generate from timestamp + random data
    let build_seed = generate_build_seed();

    // Derive BUILD_ID from seed (simple FNV-1a for build script)
    let build_id = derive_build_id(&build_seed);

    // Get customer ID for watermarking (identifies the SDK licensee)
    let customer_id = env::var("ANTICHEAT_CUSTOMER_ID")
        .unwrap_or_else(|_| "dev-customer".to_string());

    // Generate watermark from customer + build info
    let watermark = generate_watermark(&customer_id, &build_seed, timestamp);

    // Write generated constants
    writeln!(f, "/// Build timestamp (Unix epoch seconds)").unwrap();
    writeln!(f, "pub const BUILD_TIMESTAMP: u64 = {};", timestamp).unwrap();
    writeln!(f).unwrap();

    // --- DYNAMIC SEED GENERATION START ---
    // Instead of writing the seed directly, we generate a function to reconstruct it.
    // This prevents the seed from appearing as a contiguous block in .rodata.
    // Combined with OLLVM, this logic becomes extremely hard to reverse.

    // 1. Generate Entropy Pool (1024 bytes of junk)
    let mut entropy_pool = [0u8; 1024];
    let pool_seed = generate_random_seed(); // Seed for the pool itself
    let mut rng_state = pool_seed;
    for (i, byte) in entropy_pool.iter_mut().enumerate() {
        // Simple LCG for pool generation
        let mac = hmac_sha256(&rng_state, &(i as u32).to_le_bytes());
        *byte = mac[0];
        // Update state occasionally
        if i % 32 == 0 {
            rng_state = mac;
        }
    }

    // 2. Calculate Delta values to reconstruct the real seed
    // Algorithm: seed[i] = pool[(start + i * step) % 1024] ^ delta[i]
    // So: delta[i] = seed[i] ^ pool[(start + i * step) % 1024]
    
    // Generate random parameters for the access pattern
    let rnd = generate_random_seed();
    let start_offset = (u64::from_le_bytes(rnd[0..8].try_into().unwrap()) % 800) as usize;
    let step = (u64::from_le_bytes(rnd[8..16].try_into().unwrap()) % 20 + 1) as usize; // 1..21

    let mut deltas = [0u8; 32];
    for i in 0..32 {
        let pool_idx = (start_offset + i * step) % 1024;
        deltas[i] = build_seed[i] ^ entropy_pool[pool_idx];
    }

    // 3. Write the Entropy Pool constant
    writeln!(f, "/// Entropy pool for seed reconstruction").unwrap();
    writeln!(f, "const ENTROPY_POOL: [u8; 1024] = [").unwrap();
    for (i, byte) in entropy_pool.iter().enumerate() {
        if i % 16 == 0 { write!(f, "\n    ").unwrap(); }
        write!(f, "0x{:02x}, ", byte).unwrap();
    }
    writeln!(f, "\n];").unwrap();
    writeln!(f).unwrap();

    // 4. Write the Delta Array
    writeln!(f, "/// Delta values for seed reconstruction").unwrap();
    writeln!(f, "const SEED_DELTAS: [u8; 32] = [").unwrap();
    for (i, byte) in deltas.iter().enumerate() {
        if i > 0 { write!(f, ", ").unwrap(); }
        write!(f, "0x{:02x}", byte).unwrap();
    }
    writeln!(f, "];").unwrap();
    writeln!(f).unwrap();

    // 5. Write the Reconstruction Function
    // We use std::hint::black_box to prevent constant folding.
    // We mark it #[inline(always)] so OLLVM obfuscates it at every call site.
    writeln!(f, "/// Reconstruct the build seed at runtime").unwrap();
    writeln!(f, "/// This prevents the seed from existing plainly in binary").unwrap();
    writeln!(f, "#[inline(always)]").unwrap();
    writeln!(f, "pub fn get_build_seed() -> [u8; 32] {{").unwrap();
    writeln!(f, "    let mut seed = [0u8; 32];").unwrap();
    // Use black_box on the parameters
    writeln!(f, "    let start = std::hint::black_box({});", start_offset).unwrap();
    writeln!(f, "    let step = std::hint::black_box({});", step).unwrap();
    writeln!(f, "    ").unwrap();
    writeln!(f, "    for i in 0..32 {{").unwrap();
    writeln!(f, "        let idx = (start + i * step) % 1024;").unwrap();
    writeln!(f, "        // The XOR operation combined with table lookups").unwrap();
    writeln!(f, "        seed[i] = ENTROPY_POOL[idx] ^ SEED_DELTAS[i];").unwrap();
    writeln!(f, "    }}").unwrap();
    writeln!(f, "    seed").unwrap();
    writeln!(f, "}}").unwrap();
    writeln!(f).unwrap();
    
    // Note: We REMOVED the 'pub const BUILD_SEED' definition.
    // --- DYNAMIC SEED GENERATION END ---

    writeln!(f, "/// Build ID derived from BUILD_SEED (for watermarking)").unwrap();
    writeln!(f, "pub const BUILD_ID: u64 = 0x{:016x};", build_id).unwrap();
    writeln!(f).unwrap();

    // Customer ID
    writeln!(f, "/// Customer ID for build tracking (from ANTICHEAT_CUSTOMER_ID env)").unwrap();
    writeln!(f, "pub const CUSTOMER_ID: &str = \"{}\";", customer_id).unwrap();
    writeln!(f).unwrap();

    // Watermark (128-bit)
    writeln!(f, "/// 128-bit steganographic watermark derived from customer + build").unwrap();
    writeln!(f, "/// Used for identifying leaked builds").unwrap();
    write!(f, "pub const WATERMARK: [u8; 16] = [").unwrap();
    for (i, byte) in watermark.iter().enumerate() {
        if i > 0 {
            write!(f, ", ").unwrap();
        }
        write!(f, "0x{:02x}", byte).unwrap();
    }
    writeln!(f, "];").unwrap();
    writeln!(f).unwrap();

    // Watermark as two u64 for easy embedding
    let watermark_hi = u64::from_le_bytes([
        watermark[0], watermark[1], watermark[2], watermark[3],
        watermark[4], watermark[5], watermark[6], watermark[7],
    ]);
    let watermark_lo = u64::from_le_bytes([
        watermark[8], watermark[9], watermark[10], watermark[11],
        watermark[12], watermark[13], watermark[14], watermark[15],
    ]);
    writeln!(f, "/// Watermark high 64 bits").unwrap();
    writeln!(f, "pub const WATERMARK_HI: u64 = 0x{:016x};", watermark_hi).unwrap();
    writeln!(f, "/// Watermark low 64 bits").unwrap();
    writeln!(f, "pub const WATERMARK_LO: u64 = 0x{:016x};", watermark_lo).unwrap();
    writeln!(f).unwrap();

    // Git commit hash if available
    if let Some(git_hash) = get_git_hash() {
        writeln!(f, "/// Git commit hash (first 16 hex chars)").unwrap();
        writeln!(f, "pub const GIT_COMMIT: &str = \"{}\";", git_hash).unwrap();
    } else {
        writeln!(f, "/// Git commit hash (not available)").unwrap();
        writeln!(f, "pub const GIT_COMMIT: &str = \"unknown\";").unwrap();
    }
    writeln!(f).unwrap();

    // Protection level from environment
    let protection_level = env::var("ANTICHEAT_PROTECTION_LEVEL")
        .unwrap_or_else(|_| "medium".to_string());
    writeln!(f, "/// Protection level (from ANTICHEAT_PROTECTION_LEVEL env)").unwrap();
    writeln!(f, "pub const PROTECTION_LEVEL: &str = \"{}\";", protection_level).unwrap();
    writeln!(f).unwrap();

    // Build sequence number (for tracking multiple builds in same session)
    let build_seq = env::var("ANTICHEAT_BUILD_SEQ")
        .unwrap_or_else(|_| "0".to_string())
        .parse::<u32>()
        .unwrap_or(0);
    writeln!(f, "/// Build sequence number (from ANTICHEAT_BUILD_SEQ env)").unwrap();
    writeln!(f, "pub const BUILD_SEQ: u32 = {};", build_seq).unwrap();
    writeln!(f).unwrap();

    // Generate shuffled opcode table
    let opcode_table = generate_opcode_table(&build_seed);
    write_opcode_table(&mut f, &opcode_table);

    // Generate randomized MAGIC bytes for bytecode header
    let magic_bytes = generate_magic_bytes(&build_seed);
    write_magic_bytes(&mut f, &magic_bytes);

    // Generate shuffled native function IDs
    let native_ids = generate_native_ids(&build_seed);
    write_native_ids(&mut f, &native_ids);

    // Generate shuffled register mapping
    let register_map = generate_register_map(&build_seed);
    write_register_map(&mut f, &register_map);

    // Generate custom FNV hash constants
    let fnv_constants = generate_fnv_constants(&build_seed);
    write_fnv_constants(&mut f, &fnv_constants);

    // Generate randomized XOR key for domain string obfuscation
    let xor_key = generate_xor_key(&build_seed);
    write_xor_key(&mut f, xor_key);

    // Generate shuffled flag bit positions
    let flag_bits = generate_flag_bits(&build_seed);
    write_flag_bits(&mut f, &flag_bits);

    // Write build history for debugging
    write_build_history(
        &build_seed, build_id, timestamp, &customer_id, &opcode_table,
        &magic_bytes, &native_ids, &register_map, &fnv_constants, xor_key, &flag_bits
    );

    // Rerun conditions
    println!("cargo:rerun-if-env-changed=ANTICHEAT_BUILD_KEY");
    println!("cargo:rerun-if-env-changed=ANTICHEAT_PROTECTION_LEVEL");
    println!("cargo:rerun-if-env-changed=ANTICHEAT_CUSTOMER_ID");
    println!("cargo:rerun-if-env-changed=ANTICHEAT_BUILD_SEQ");
    println!("cargo:rerun-if-changed=build.rs");
}

/// Generate watermark from customer ID, build seed, and timestamp
/// The watermark is designed to be:
/// 1. Unique per customer+build combination
/// 2. Verifiable server-side to identify leaked builds
/// 3. Spread across the binary via steganographic embedding
fn generate_watermark(customer_id: &str, build_seed: &[u8; 32], timestamp: u64) -> [u8; 16] {
    // Create watermark input: customer_id || build_seed || timestamp
    let mut input = Vec::new();
    input.extend_from_slice(customer_id.as_bytes());
    input.extend_from_slice(build_seed);
    input.extend_from_slice(&timestamp.to_le_bytes());
    input.extend_from_slice(b"watermark-v1");

    // Hash to get watermark
    let hash = sha256(&input);

    // Take first 16 bytes as watermark
    let mut watermark = [0u8; 16];
    watermark.copy_from_slice(&hash[..16]);
    watermark
}

/// Generate build seed from environment or random
/// The seed is also written to a shared file so vm-macro can read it
fn generate_build_seed() -> [u8; 32] {
    // Check for explicit build key (for reproducible builds)
    if let Ok(key) = env::var("ANTICHEAT_BUILD_KEY") {
        // Use HMAC(build_key, seed_domain)
        // This ensures reproducible but secure seeds
        let seed = hmac_sha256(key.as_bytes(), b"anticheat-vm-seed-v1");
        write_shared_seed(&seed);
        return seed;
    }

    // No explicit key - generate random seed for this build
    // Each build will have unique opcodes, encryption, etc.
    let seed = generate_random_seed();
    write_shared_seed(&seed);
    seed
}

/// Write seed to shared location for vm-macro to read
fn write_shared_seed(seed: &[u8; 32]) {
    // Write to target directory so vm-macro can find it
    if let Ok(out_dir) = env::var("OUT_DIR") {
        // OUT_DIR is like:
        //   target/debug/build/anticheat-vm-xxx/out (native)
        //   target/aarch64-linux-android/debug/build/xxx/out (cross-compile)
        // Find the actual "target" directory (not debug/release inside it)
        let target_dir = Path::new(&out_dir)
            .ancestors()
            .find(|p| p.file_name().is_some_and(|n| n == "target"));

        if let Some(target) = target_dir {
            let seed_file = target.join(".anticheat_build_seed");
            if let Ok(mut f) = File::create(&seed_file) {
                // Write as hex
                for byte in seed {
                    let _ = write!(f, "{:02x}", byte);
                }
            }
        }
    }
}

/// Write build history to file for debugging/inspection
#[allow(clippy::too_many_arguments)]
fn write_build_history(
    seed: &[u8; 32],
    build_id: u64,
    timestamp: u64,
    customer_id: &str,
    opcode_table: &OpcodeTable,
    magic_bytes: &[u8; 4],
    native_ids: &NativeIdMap,
    register_map: &RegisterMap,
    fnv_constants: &FnvConstants,
    xor_key: u8,
    flag_bits: &FlagBits,
) {
    use std::fs::OpenOptions;

    // Find project root (where Cargo.toml is)
    let history_path = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        Path::new(&manifest_dir).parent().and_then(|p| p.parent())
            .map(|p| p.join("build_history.txt"))
    } else {
        Some(Path::new("build_history.txt").to_path_buf())
    };

    let Some(history_path) = history_path else { return };

    let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history_path) else { return };

    // Format timestamp as human readable
    let datetime = format_timestamp(timestamp);

    writeln!(file, "================================================================================").ok();
    writeln!(file, "BUILD: {}", datetime).ok();
    writeln!(file, "================================================================================").ok();
    writeln!(file, "Timestamp:   {} ({})", timestamp, datetime).ok();
    writeln!(file, "Customer ID: {}", customer_id).ok();
    writeln!(file, "Build ID:    0x{:016x}", build_id).ok();
    writeln!(file).ok();

    // Seed as hex
    write!(file, "Seed: ").ok();
    for byte in seed {
        write!(file, "{:02x}", byte).ok();
    }
    writeln!(file).ok();
    writeln!(file).ok();

    // MAGIC bytes
    writeln!(file, "MAGIC Bytes: 0x{:02x} 0x{:02x} 0x{:02x} 0x{:02x}",
             magic_bytes[0], magic_bytes[1], magic_bytes[2], magic_bytes[3]).ok();
    writeln!(file).ok();

    // Important opcodes (shuffled values)
    writeln!(file, "Shuffled Opcodes (base -> shuffled):").ok();
    writeln!(file, "  PUSH_IMM8:  0x02 -> 0x{:02x}", opcode_table.encode[0x02]).ok();
    writeln!(file, "  PUSH_IMM:   0x01 -> 0x{:02x}", opcode_table.encode[0x01]).ok();
    writeln!(file, "  DROP:       0x07 -> 0x{:02x}", opcode_table.encode[0x07]).ok();
    writeln!(file, "  ADD:        0x20 -> 0x{:02x}", opcode_table.encode[0x20]).ok();
    writeln!(file, "  SUB:        0x21 -> 0x{:02x}", opcode_table.encode[0x21]).ok();
    writeln!(file, "  MUL:        0x22 -> 0x{:02x}", opcode_table.encode[0x22]).ok();
    writeln!(file, "  XOR:        0x23 -> 0x{:02x}", opcode_table.encode[0x23]).ok();
    writeln!(file, "  CMP:        0x30 -> 0x{:02x}", opcode_table.encode[0x30]).ok();
    writeln!(file, "  JMP:        0x31 -> 0x{:02x}", opcode_table.encode[0x31]).ok();
    writeln!(file, "  JZ:         0x32 -> 0x{:02x}", opcode_table.encode[0x32]).ok();
    writeln!(file, "  JNZ:        0x33 -> 0x{:02x}", opcode_table.encode[0x33]).ok();
    writeln!(file, "  NOP:        0x40 -> 0x{:02x}", opcode_table.encode[0x40]).ok();
    writeln!(file, "  HALT:       0xFF -> 0xff (fixed)").ok();
    writeln!(file).ok();

    // Handler Duplication aliases
    writeln!(file, "Handler Duplication (aliases that decode to same base):").ok();
    for &base in DUPLICATED_OPCODES {
        let base_name = match base {
            0x20 => "ADD",
            0x21 => "SUB",
            0x23 => "XOR",
            0x24 => "AND",
            0x25 => "OR",
            0x30 => "CMP",
            _ => "???",
        };
        let primary = opcode_table.encode[base as usize];
        if let Some(aliases) = opcode_table.aliases.get(&base) {
            let alias_str: Vec<String> = aliases.iter().map(|a| format!("0x{:02x}", a)).collect();
            writeln!(file, "  {}: primary=0x{:02x}, aliases=[{}]",
                     base_name, primary, alias_str.join(", ")).ok();
        }
    }
    writeln!(file).ok();

    // Native function IDs
    writeln!(file, "Native Function IDs:").ok();
    writeln!(file, "  CHECK_ROOT:      {}", native_ids.check_root).ok();
    writeln!(file, "  CHECK_EMULATOR:  {}", native_ids.check_emulator).ok();
    writeln!(file, "  CHECK_HOOKS:     {}", native_ids.check_hooks).ok();
    writeln!(file, "  CHECK_DEBUGGER:  {}", native_ids.check_debugger).ok();
    writeln!(file, "  CHECK_TAMPER:    {}", native_ids.check_tamper).ok();
    writeln!(file, "  GET_TIMESTAMP:   {}", native_ids.get_timestamp).ok();
    writeln!(file, "  HASH_FNV1A:      {}", native_ids.hash_fnv1a).ok();
    writeln!(file, "  READ_MEMORY:     {}", native_ids.read_memory).ok();
    writeln!(file, "  GET_DEVICE_HASH: {}", native_ids.get_device_hash).ok();
    writeln!(file).ok();

    // Register mapping
    writeln!(file, "Register Mapping (logical -> physical):").ok();
    for i in 0..8 {
        writeln!(file, "  R{} -> physical {}", i, register_map.map[i]).ok();
    }
    writeln!(file).ok();

    // FNV constants
    writeln!(file, "FNV Hash Constants:").ok();
    writeln!(file, "  BASIS_64: 0x{:016x}", fnv_constants.basis_64).ok();
    writeln!(file, "  PRIME_64: 0x{:016x}", fnv_constants.prime_64).ok();
    writeln!(file, "  BASIS_32: 0x{:08x}", fnv_constants.basis_32).ok();
    writeln!(file, "  PRIME_32: 0x{:08x}", fnv_constants.prime_32).ok();
    writeln!(file).ok();

    // XOR key
    writeln!(file, "XOR Obfuscation Key: 0x{:02x}", xor_key).ok();
    writeln!(file).ok();

    // Flag bits
    writeln!(file, "Flag Bit Positions:").ok();
    writeln!(file, "  ZERO:     0b{:08b}", flag_bits.zero).ok();
    writeln!(file, "  CARRY:    0b{:08b}", flag_bits.carry).ok();
    writeln!(file, "  OVERFLOW: 0b{:08b}", flag_bits.overflow).ok();
    writeln!(file, "  SIGN:     0b{:08b}", flag_bits.sign).ok();
    writeln!(file).ok();
    writeln!(file).ok();
}

/// Format unix timestamp as human readable string
fn format_timestamp(timestamp: u64) -> String {
    let secs = timestamp;
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Simple date calculation (not accounting for leap seconds)
    let mut year = 1970;
    let mut remaining_days = days_since_epoch;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months: [u64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days in days_in_months.iter() {
        if remaining_days < *days {
            break;
        }
        remaining_days -= days;
        month += 1;
    }

    let day = remaining_days + 1;

    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
            year, month, day, hours, minutes, seconds)
}

fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// Generate cryptographically random seed
fn generate_random_seed() -> [u8; 32] {
    use std::io::Read;

    let mut seed = [0u8; 32];

    // Try /dev/urandom first (Unix)
    if let Ok(mut file) = File::open("/dev/urandom") {
        if file.read_exact(&mut seed).is_ok() {
            return seed;
        }
    }

    // Fallback: combine multiple entropy sources
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let mut entropy = Vec::new();
    entropy.extend_from_slice(&timestamp.as_nanos().to_le_bytes());
    entropy.extend_from_slice(&std::process::id().to_le_bytes());

    // Add some environment entropy
    if let Ok(pwd) = env::var("PWD") {
        entropy.extend_from_slice(pwd.as_bytes());
    }
    if let Ok(user) = env::var("USER") {
        entropy.extend_from_slice(user.as_bytes());
    }

    // Hash for uniform distribution
    sha256(&entropy)
}

/// Derive build ID from seed using HMAC-SHA256
/// Must match derive_build_id in vm-macro/src/crypto.rs
fn derive_build_id(seed: &[u8; 32]) -> u64 {
    // Domain string for build ID derivation
    const BUILDID_DOMAIN: &[u8] = b"anticheat-vm-build-id-v1";
    let hmac_result = hmac_sha256(seed, BUILDID_DOMAIN);
    u64::from_le_bytes([
        hmac_result[0], hmac_result[1], hmac_result[2], hmac_result[3],
        hmac_result[4], hmac_result[5], hmac_result[6], hmac_result[7],
    ])
}

/// Get git commit hash (first 16 chars)
fn get_git_hash() -> Option<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["rev-parse", "--short=16", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        let hash = String::from_utf8_lossy(&output.stdout);
        Some(hash.trim().to_string())
    } else {
        None
    }
}

/// Simple SHA-256 implementation for build script (no external deps)
fn sha256(data: &[u8]) -> [u8; 32] {
    // Initial hash values (first 32 bits of fractional parts of square roots of first 8 primes)
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    // Round constants
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    // Pre-processing: adding padding bits
    let ml = (data.len() as u64) * 8;
    let mut padded = data.to_vec();
    padded.push(0x80);

    while (padded.len() % 64) != 56 {
        padded.push(0x00);
    }

    // Append original length in bits as 64-bit big-endian
    padded.extend_from_slice(&ml.to_be_bytes());

    // Process each 512-bit (64-byte) chunk
    for chunk in padded.chunks(64) {
        let mut w = [0u32; 64];

        // Break chunk into 16 32-bit big-endian words
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }

        // Extend the first 16 words into the remaining 48 words
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        // Initialize working variables
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);

        // Compression function main loop
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        // Add compressed chunk to current hash value
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    // Produce final hash value (big-endian)
    let mut result = [0u8; 32];
    for (i, &word) in h.iter().enumerate() {
        result[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    result
}

/// Base opcode definitions (canonical values)
/// These are the "logical" opcodes that the compiler uses
#[rustfmt::skip]
const BASE_OPCODES: &[(& str, &str, u8)] = &[
    // Stack operations
    ("stack", "PUSH_IMM", 0x01),
    ("stack", "PUSH_IMM8", 0x02),
    ("stack", "PUSH_REG", 0x03),
    ("stack", "POP_REG", 0x04),
    ("stack", "DUP", 0x05),
    ("stack", "SWAP", 0x06),
    ("stack", "DROP", 0x07),
    ("stack", "PUSH_IMM16", 0x08),
    ("stack", "PUSH_IMM32", 0x09),
    // Register operations
    ("register", "MOV_IMM", 0x10),
    ("register", "MOV_REG", 0x11),
    ("register", "LOAD_MEM", 0x12),
    ("register", "STORE_MEM", 0x13),
    // Arithmetic operations
    ("arithmetic", "ADD", 0x20),
    ("arithmetic", "SUB", 0x21),
    ("arithmetic", "MUL", 0x22),
    ("arithmetic", "XOR", 0x23),
    ("arithmetic", "AND", 0x24),
    ("arithmetic", "OR", 0x25),
    ("arithmetic", "SHL", 0x26),
    ("arithmetic", "SHR", 0x27),
    ("arithmetic", "NOT", 0x28),
    ("arithmetic", "ROL", 0x29),
    ("arithmetic", "ROR", 0x2A),
    ("arithmetic", "INC", 0x2B),
    ("arithmetic", "DEC", 0x2C),
    ("arithmetic", "DIV", 0x46),
    ("arithmetic", "MOD", 0x47),
    ("arithmetic", "IDIV", 0x48),
    ("arithmetic", "IMOD", 0x49),
    // Control flow
    ("control", "CMP", 0x30),
    ("control", "JMP", 0x31),
    ("control", "JZ", 0x32),
    ("control", "JNZ", 0x33),
    ("control", "JGT", 0x34),
    ("control", "JLT", 0x35),
    ("control", "JGE", 0x36),
    ("control", "JLE", 0x37),
    ("control", "CALL", 0x38),
    ("control", "RET", 0x39),
    // Special operations
    ("special", "NOP", 0x40),
    ("special", "NOP_N", 0x41),
    ("special", "OPAQUE_TRUE", 0x42),
    ("special", "OPAQUE_FALSE", 0x43),
    ("special", "HASH_CHECK", 0x44),
    ("special", "TIMING_CHECK", 0x45),
    // Type conversion
    ("convert", "SEXT8", 0x50),
    ("convert", "SEXT16", 0x51),
    ("convert", "SEXT32", 0x52),
    ("convert", "TRUNC8", 0x53),
    ("convert", "TRUNC16", 0x54),
    ("convert", "TRUNC32", 0x55),
    // Memory operations
    ("memory", "LOAD8", 0x60),
    ("memory", "LOAD16", 0x61),
    ("memory", "LOAD32", 0x62),
    ("memory", "LOAD64", 0x63),
    ("memory", "STORE8", 0x64),
    ("memory", "STORE16", 0x65),
    ("memory", "STORE32", 0x66),
    ("memory", "STORE64", 0x67),
    // Native calls
    ("native", "NATIVE_CALL", 0xF0),
    ("native", "NATIVE_READ", 0xF1),
    ("native", "NATIVE_WRITE", 0xF2),
    ("native", "INPUT_LEN", 0xF3),
    // Execution control
    ("exec", "HALT", 0xFF),
    ("exec", "HALT_ERR", 0xFE),
];

/// Critical opcodes that get handler duplication (alias opcodes)
/// Each critical opcode gets 2 additional aliases that decode to the same base value
const DUPLICATED_OPCODES: &[u8] = &[
    0x20, // ADD - most common arithmetic
    0x21, // SUB
    0x23, // XOR - used heavily in MBA
    0x24, // AND
    0x25, // OR
    0x30, // CMP - control flow critical
];

/// Generate shuffled opcode table from build seed
/// Returns a mapping: shuffled_value -> base_value (for runtime decode)
/// And we also need: base_value -> shuffled_value (for compile-time encode)
///
/// Handler Duplication: Critical opcodes (ADD, SUB, XOR, CMP) get multiple
/// shuffled values that all decode to the same base opcode. This confuses
/// reverse engineers who see different opcodes doing the same thing.
fn generate_opcode_table(seed: &[u8; 32]) -> OpcodeTable {
    // Derive shuffle key from seed
    let shuffle_key = hmac_sha256(seed, b"opcode-shuffle-v1");

    // Create list of available byte values (0x00-0xFD, excluding 0xFE and 0xFF for HALT)
    // We keep HALT and HALT_ERR fixed for simplicity in error handling
    let mut available: Vec<u8> = (0x00..0xFE).collect();

    // Fisher-Yates shuffle using HMAC-derived randomness
    let mut rng_state = shuffle_key;
    for i in (1..available.len()).rev() {
        // Get next random index
        let rand_bytes = hmac_sha256(&rng_state, &(i as u32).to_le_bytes());
        rng_state = rand_bytes;
        let j = (u64::from_le_bytes([
            rand_bytes[0], rand_bytes[1], rand_bytes[2], rand_bytes[3],
            rand_bytes[4], rand_bytes[5], rand_bytes[6], rand_bytes[7],
        ]) as usize) % (i + 1);
        available.swap(i, j);
    }

    // Build the mapping tables
    let mut encode = [0u8; 256]; // base -> shuffled (primary)
    let mut decode = [0u8; 256]; // shuffled -> base

    // Initialize as identity mapping
    for i in 0..256 {
        encode[i] = i as u8;
        decode[i] = i as u8;
    }

    // Track alias opcodes for each duplicated base opcode
    // aliases[base_opcode] = [alias1, alias2] (additional shuffled values)
    let mut aliases: std::collections::HashMap<u8, Vec<u8>> = std::collections::HashMap::new();

    // Assign shuffled values to each base opcode (except HALT/HALT_ERR)
    let mut available_idx = 0;
    for (_, _, base_val) in BASE_OPCODES.iter() {
        if *base_val == 0xFF || *base_val == 0xFE {
            // Keep HALT and HALT_ERR fixed
            continue;
        }
        let shuffled_val = available[available_idx];
        available_idx += 1;

        encode[*base_val as usize] = shuffled_val;
        decode[shuffled_val as usize] = *base_val;
    }

    // Handler Duplication: Assign additional aliases for critical opcodes
    // These aliases also decode to the same base opcode
    for &base_val in DUPLICATED_OPCODES {
        let mut op_aliases = Vec::new();

        // Assign 2 additional aliases per critical opcode
        for _ in 0..2 {
            if available_idx < available.len() {
                let alias_shuffled = available[available_idx];
                available_idx += 1;

                // This alias decodes to the same base opcode
                decode[alias_shuffled as usize] = base_val;
                op_aliases.push(alias_shuffled);
            }
        }

        if !op_aliases.is_empty() {
            aliases.insert(base_val, op_aliases);
        }
    }

    OpcodeTable { encode, decode, aliases }
}

/// Opcode table with handler duplication support
struct OpcodeTable {
    encode: [u8; 256], // base opcode -> shuffled opcode (for compiler)
    decode: [u8; 256], // shuffled opcode -> base opcode (for runtime)
    aliases: std::collections::HashMap<u8, Vec<u8>>, // base -> additional shuffled values
}

/// Write opcode table to generated file
fn write_opcode_table(f: &mut File, table: &OpcodeTable) {
    writeln!(f, "/// Opcode encoding table (base -> shuffled)").unwrap();
    writeln!(f, "/// Used by vm-macro at compile time").unwrap();
    write!(f, "pub const OPCODE_ENCODE: [u8; 256] = [").unwrap();
    for (i, &val) in table.encode.iter().enumerate() {
        if i % 16 == 0 {
            write!(f, "\n    ").unwrap();
        }
        write!(f, "0x{:02x}, ", val).unwrap();
    }
    writeln!(f, "\n];").unwrap();
    writeln!(f).unwrap();

    writeln!(f, "/// Opcode decoding table (shuffled -> base)").unwrap();
    writeln!(f, "/// Used by VM engine at runtime").unwrap();
    writeln!(f, "/// Note: Multiple shuffled values may decode to the same base (handler duplication)").unwrap();
    write!(f, "pub const OPCODE_DECODE: [u8; 256] = [").unwrap();
    for (i, &val) in table.decode.iter().enumerate() {
        if i % 16 == 0 {
            write!(f, "\n    ").unwrap();
        }
        write!(f, "0x{:02x}, ", val).unwrap();
    }
    writeln!(f, "\n];").unwrap();
    writeln!(f).unwrap();

    // Write alias information for vm-macro to use during polymorphic code generation
    writeln!(f, "/// Handler duplication aliases (base opcode -> additional shuffled values)").unwrap();
    writeln!(f, "/// These decode to the same base opcode, confusing reverse engineers").unwrap();
    writeln!(f, "pub mod opcode_aliases {{").unwrap();

    // Get opcode name from base value
    let get_name = |base: u8| -> &'static str {
        for (_, name, val) in BASE_OPCODES.iter() {
            if *val == base { return name; }
        }
        "UNKNOWN"
    };

    for &base in DUPLICATED_OPCODES {
        if let Some(aliases) = table.aliases.get(&base) {
            let name = get_name(base);
            write!(f, "    pub const {}_ALIASES: &[u8] = &[", name).unwrap();
            for (i, &alias) in aliases.iter().enumerate() {
                if i > 0 { write!(f, ", ").unwrap(); }
                write!(f, "0x{:02x}", alias).unwrap();
            }
            writeln!(f, "];").unwrap();
        }
    }
    writeln!(f, "}}").unwrap();
    writeln!(f).unwrap();

    // Also write individual opcode constants for the runtime
    writeln!(f, "/// Shuffled opcode values for this build").unwrap();
    writeln!(f, "pub mod opcodes {{").unwrap();

    let mut current_mod = "";
    for (module, name, base_val) in BASE_OPCODES.iter() {
        if *module != current_mod {
            if !current_mod.is_empty() {
                writeln!(f, "    }}").unwrap();
            }
            writeln!(f, "    pub mod {} {{", module).unwrap();
            current_mod = module;
        }
        let shuffled = table.encode[*base_val as usize];
        writeln!(f, "        pub const {}: u8 = 0x{:02x};", name, shuffled).unwrap();
    }
    if !current_mod.is_empty() {
        writeln!(f, "    }}").unwrap();
    }
    writeln!(f, "}}").unwrap();
    writeln!(f).unwrap();
}

/// HMAC-SHA256 for production key derivation
fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64;

    // Prepare key
    let mut k = [0u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let hashed = sha256(key);
        k[..32].copy_from_slice(&hashed);
    } else {
        k[..key.len()].copy_from_slice(key);
    }

    // Inner and outer pads
    let mut ipad = [0x36u8; BLOCK_SIZE];
    let mut opad = [0x5cu8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }

    // Inner hash: H(ipad || data)
    let mut inner_data = ipad.to_vec();
    inner_data.extend_from_slice(data);
    let inner_hash = sha256(&inner_data);

    // Outer hash: H(opad || inner_hash)
    let mut outer_data = opad.to_vec();
    outer_data.extend_from_slice(&inner_hash);
    sha256(&outer_data)
}

// ============================================================================
// MAGIC BYTES - Randomized bytecode header magic
// ============================================================================

/// Generate random MAGIC bytes for bytecode header identification
fn generate_magic_bytes(seed: &[u8; 32]) -> [u8; 4] {
    let hash = hmac_sha256(seed, b"magic-bytes-v1");
    [hash[0], hash[1], hash[2], hash[3]]
}

fn write_magic_bytes(f: &mut File, magic: &[u8; 4]) {
    writeln!(f, "/// Randomized MAGIC bytes for bytecode header").unwrap();
    writeln!(f, "pub const MAGIC: [u8; 4] = [0x{:02x}, 0x{:02x}, 0x{:02x}, 0x{:02x}];",
             magic[0], magic[1], magic[2], magic[3]).unwrap();
    writeln!(f).unwrap();
}

// ============================================================================
// NATIVE FUNCTION IDs - Shuffled native call identifiers
// ============================================================================

/// Native function ID mapping
struct NativeIdMap {
    check_root: u8,
    check_emulator: u8,
    check_hooks: u8,
    check_debugger: u8,
    check_tamper: u8,
    get_timestamp: u8,
    hash_fnv1a: u8,
    read_memory: u8,
    get_device_hash: u8,
    custom_start: u8,
}

fn generate_native_ids(seed: &[u8; 32]) -> NativeIdMap {
    let hash = hmac_sha256(seed, b"native-ids-v1");

    // Use first 9 bytes of hash as shuffled IDs (0-8 range shuffled)
    let mut ids: Vec<u8> = (0..9).collect();

    // Fisher-Yates shuffle using hash bytes
    for i in (1..9).rev() {
        let j = (hash[i] as usize) % (i + 1);
        ids.swap(i, j);
    }

    NativeIdMap {
        check_root: ids[0],
        check_emulator: ids[1],
        check_hooks: ids[2],
        check_debugger: ids[3],
        check_tamper: ids[4],
        get_timestamp: ids[5],
        hash_fnv1a: ids[6],
        read_memory: ids[7],
        get_device_hash: ids[8],
        custom_start: 128, // Keep custom start fixed at 128
    }
}

fn write_native_ids(f: &mut File, ids: &NativeIdMap) {
    writeln!(f, "/// Shuffled native function IDs").unwrap();
    writeln!(f, "pub mod native_ids {{").unwrap();
    writeln!(f, "    pub const CHECK_ROOT: u8 = {};", ids.check_root).unwrap();
    writeln!(f, "    pub const CHECK_EMULATOR: u8 = {};", ids.check_emulator).unwrap();
    writeln!(f, "    pub const CHECK_HOOKS: u8 = {};", ids.check_hooks).unwrap();
    writeln!(f, "    pub const CHECK_DEBUGGER: u8 = {};", ids.check_debugger).unwrap();
    writeln!(f, "    pub const CHECK_TAMPER: u8 = {};", ids.check_tamper).unwrap();
    writeln!(f, "    pub const GET_TIMESTAMP: u8 = {};", ids.get_timestamp).unwrap();
    writeln!(f, "    pub const HASH_FNV1A: u8 = {};", ids.hash_fnv1a).unwrap();
    writeln!(f, "    pub const READ_MEMORY: u8 = {};", ids.read_memory).unwrap();
    writeln!(f, "    pub const GET_DEVICE_HASH: u8 = {};", ids.get_device_hash).unwrap();
    writeln!(f, "    pub const CUSTOM_START: u8 = {};", ids.custom_start).unwrap();
    writeln!(f, "}}").unwrap();
    writeln!(f).unwrap();
}

// ============================================================================
// REGISTER MAPPING - Shuffled register indices
// ============================================================================

/// Register mapping (logical R0-R7 to physical indices)
struct RegisterMap {
    map: [u8; 8],      // logical -> physical
    reverse: [u8; 8],  // physical -> logical
}

fn generate_register_map(seed: &[u8; 32]) -> RegisterMap {
    let hash = hmac_sha256(seed, b"register-map-v1");

    // Shuffle 0-7
    let mut map: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

    for i in (1..8).rev() {
        let j = (hash[i] as usize) % (i + 1);
        map.swap(i, j);
    }

    // Build reverse mapping
    let mut reverse = [0u8; 8];
    for (logical, &physical) in map.iter().enumerate() {
        reverse[physical as usize] = logical as u8;
    }

    RegisterMap { map, reverse }
}

fn write_register_map(f: &mut File, reg_map: &RegisterMap) {
    writeln!(f, "/// Shuffled register mapping (logical -> physical)").unwrap();
    write!(f, "pub const REGISTER_MAP: [u8; 8] = [").unwrap();
    for (i, &val) in reg_map.map.iter().enumerate() {
        if i > 0 { write!(f, ", ").unwrap(); }
        write!(f, "{}", val).unwrap();
    }
    writeln!(f, "];").unwrap();

    writeln!(f, "/// Reverse register mapping (physical -> logical)").unwrap();
    write!(f, "pub const REGISTER_REVERSE: [u8; 8] = [").unwrap();
    for (i, &val) in reg_map.reverse.iter().enumerate() {
        if i > 0 { write!(f, ", ").unwrap(); }
        write!(f, "{}", val).unwrap();
    }
    writeln!(f, "];").unwrap();
    writeln!(f).unwrap();
}

// ============================================================================
// FNV HASH CONSTANTS - Custom hash function parameters
// ============================================================================

/// FNV hash constants
struct FnvConstants {
    basis_64: u64,
    prime_64: u64,
    basis_32: u32,
    prime_32: u32,
}

fn generate_fnv_constants(seed: &[u8; 32]) -> FnvConstants {
    let hash = hmac_sha256(seed, b"fnv-constants-v1");

    // Generate random basis values (must be non-zero, odd for better distribution)
    let basis_64 = u64::from_le_bytes([
        hash[0], hash[1], hash[2], hash[3],
        hash[4], hash[5], hash[6], hash[7],
    ]) | 1; // Ensure odd

    let basis_32 = u32::from_le_bytes([
        hash[8], hash[9], hash[10], hash[11],
    ]) | 1; // Ensure odd

    // Generate primes - use well-known FNV-like primes with slight modification
    // Original FNV-1a prime for 64-bit: 0x100000001b3
    // Original FNV-1a prime for 32-bit: 0x01000193
    // We'll XOR with some seed bytes to vary them slightly while keeping good properties
    let prime_modifier = u64::from_le_bytes([
        hash[16], hash[17], hash[18], hash[19],
        hash[20], hash[21], hash[22], hash[23],
    ]);

    // Keep lower bits of original prime, modify upper bits
    let prime_64 = 0x100000001b3 ^ ((prime_modifier & 0xFFFF_0000_0000_0000) >> 8);
    let prime_32 = 0x01000193 ^ ((hash[24] as u32) << 24);

    FnvConstants {
        basis_64,
        prime_64,
        basis_32,
        prime_32,
    }
}

fn write_fnv_constants(f: &mut File, fnv: &FnvConstants) {
    writeln!(f, "/// Randomized FNV-1a hash constants").unwrap();
    writeln!(f, "pub const FNV_BASIS_64: u64 = 0x{:016x};", fnv.basis_64).unwrap();
    writeln!(f, "pub const FNV_PRIME_64: u64 = 0x{:016x};", fnv.prime_64).unwrap();
    writeln!(f, "pub const FNV_BASIS_32: u32 = 0x{:08x};", fnv.basis_32).unwrap();
    writeln!(f, "pub const FNV_PRIME_32: u32 = 0x{:08x};", fnv.prime_32).unwrap();
    writeln!(f).unwrap();
}

// ============================================================================
// XOR KEY - Randomized obfuscation key
// ============================================================================

fn generate_xor_key(seed: &[u8; 32]) -> u8 {
    let hash = hmac_sha256(seed, b"xor-key-v1");
    // Ensure non-zero XOR key
    if hash[0] == 0 { hash[1] | 1 } else { hash[0] }
}

fn write_xor_key(f: &mut File, key: u8) {
    writeln!(f, "/// Randomized XOR key for string obfuscation").unwrap();
    writeln!(f, "pub const XOR_KEY: u8 = 0x{:02x};", key).unwrap();
    writeln!(f).unwrap();
}

// ============================================================================
// FLAG BITS - Shuffled CPU flag bit positions
// ============================================================================

/// Flag bit positions
struct FlagBits {
    zero: u8,
    carry: u8,
    overflow: u8,
    sign: u8,
}

fn generate_flag_bits(seed: &[u8; 32]) -> FlagBits {
    let hash = hmac_sha256(seed, b"flag-bits-v1");

    // Shuffle bit positions 0-3
    let mut positions: [u8; 4] = [0, 1, 2, 3];
    for i in (1..4).rev() {
        let j = (hash[i] as usize) % (i + 1);
        positions.swap(i, j);
    }

    FlagBits {
        zero: 1 << positions[0],
        carry: 1 << positions[1],
        overflow: 1 << positions[2],
        sign: 1 << positions[3],
    }
}

fn write_flag_bits(f: &mut File, flags: &FlagBits) {
    writeln!(f, "/// Shuffled flag bit positions").unwrap();
    writeln!(f, "pub mod flags {{").unwrap();
    writeln!(f, "    pub const ZERO: u8 = 0b{:08b};", flags.zero).unwrap();
    writeln!(f, "    pub const CARRY: u8 = 0b{:08b};", flags.carry).unwrap();
    writeln!(f, "    pub const OVERFLOW: u8 = 0b{:08b};", flags.overflow).unwrap();
    writeln!(f, "    pub const SIGN: u8 = 0b{:08b};", flags.sign).unwrap();
    writeln!(f, "}}").unwrap();
    writeln!(f).unwrap();
}
