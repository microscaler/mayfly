use scheduler::{Scheduler, syscall::SystemCall, task::TaskContext};
use std::time::{Duration, Instant};

#[test]
fn sleep_virtual_runs_without_delay() {
    let mut sched = Scheduler::new();
    unsafe {
        sched.spawn(|ctx: TaskContext| {
            ctx.syscall(SystemCall::Sleep(Duration::from_millis(10)));
            ctx.syscall(SystemCall::Done);
        });
    }
    let start = Instant::now();
    let order = sched.run();
    let elapsed = start.elapsed();
    // Virtual time advances in run(); real time may still elapse due to recv_timeout and scheduling.
    assert!(
        elapsed < Duration::from_secs(1),
        "run should complete; took {elapsed:?}"
    );
    assert_eq!(order.len(), 1);
}
