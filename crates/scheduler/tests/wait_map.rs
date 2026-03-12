//! Tests for WaitMap: join waiters, I/O waiters, and remove_waiter (e.g. timeout).

use scheduler::WaitMap;
use scheduler::task::{TaskCompletionReason, TaskState};

#[test]
fn wait_for_and_complete_returns_waiters_and_state() {
    let mut w = WaitMap::new();
    w.wait_for(10, 1);
    w.wait_for(10, 2);
    let (waiters, state) = w.complete(10, TaskState::Finished(TaskCompletionReason::WorkDone));
    assert_eq!(waiters, vec![1, 2]);
    assert_eq!(state, TaskState::Finished(TaskCompletionReason::WorkDone));
}

#[test]
fn complete_with_no_waiters_returns_empty() {
    let mut w = WaitMap::new();
    let (waiters, _) = w.complete(99, TaskState::Ready);
    assert!(waiters.is_empty());
}

#[test]
fn wait_io_and_complete_io_returns_waiters() {
    let mut w = WaitMap::new();
    w.wait_io(1, 10);
    w.wait_io(1, 11);
    let tids = w.complete_io(1);
    assert_eq!(tids, vec![10, 11]);
}

#[test]
fn complete_io_with_no_waiters_returns_empty() {
    let mut w = WaitMap::new();
    let tids = w.complete_io(42);
    assert!(tids.is_empty());
}

#[test]
fn remove_waiter_removes_one_returns_true() {
    let mut w = WaitMap::new();
    w.wait_for(10, 1);
    w.wait_for(10, 2);
    assert!(w.remove_waiter(10, 1));
    let (waiters, _) = w.complete(10, TaskState::Ready);
    assert_eq!(waiters, vec![2]);
}

#[test]
fn remove_waiter_last_removes_target_key() {
    let mut w = WaitMap::new();
    w.wait_for(10, 1);
    assert!(w.remove_waiter(10, 1));
    let (waiters, _) = w.complete(10, TaskState::Ready);
    assert!(waiters.is_empty());
}

#[test]
fn remove_waiter_nonexistent_target_returns_false() {
    let mut w = WaitMap::new();
    w.wait_for(10, 1);
    assert!(!w.remove_waiter(99, 1));
    let (waiters, _) = w.complete(10, TaskState::Ready);
    assert_eq!(waiters, vec![1]);
}

#[test]
fn remove_waiter_wrong_waiter_returns_false() {
    let mut w = WaitMap::new();
    w.wait_for(10, 1);
    assert!(!w.remove_waiter(10, 2));
    let (waiters, _) = w.complete(10, TaskState::Ready);
    assert_eq!(waiters, vec![1]);
}
