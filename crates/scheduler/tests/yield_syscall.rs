use scheduler::{Scheduler, SystemCall, task::TaskContext};
use serial_test::file_serial;
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
#[file_serial]
fn syscall_yield_order() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        unsafe {
            sched.spawn(|ctx: TaskContext| {
                ctx.syscall(SystemCall::Done);
            });
        }
        unsafe {
            sched.spawn(|ctx: TaskContext| {
                ctx.syscall(SystemCall::Done);
            });
        }
        barrier.wait();
        handle.join().unwrap()
    });
    assert_eq!(order, vec![1, 2]);
}
