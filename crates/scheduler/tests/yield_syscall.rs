use scheduler::{Scheduler, SystemCall, task::TaskContext};

#[test]
fn syscall_yield_order() {
    let mut sched = Scheduler::new();
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
    let order = sched.run();
    assert_eq!(order.len(), 2);
    assert!(order.contains(&1));
    assert!(order.contains(&2));
}
