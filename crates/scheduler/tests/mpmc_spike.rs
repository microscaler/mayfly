//! MPMC spike (D3.2): validates lock-free pattern for multi-consumer syscall processing.
//!
//! - Cloned crossbeam Receiver: two threads compete to recv(); each message delivered to exactly one.
//! - SegQueue: lock-free handoff from recv threads to a single consumer (no mutex around queue).
//!
//! Does not integrate with Scheduler; see docs/design-mayfly-channels-mpmc.md for integration notes.

use crossbeam::channel::unbounded;
use crossbeam_queue::SegQueue;
use scheduler::TaskId;
use scheduler::syscall::SystemCall;
use std::sync::Arc;
use std::thread;

#[test]
fn mpmc_cloned_receiver_and_segqueue_handoff() {
    const N: u64 = 1000;
    let (tx, rx) = unbounded::<(TaskId, SystemCall)>();
    let queue: Arc<SegQueue<(TaskId, SystemCall)>> = Arc::new(SegQueue::new());

    let rx1 = rx.clone();
    let q1 = Arc::clone(&queue);
    let t1 = thread::spawn(move || {
        while let Ok(msg) = rx1.recv() {
            q1.push(msg);
        }
    });

    let q2 = Arc::clone(&queue);
    let t2 = thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            q2.push(msg);
        }
    });

    for tid in 1..=N {
        tx.send((tid, SystemCall::Yield)).unwrap();
    }
    drop(tx);

    t1.join().unwrap();
    t2.join().unwrap();

    let mut count = 0u64;
    while let Some((tid, _)) = queue.pop() {
        count += 1;
        assert!((1..=N).contains(&tid));
    }
    assert_eq!(count, N);
}
