use scheduler::{Scheduler, SystemCall, task::TaskContext};
use serial_test::file_serial;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

#[test]
#[file_serial]
fn join_timeout_wakes() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let start = Instant::now();
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
                ctx.syscall(SystemCall::JoinTimeout {
                    target: child,
                    dur: Duration::from_millis(10),
                });
                ctx.syscall(SystemCall::Cancel(child));
                ctx.syscall(SystemCall::Done);
            })
        };

        barrier.wait();
        let order = handle.join().unwrap();
        (child, parent, order)
    });
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_millis(50), "elapsed {elapsed:?}");
    assert_eq!(order.first().copied(), Some(child));
    assert!(order.contains(&parent));
}
