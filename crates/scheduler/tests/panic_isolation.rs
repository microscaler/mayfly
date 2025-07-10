use scheduler::{Scheduler, SystemCall, task::TaskContext};
use serial_test::file_serial;
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
#[file_serial]
fn panic_isolation() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let (child, parent, order) = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };

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

        barrier.wait();
        let order = handle.join().unwrap();
        (child, parent, order)
    });

    assert!(order.contains(&parent));
    use scheduler::task::{TaskCompletionReason, TaskState};
    assert_eq!(
        sched.task_state(child),
        Some(TaskState::Finished(TaskCompletionReason::Failed))
    );
}
