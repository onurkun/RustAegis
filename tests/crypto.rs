//! Crypto tests for AES-256-GCM encryption and HMAC

use aegis_vm::crypto::{
    derive_key, derive_nonce, derive_build_id,
    compute_hmac, verify_hmac, CryptoContext,
};

#[test]
fn test_key_derivation() {
    let seed = [0x42u8; 32];
    let key1 = derive_key(&seed, b"context1");
    let key2 = derive_key(&seed, b"context2");

    assert_ne!(key1, key2);
    assert_ne!(key1, [0u8; 32]);
}

#[test]
fn test_nonce_derivation() {
    let seed = [0x42u8; 32];
    let nonce1 = derive_nonce(&seed, 0);
    let nonce2 = derive_nonce(&seed, 1);

    assert_ne!(nonce1, nonce2);
    assert_ne!(nonce1, [0u8; 12]);
}

#[test]
fn test_encrypt_decrypt() {
    let seed = [0x42u8; 32];
    let mut ctx = CryptoContext::new(seed);

    let plaintext = b"Hello, VM bytecode!";
    let (ciphertext, nonce, tag) = ctx.encrypt(plaintext).unwrap();

    let decrypted = ctx.decrypt(&ciphertext, &nonce, &tag).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_tampered_ciphertext_fails() {
    let seed = [0x42u8; 32];
    let mut ctx = CryptoContext::new(seed);

    let plaintext = b"Secret bytecode";
    let (mut ciphertext, nonce, tag) = ctx.encrypt(plaintext).unwrap();

    // Tamper with ciphertext
    if !ciphertext.is_empty() {
        ciphertext[0] ^= 0xFF;
    }

    let result = ctx.decrypt(&ciphertext, &nonce, &tag);
    assert!(result.is_err());
}

#[test]
fn test_hmac_verification() {
    let key = [0x42u8; 32];
    let data = b"some data to verify";

    let hmac = compute_hmac(&key, data);
    assert!(verify_hmac(&key, data, &hmac));

    // Tampered data should fail
    let tampered = b"tampered data here";
    assert!(!verify_hmac(&key, tampered, &hmac));
}

#[test]
fn test_build_id_derivation() {
    let seed1 = [0x42u8; 32];
    let seed2 = [0x43u8; 32];

    let id1 = derive_build_id(&seed1);
    let id2 = derive_build_id(&seed2);

    assert_ne!(id1, id2);
    assert_ne!(id1, 0);
}

#[test]
fn test_encrypt_empty() {
    let seed = [0x42u8; 32];
    let mut ctx = CryptoContext::new(seed);

    let plaintext = b"";
    let (ciphertext, nonce, tag) = ctx.encrypt(plaintext).unwrap();

    let decrypted = ctx.decrypt(&ciphertext, &nonce, &tag).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_encrypt_large() {
    let seed = [0x42u8; 32];
    let mut ctx = CryptoContext::new(seed);

    // 1KB of data
    let plaintext: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let (ciphertext, nonce, tag) = ctx.encrypt(&plaintext).unwrap();

    let decrypted = ctx.decrypt(&ciphertext, &nonce, &tag).unwrap();
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_nonce_uniqueness() {
    let seed = [0x42u8; 32];
    let mut ctx = CryptoContext::new(seed);

    // Each encryption should use a unique nonce
    let (_, nonce1, _) = ctx.encrypt(b"test1").unwrap();
    let (_, nonce2, _) = ctx.encrypt(b"test2").unwrap();
    let (_, nonce3, _) = ctx.encrypt(b"test3").unwrap();

    assert_ne!(nonce1, nonce2);
    assert_ne!(nonce2, nonce3);
    assert_ne!(nonce1, nonce3);
}

#[test]
fn test_different_seeds_different_encryption() {
    let seed1 = [0x42u8; 32];
    let seed2 = [0x43u8; 32];

    let mut ctx1 = CryptoContext::new(seed1);
    let mut ctx2 = CryptoContext::new(seed2);

    let plaintext = b"same plaintext";
    let (ciphertext1, _, _) = ctx1.encrypt(plaintext).unwrap();
    let (ciphertext2, _, _) = ctx2.encrypt(plaintext).unwrap();

    // Same plaintext encrypted with different keys should produce different ciphertext
    assert_ne!(ciphertext1, ciphertext2);
}
