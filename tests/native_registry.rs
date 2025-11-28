//! Tests for native function registry

use aegis_vm::native::{NativeRegistry, NativeRegistryBuilder, standard_ids};
use aegis_vm::error::VmError;

#[test]
fn test_register_and_call() {
    let mut registry = NativeRegistry::new();
    registry.register(0, |_| 42).unwrap();

    assert_eq!(registry.call(0, &[]).unwrap(), 42);
}

#[test]
fn test_call_with_args() {
    let mut registry = NativeRegistry::new();
    registry.register(0, |args| {
        args.iter().sum()
    }).unwrap();

    assert_eq!(registry.call(0, &[1, 2, 3, 4, 5]).unwrap(), 15);
}

#[test]
fn test_not_found() {
    let registry = NativeRegistry::new();
    assert!(matches!(
        registry.call(99, &[]),
        Err(VmError::NativeFunctionNotFound)
    ));
}

#[test]
fn test_already_registered() {
    let mut registry = NativeRegistry::new();
    registry.register(0, |_| 1).unwrap();
    assert!(matches!(
        registry.register(0, |_| 2),
        Err(VmError::NativeFunctionAlreadyRegistered)
    ));
}

#[test]
fn test_replace() {
    let mut registry = NativeRegistry::new();
    registry.register(0, |_| 1).unwrap();
    registry.register_replace(0, |_| 2);
    assert_eq!(registry.call(0, &[]).unwrap(), 2);
}

#[test]
fn test_builder() {
    let registry = NativeRegistryBuilder::new()
        .with_function(0, |_| 100)
        .with_function(1, |args| args.get(0).copied().unwrap_or(0) * 2)
        .with_hash()
        .build();

    assert_eq!(registry.call(0, &[]).unwrap(), 100);
    assert_eq!(registry.call(1, &[21]).unwrap(), 42);
    assert!(registry.is_registered(standard_ids::HASH_FNV1A));
}

#[test]
fn test_count_and_clear() {
    let mut registry = NativeRegistry::new();
    assert_eq!(registry.count(), 0);

    registry.register(0, |_| 0).unwrap();
    registry.register(5, |_| 0).unwrap();
    assert_eq!(registry.count(), 2);

    registry.clear();
    assert_eq!(registry.count(), 0);
}
