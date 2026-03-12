use scheduler::SystemCall;

#[test]
fn test_may_scheduler() {
    let mut sched = scheduler::Scheduler::new();
    unsafe {
        sched.spawn(|ctx| {
            println!("hello from may coroutine!");
            ctx.syscall(SystemCall::Done);
        });
    }
    let _order = sched.run();
}
