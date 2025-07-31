use scheduler::task::TaskContext;
use scheduler::{Scheduler, SystemCall};
use serial_test::file_serial;
use std::time::Duration;

#[test]
#[file_serial]
fn test_done_order() {
    let mut sched = Scheduler::new();

    // Spawn all tasks before running the scheduler
    let t1 = unsafe {
        sched.spawn(|ctx: TaskContext| {
            std::thread::sleep(Duration::from_millis(50));
            ctx.syscall(SystemCall::Done);
        })
    };
    let t2 = unsafe {
        sched.spawn(|ctx: TaskContext| {
            std::thread::sleep(Duration::from_millis(10));
            ctx.syscall(SystemCall::Done);
        })
    };

    let order = sched.run();
    assert_eq!(order, vec![t2, t1]);
    assert_eq!(sched.ready_len(), 0);
}
