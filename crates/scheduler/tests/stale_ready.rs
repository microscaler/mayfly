use scheduler::{Scheduler, SystemCall, task::TaskContext};
use serial_test::file_serial;
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
#[file_serial]
fn stale_ready_id_is_ignored() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        let child = unsafe {
            sched.spawn(|ctx: TaskContext| {
                ctx.syscall(SystemCall::Done);
            })
        };
        sched.ready_push_duplicate_for_test(child);
        barrier.wait();
        let order = handle.join().unwrap();
        assert_eq!(order, vec![child]);
    });
}
