use scheduler::{Scheduler, SystemCall, task::TaskContext};

#[test]
fn panic_isolation() {
    let mut sched = Scheduler::new();
    let child = unsafe {
        sched.spawn(|ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
            panic!("boom");
        })
    };
    let parent = unsafe {
        sched.spawn(move |ctx: TaskContext| {
            ctx.syscall(SystemCall::Join(child));
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = sched.run();

    assert!(order.contains(&parent));
    use scheduler::task::{TaskCompletionReason, TaskState};
    assert_eq!(
        sched.task_state(child),
        Some(TaskState::Finished(TaskCompletionReason::Failed))
    );
}
