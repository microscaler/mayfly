use scheduler::{Scheduler, syscall::SystemCall, task::TaskContext};
use serial_test::file_serial;
use std::time::{Duration, Instant};

#[test]
#[file_serial]
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
    assert!(elapsed < Duration::from_millis(5), "took {elapsed:?}");
    assert_eq!(order.len(), 1);
}
