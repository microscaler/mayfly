use scheduler::{Scheduler, SystemCall, task::TaskContext};
use serial_test::file_serial;

#[test]
#[file_serial]
fn yield_order() {
    let mut sched = Scheduler::new();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let task_barrier = std::sync::Arc::new(std::sync::Barrier::new(3));
    let order = std::thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        let a = unsafe {
            let tb = task_barrier.clone();
            sched.spawn(move |ctx: TaskContext| {
                tb.wait();
                ctx.yield_now();
                ctx.syscall(SystemCall::Done);
            })
        };
        let b = unsafe {
            let tb = task_barrier.clone();
            sched.spawn(move |ctx: TaskContext| {
                tb.wait();
                ctx.syscall(SystemCall::Done);
            })
        };
        barrier.wait();
        task_barrier.wait();
        let order = handle.join().unwrap();
        assert_eq!(order, vec![b, a]);
        order
    });
    assert_eq!(order.len(), 2);
}
