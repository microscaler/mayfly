use scheduler::{Scheduler, SystemCall, task::TaskContext};

#[test]
fn stale_ready_id_is_ignored() {
    let mut sched = Scheduler::new();
    let child = unsafe {
        sched.spawn(|ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    sched.ready_push_duplicate_for_test(child);
    let order = sched.run();
    assert_eq!(order, vec![child]);
}
