#![cfg(not(feature = "async-io"))]

use scheduler::{Scheduler, SystemCall, task::TaskContext};
use std::time::Duration;

#[cfg(not(feature = "async-io"))]
#[test]
fn test_io_wait_wakes_task() {
    let mut sched = Scheduler::new();
    let io_tx = sched.io_handle();
    unsafe {
        sched.spawn(|ctx: TaskContext| {
            ctx.syscall(SystemCall::IoWait(1));
            ctx.syscall(SystemCall::Done);
        });
    }
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        io_tx.send(1).unwrap();
    });
    let order = sched.run();
    assert_eq!(order, vec![1]);
}
