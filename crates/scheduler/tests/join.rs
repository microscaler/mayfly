use scheduler::{Scheduler, SystemCall, task::TaskContext};
use serial_test::file_serial;

#[test]
#[file_serial]
fn test_join_wakes_waiter() {
    let mut sched = Scheduler::new();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let order = std::thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        let child = unsafe {
            sched.spawn(|ctx: TaskContext| {
                ctx.syscall(SystemCall::Done);
            })
        };

        let _parent = unsafe {
            sched.spawn(move |ctx: TaskContext| {
                ctx.syscall(SystemCall::Join(child));
                ctx.syscall(SystemCall::Done);
            })
        };

        barrier.wait();
        handle.join().unwrap()
    });
    assert_eq!(order.len(), 2);
}
