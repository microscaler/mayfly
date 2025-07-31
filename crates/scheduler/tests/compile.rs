use scheduler::Scheduler;
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn compile() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        assert!(sched.ready_is_empty());
        barrier.wait();
        handle.join().unwrap()
    });
    assert!(order.is_empty());
}
