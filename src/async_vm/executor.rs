//! Micro Custom Executor
//!
//! A minimal single-threaded executor for running async VM code.
//! No external dependencies, `no_std` compatible.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// No-op waker vtable functions
const RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| RAW_WAKER_INSTANCE, // clone: return same instance
    |_| {},                  // wake: no-op (single threaded)
    |_| {},                  // wake_by_ref: no-op
    |_| {},                  // drop: no-op (static)
);

/// Static raw waker instance
const RAW_WAKER_INSTANCE: RawWaker = RawWaker::new(core::ptr::null(), &RAW_WAKER_VTABLE);

/// Create a no-op waker for single-threaded execution
///
/// Since we're running in a single thread with no actual I/O,
/// the waker never needs to do anything.
#[inline]
fn noop_waker() -> Waker {
    // SAFETY: The vtable functions are valid and the waker is static
    unsafe { Waker::from_raw(RAW_WAKER_INSTANCE) }
}

/// Block on a future until completion
///
/// This is a minimal executor that polls the future in a loop.
/// It's designed for single-threaded, synchronous execution of
/// async code (for state machine obfuscation purposes).
///
/// # Battery-Friendly Design
/// - Uses `std::thread::yield_now()` when `std` feature is enabled
/// - Uses `core::hint::spin_loop()` for `no_std` (CPU hint, not busy spin)
///
/// # Safety
/// Uses `Pin::new_unchecked` which is safe because:
/// - The future is stack-pinned and never moved after creation
/// - The function owns the future until completion
#[inline]
pub fn block_on<F: Future>(mut future: F) -> F::Output {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);

    loop {
        // SAFETY: Future is stack-pinned and we never move it
        let pinned = unsafe { Pin::new_unchecked(&mut future) };

        match pinned.poll(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => {
                // Yield to OS scheduler (battery friendly)
                #[cfg(feature = "std")]
                std::thread::yield_now();

                // no_std: CPU hint that we're in a spin loop
                // This allows the CPU to optimize power consumption
                #[cfg(not(feature = "std"))]
                core::hint::spin_loop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_on_ready() {
        let result = block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_block_on_with_yield() {
        use crate::async_vm::YieldNow;

        let result = block_on(async {
            let mut sum = 0;
            for i in 0..5 {
                sum += i;
                YieldNow::new().await;
            }
            sum
        });
        assert_eq!(result, 10);
    }
}
