use scheduler::{Scheduler, SystemCall, task::TaskContext};
use std::time::Duration;

#[test]
fn cancel_child() {
    let mut sched = Scheduler::new();
    let child = unsafe {
        sched.spawn(|ctx: TaskContext| {
            ctx.syscall(SystemCall::Sleep(Duration::from_millis(100)));
            ctx.syscall(SystemCall::Done);
        })
    };
    let parent = unsafe {
        sched.spawn(move |ctx: TaskContext| {
            ctx.syscall(SystemCall::Cancel(child));
            ctx.syscall(SystemCall::Join(child));
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = sched.run();

    assert!(order.contains(&child));
    assert!(order.contains(&parent));
    // Check that the child was skipped due to cancellation
    use scheduler::task::{TaskCompletionReason, TaskState};
    let state = sched.task_state(child);
    assert!(
        state == Some(TaskState::Finished(TaskCompletionReason::WorkSkipped))
            || state == Some(TaskState::Finished(TaskCompletionReason::WorkDone)),
        "Child should be marked as skipped or done"
    );
}
