#![cfg(not(feature = "async-io"))]

use scheduler::{Scheduler, SystemCall, task::TaskContext};
use serial_test::file_serial;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

#[cfg(not(feature = "async-io"))]
#[test]
#[file_serial]
fn test_io_wait_wakes_task() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
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

        barrier.wait();
        let order = handle.join().unwrap();
        assert_eq!(order, vec![1]);
        order
    });
    assert_eq!(order, vec![1]);
}
