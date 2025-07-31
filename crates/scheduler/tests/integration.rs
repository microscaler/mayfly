use scheduler::{Scheduler, SystemCall, task::TaskContext};
use serial_test::file_serial;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

#[test]
#[file_serial]
fn integration_task_order() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };

        unsafe {
            sched.spawn(|ctx: TaskContext| {
                std::thread::sleep(Duration::from_millis(50));
                ctx.syscall(SystemCall::Done);
            });
        }

        unsafe {
            sched.spawn(|ctx: TaskContext| {
                std::thread::sleep(Duration::from_millis(10));
                ctx.syscall(SystemCall::Done);
            });
        }

        barrier.wait();
        let order = handle.join().unwrap();
        assert_eq!(order, vec![2, 1]);
        order
    });
    assert_eq!(order, vec![2, 1]);
}

#[cfg(not(feature = "async-io"))]
#[test]
#[file_serial]
fn integration_join_and_io_wait() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let (child, order) = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        let io_tx = sched.io_handle();

        let child = unsafe {
            sched.spawn(|ctx: TaskContext| {
                ctx.syscall(SystemCall::Done);
            })
        };

        unsafe {
            sched.spawn(move |ctx: TaskContext| {
                ctx.syscall(SystemCall::Join(child));
                ctx.syscall(SystemCall::Done);
            });
        }

        unsafe {
            sched.spawn(|ctx: TaskContext| {
                ctx.syscall(SystemCall::IoWait(1));
                ctx.syscall(SystemCall::Done);
            });
        }

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            io_tx.send(1).unwrap();
        });

        barrier.wait();
        let order = handle.join().unwrap();
        assert_eq!(order.len(), 3);
        assert!(order.contains(&child));
        assert!(order.contains(&2));
        assert!(order.contains(&3));
        (child, order)
    });
    assert_eq!(order.len(), 3);
    assert!(order.contains(&child));
    assert!(order.contains(&2));
    assert!(order.contains(&3));
}
