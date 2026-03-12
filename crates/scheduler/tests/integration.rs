use scheduler::{Scheduler, SystemCall, task::TaskContext};
use std::time::Duration;

#[test]
fn integration_task_order() {
    let mut sched = Scheduler::new();
    unsafe {
        sched.spawn(|ctx: TaskContext| {
            ctx.syscall(SystemCall::Sleep(Duration::from_millis(50)));
            ctx.syscall(SystemCall::Done);
        });
    }
    unsafe {
        sched.spawn(|ctx: TaskContext| {
            ctx.syscall(SystemCall::Sleep(Duration::from_millis(10)));
            ctx.syscall(SystemCall::Done);
        });
    }
    let order = sched.run();
    assert_eq!(order.len(), 2);
    assert!(order.contains(&1));
    assert!(order.contains(&2));
}

#[cfg(not(feature = "async-io"))]
#[test]
fn integration_join_and_io_wait() {
    let mut sched = Scheduler::new();
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
    let order = sched.run();
    assert_eq!(order.len(), 3);
    assert!(order.contains(&child));
    assert!(order.contains(&2));
    assert!(order.contains(&3));
}
