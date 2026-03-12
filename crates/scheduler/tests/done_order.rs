use scheduler::task::TaskContext;
use scheduler::{Scheduler, SystemCall};
use std::time::Duration;

#[test]
fn test_done_order() {
    let mut sched = Scheduler::new();
    let t1 = unsafe {
        sched.spawn(|ctx: TaskContext| {
            ctx.syscall(SystemCall::Sleep(Duration::from_millis(50)));
            ctx.syscall(SystemCall::Done);
        })
    };
    let t2 = unsafe {
        sched.spawn(|ctx: TaskContext| {
            ctx.syscall(SystemCall::Sleep(Duration::from_millis(10)));
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = sched.run();
    assert_eq!(order.len(), 2);
    assert!(order.contains(&t1));
    assert!(order.contains(&t2));
    assert_eq!(sched.ready_len(), 0);
}
