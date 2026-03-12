use crossbeam::channel::unbounded;
use scheduler::{Scheduler, SystemCall, task::TaskContext};

#[test]
fn tasks_yield_after_syscall() {
    let mut sched = Scheduler::new();
    let (tx, rx) = unbounded();
    let child = unsafe {
        let tx = tx.clone();
        sched.spawn(move |ctx: TaskContext| {
            tx.send("child done").unwrap();
            ctx.syscall(SystemCall::Done);
        })
    };
    unsafe {
        let tx = tx.clone();
        sched.spawn(move |ctx: TaskContext| {
            tx.send("parent before join").unwrap();
            ctx.syscall(SystemCall::Join(child));
            tx.send("parent after join").unwrap();
            ctx.syscall(SystemCall::Done);
        });
    }
    let _order = sched.run();
    let events: Vec<&str> = rx.try_iter().collect();
    assert_eq!(events.len(), 3);
}
