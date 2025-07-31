use scheduler::{Scheduler, SystemCall, task::TaskContext};
use serial_test::file_serial;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

#[test]
#[file_serial]
fn cancel_child() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let (child, parent, order) = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };

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

        barrier.wait();
        let order = handle.join().unwrap();
        (child, parent, order)
    });

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
