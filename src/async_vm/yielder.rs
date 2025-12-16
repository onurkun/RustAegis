//! Yield Future
//!
//! A simple future that yields once before completing.
//! Used to create state machine transition points in the async VM.

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

/// A future that yields control once before completing
///
/// This is used to inject yield points into the async VM loop,
/// causing Rust to generate state machine transitions that
/// complicate control flow analysis.
///
/// # How it works
/// 1. First poll: Returns `Pending` (yields control)
/// 2. Second poll: Returns `Ready(())` (completes)
///
/// # Example
/// ```ignore
/// async fn vm_loop() {
///     loop {
///         execute_instruction();
///         if should_yield() {
///             YieldNow::new().await; // State machine transition
///         }
///     }
/// }
/// ```
#[derive(Debug)]
pub struct YieldNow {
    yielded: bool,
}

impl YieldNow {
    /// Create a new yield future
    #[inline]
    pub const fn new() -> Self {
        Self { yielded: false }
    }
}

impl Default for YieldNow {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Future for YieldNow {
    type Output = ();

    #[inline]
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            // Second poll: complete
            Poll::Ready(())
        } else {
            // First poll: yield control
            self.yielded = true;
            // Note: We don't call cx.waker().wake() because our
            // block_on executor will poll again regardless
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::async_vm::block_on;

    #[test]
    fn test_yield_now() {
        let mut call_count = 0;

        block_on(async {
            call_count += 1;
            YieldNow::new().await;
            call_count += 1;
        });

        assert_eq!(call_count, 2);
    }

    #[test]
    fn test_multiple_yields() {
        let mut yields = 0;

        block_on(async {
            for _ in 0..10 {
                YieldNow::new().await;
                yields += 1;
            }
        });

        assert_eq!(yields, 10);
    }
}
