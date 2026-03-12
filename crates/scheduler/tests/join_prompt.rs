use scheduler::{Scheduler, SystemCall, task::TaskContext};

#[test]
fn join_wake_before_next_ready() {
    let mut sched = Scheduler::new();
    let child = unsafe {
        sched.spawn(|ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    unsafe {
        sched.spawn(move |ctx: TaskContext| {
            ctx.syscall(SystemCall::Join(child));
            ctx.syscall(SystemCall::Done);
        });
    }
    unsafe {
        sched.spawn(|ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        });
    }
    let order = sched.run();
    assert_eq!(order.len(), 3);
}
