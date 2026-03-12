use crossbeam::channel::unbounded;
use scheduler::{SpawnRequest, Scheduler, SystemCall, task::TaskContext};

/// Scheduler runs on a dedicated thread and owns spawning via spawn requests; tasks run on that thread.
/// Completion order: b (no yield) then a (yield_now then Done).
#[test]
fn yield_order() {
    let (spawn_tx, spawn_rx) = unbounded();
    let (done_tx, done_rx) = unbounded();
    let task_barrier = std::sync::Arc::new(std::sync::Barrier::new(3));

    let handle = std::thread::spawn(move || {
        let mut sched = Scheduler::new();
        let order = sched.run_with_spawn_requests(spawn_rx);
        let _ = done_tx.send(order);
    });

    let (reply_a_tx, reply_a_rx) = unbounded();
    let (reply_b_tx, reply_b_rx) = unbounded();

    let tb = task_barrier.clone();
    spawn_tx
        .send(SpawnRequest::with_reply(
            move |sched| {
                unsafe {
                    sched.spawn(move |ctx: TaskContext| {
                        tb.wait();
                        ctx.yield_now();
                        ctx.syscall(SystemCall::Done);
                    })
                }
            },
            reply_a_tx,
        ))
        .unwrap();

    let tb = task_barrier.clone();
    spawn_tx
        .send(SpawnRequest::with_reply(
            move |sched| {
                unsafe {
                    sched.spawn(move |ctx: TaskContext| {
                        tb.wait();
                        ctx.syscall(SystemCall::Done);
                    })
                }
            },
            reply_b_tx,
        ))
        .unwrap();

    let a = reply_a_rx.recv().unwrap();
    let b = reply_b_rx.recv().unwrap();

    task_barrier.wait();

    let order = done_rx.recv().unwrap();
    handle.join().unwrap();

    assert_eq!(order.len(), 2);
    assert_eq!(order, vec![b, a]);
}
