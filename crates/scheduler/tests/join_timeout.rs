use scheduler::{Scheduler, SystemCall, task::TaskContext};
use std::time::{Duration, Instant};

#[test]
fn join_timeout_wakes() {
    let mut sched = Scheduler::new();
    // Child finishes quickly; parent waits with 10ms timeout (can wake from timeout or from child done).
    let child = unsafe {
        sched.spawn(|ctx: TaskContext| {
            ctx.syscall(SystemCall::Sleep(Duration::from_millis(5)));
            ctx.syscall(SystemCall::Done);
        })
    };
    let parent = unsafe {
        sched.spawn(move |ctx: TaskContext| {
            ctx.syscall(SystemCall::JoinTimeout {
                target: child,
                dur: Duration::from_millis(10),
            });
            ctx.syscall(SystemCall::Done);
        })
    };
    let start = Instant::now();
    let order = sched.run();
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_millis(50), "elapsed {elapsed:?}");
    assert!(order.contains(&child));
    assert!(order.contains(&parent));
}
