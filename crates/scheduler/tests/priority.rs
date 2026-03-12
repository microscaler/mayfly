use scheduler::{Scheduler, SystemCall, task::TaskContext};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

/// run() with no tasks must exit after a few idle ticks.
#[test]
fn run_empty_exits() {
    let mut sched = Scheduler::new();
    let order = sched.run();
    assert!(order.is_empty());
}

#[test]
fn priority_order() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let _high = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let _low = unsafe {
        sched.spawn_with_priority(20, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Sleep(Duration::from_millis(1)));
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        barrier.wait();
        handle.join().unwrap()
    });
    println!("order: {order:?}");
    assert_eq!(order.first().copied(), Some(1));
    assert_eq!(order.last().copied(), Some(2));
}

#[test]
fn dag_dependencies() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let a = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let b = unsafe {
        sched.spawn_with_deps(&[a], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let c = unsafe {
        sched.spawn_with_deps(&[a], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let d = unsafe {
        sched.spawn_with_deps(&[b, c], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        barrier.wait();
        handle.join().unwrap()
    });
    println!("Task order: {order:?}");
    println!("A: {a}, B: {b}, C: {c}, D: {d}");
    // A must finish before B and C, and B and C before D
    let pos_a = order.iter().position(|&tid| tid == a).unwrap();
    let pos_b = order.iter().position(|&tid| tid == b).unwrap();
    let pos_c = order.iter().position(|&tid| tid == c).unwrap();
    let pos_d = order.iter().position(|&tid| tid == d).unwrap();
    println!("pos_a: {pos_a}, pos_b: {pos_b}, pos_c: {pos_c}, pos_d: {pos_d}");
    assert!(pos_a < pos_b, "A should finish before B");
    assert!(pos_a < pos_c, "A should finish before C");
    assert!(pos_b < pos_d, "B should finish before D");
    assert!(pos_c < pos_d, "C should finish before D");
}

/// Scheduler must run in a dedicated thread so may coroutines (t1, t2, t3) can run.
/// No #[file_serial] to avoid deadlock with scheduler thread.
#[test]
fn cooperative_cancellation_ordering() {
    use scheduler::task::TaskCompletionReason;
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    // Task 1: will be cancelled by t2 before or after running
    let t1 = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            if ctx.is_cancelled() {
                ctx.syscall(SystemCall::Done);
                return;
            }
            ctx.syscall(SystemCall::Done);
        })
    };
    // Task 2: cancels t1, then completes
    let t2 = unsafe {
        sched.spawn_with_priority(10, move |ctx: TaskContext| {
            ctx.syscall(SystemCall::Cancel(t1));
            ctx.syscall(SystemCall::Done);
        })
    };
    // Task 3: normal completion
    let t3 = unsafe {
        sched.spawn_with_priority(15, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        barrier.wait();
        handle.join().unwrap()
    });
    println!("Cancel test order: {order:?}");
    assert!(order.contains(&t1));
    assert!(order.contains(&t2));
    assert!(order.contains(&t3));
    let state_t1 = sched.task_state(t1);
    if let Some(scheduler::task::TaskState::Finished(reason)) = state_t1 {
        assert!(
            reason == TaskCompletionReason::WorkSkipped || reason == TaskCompletionReason::WorkDone,
            "t1 should be skipped or done"
        );
    }
}

#[test]
fn double_cancellation() {
    use scheduler::task::{TaskCompletionReason, TaskState};
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let t1 = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            if ctx.is_cancelled() {
                ctx.syscall(SystemCall::Done);
                return;
            }
            ctx.syscall(SystemCall::Done);
        })
    };
    sched.cancel_task(t1);
    sched.cancel_task(t1);
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        barrier.wait();
        handle.join().unwrap()
    });
    assert!(order.contains(&t1));
    let state = sched.task_state(t1);
    assert!(
        state == Some(TaskState::Finished(TaskCompletionReason::WorkSkipped))
            || state == Some(TaskState::Finished(TaskCompletionReason::WorkDone)),
        "Task should be skipped or done"
    );
}

/// Cancel before run: scheduler runs in dedicated thread; finalization_queue drains t1 so run() exits.
/// (No #[file_serial] to avoid deadlock: scheduler thread must not contend for the same lock.)
#[test]
fn cancellation_before_run() {
    use scheduler::task::{TaskCompletionReason, TaskState};
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let t1 = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    sched.cancel_task(t1);
    thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        barrier.wait();
        let order = handle.join().unwrap();
        assert!(order.contains(&t1), "order = {:?}", order);
    });
    let state = sched.task_state(t1);
    assert_eq!(
        state,
        Some(TaskState::Finished(TaskCompletionReason::WorkSkipped)),
        "cancelled-before-run should be WorkSkipped: {:?}",
        state
    );
}

/// Cancel before run with a task that would Sleep: same as cancellation_before_run.
/// (No #[file_serial] to avoid deadlock with scheduler thread.)
#[test]
fn cancellation_during_execution() {
    use scheduler::task::{TaskCompletionReason, TaskState};
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let t1 = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            if ctx.is_cancelled() {
                ctx.syscall(SystemCall::Done);
                return;
            }
            ctx.syscall(SystemCall::Sleep(Duration::from_millis(10)));
            if ctx.is_cancelled() {
                ctx.syscall(SystemCall::Done);
                return;
            }
            ctx.syscall(SystemCall::Done);
        })
    };
    sched.cancel_task(t1);
    thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        barrier.wait();
        let order = handle.join().unwrap();
        assert!(order.contains(&t1));
    });
    let state = sched.task_state(t1);
    assert!(
        state == Some(TaskState::Finished(TaskCompletionReason::WorkSkipped))
            || state == Some(TaskState::Finished(TaskCompletionReason::WorkDone)),
        "Task should be skipped or done: {:?}",
        state
    );
}

#[test]
fn dependency_chain_with_cancellation() {
    use scheduler::task::{TaskCompletionReason, TaskState};
    let mut sched = Scheduler::new();
    let a = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let b = unsafe {
        sched.spawn_with_deps(&[a], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let c = unsafe {
        sched.spawn_with_deps(&[b], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    sched.cancel_task(b);
    let barrier = Arc::new(Barrier::new(2));
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        barrier.wait();
        handle.join().unwrap()
    });
    assert!(order.contains(&a));
    assert!(order.contains(&b));
    // c is finalized as cancelled (dependent of b) and appears in order with WorkSkipped
    assert!(order.contains(&c), "c should be finalized when b is cancelled");
    let state_b = sched.task_state(b);
    assert_eq!(
        state_b,
        Some(TaskState::Finished(TaskCompletionReason::WorkSkipped))
    );
    let state_c = sched.task_state(c);
    assert_eq!(
        state_c,
        Some(TaskState::Finished(TaskCompletionReason::WorkSkipped)),
        "c should be skipped (dependency b was cancelled)"
    );
}

#[test]
fn panic_task_is_finalized() {
    use scheduler::task::{TaskCompletionReason, TaskState};
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let t1 = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
            panic!("intentional panic");
        })
    };
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        barrier.wait();
        handle.join().unwrap()
    });
    assert!(order.contains(&t1));
    let state = sched.task_state(t1);
    assert_eq!(
        state,
        Some(TaskState::Finished(TaskCompletionReason::Failed))
    );
}

#[test]
fn scheduler_shutdown_finalizes_pending_tasks() {
    // This test demonstrates that dropping the scheduler with pending tasks does not panic.
    // Note: No assertions are made here; this is a resource cleanup scenario.
    let mut sched = Scheduler::new();
    let _t1 = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Sleep(Duration::from_millis(1000)));
            ctx.syscall(SystemCall::Done);
        })
    };
    // Simulate shutdown by dropping scheduler before t1 completes
    drop(sched);
    // No panic should occur, and resources should be cleaned up
}

#[test]
fn timeout_task_completion_reason() {
    use scheduler::task::{TaskCompletionReason, TaskState};
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let t1 = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Sleep(Duration::from_millis(10)));
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        barrier.wait();
        handle.join().unwrap()
    });
    assert!(order.contains(&t1));
    // For now, just check that the task is marked as done or skipped
    let state = sched.task_state(t1);
    assert!(
        state == Some(TaskState::Finished(TaskCompletionReason::WorkDone))
            || state == Some(TaskState::Finished(TaskCompletionReason::WorkSkipped)),
        "Task should be done or skipped"
    );
}

/// Stress test: Large parallel fan-out and fan-in DAG. Scheduler in dedicated thread.
#[test]
fn dag_stress_fanout_fanin() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let root = unsafe {
        sched.spawn_with_priority(1, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let mut mids = Vec::new();
    for _ in 0..50 {
        let mid = unsafe {
            sched.spawn_with_deps(&[root], |ctx: TaskContext| {
                ctx.syscall(SystemCall::Done);
            })
        };
        mids.push(mid);
    }
    let leaf = unsafe {
        sched.spawn_with_deps(&mids, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        barrier.wait();
        handle.join().unwrap()
    });
    let pos_root = order.iter().position(|&tid| tid == root).unwrap();
    let pos_leaf = order.iter().position(|&tid| tid == leaf).unwrap();
    // All mids must finish after root and before leaf
    for &mid in &mids {
        let pos_mid = order.iter().position(|&tid| tid == mid).unwrap();
        assert!(pos_root < pos_mid, "Root should finish before mid");
        assert!(pos_mid < pos_leaf, "Mid should finish before leaf");
    }
}

/// Complex DAG: Multiple levels and cross dependencies. Scheduler in dedicated thread.
#[test]
fn dag_complex_multilevel() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let a = unsafe {
        sched.spawn_with_priority(1, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let b = unsafe {
        sched.spawn_with_priority(1, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let c = unsafe {
        sched.spawn_with_deps(&[a], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let d = unsafe {
        sched.spawn_with_deps(&[a, b], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let e = unsafe {
        sched.spawn_with_deps(&[b], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let f = unsafe {
        sched.spawn_with_deps(&[c, d], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let g = unsafe {
        sched.spawn_with_deps(&[d, e], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let h = unsafe {
        sched.spawn_with_deps(&[f, g], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        barrier.wait();
        handle.join().unwrap()
    });
    let pos = |tid| order.iter().position(|&t| t == tid).unwrap();
    // Check all dependencies
    assert!(pos(a) < pos(c), "A before C");
    assert!(pos(a) < pos(d), "A before D");
    assert!(pos(b) < pos(d), "B before D");
    assert!(pos(b) < pos(e), "B before E");
    assert!(pos(c) < pos(f), "C before F");
    assert!(pos(d) < pos(f), "D before F");
    assert!(pos(d) < pos(g), "D before G");
    assert!(pos(e) < pos(g), "E before G");
    assert!(pos(f) < pos(h), "F before H");
    assert!(pos(g) < pos(h), "G before H");
}

/// Stress test: Wide and deep DAG. Scheduler in dedicated thread.
#[test]
fn dag_stress_wide_deep() {
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let mut prev_level = Vec::new();
    for _ in 0..10 {
        let tid = unsafe {
            sched.spawn_with_priority(1, |ctx: TaskContext| {
                ctx.syscall(SystemCall::Done);
            })
        };
        prev_level.push(tid);
    }
    for _ in 0..10 {
        let mut this_level = Vec::new();
        for _ in 0..10 {
            let tid = unsafe {
                sched.spawn_with_deps(&prev_level, |ctx: TaskContext| {
                    ctx.syscall(SystemCall::Done);
                })
            };
            this_level.push(tid);
        }
        prev_level = this_level;
    }
    let final_task = unsafe {
        sched.spawn_with_deps(&prev_level, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        barrier.wait();
        handle.join().unwrap()
    });
    let pos_final = order.iter().position(|&tid| tid == final_task).unwrap();
    // All previous tasks must finish before final_task
    for &tid in &prev_level {
        let pos_tid = order.iter().position(|&t| t == tid).unwrap();
        assert!(pos_tid < pos_final, "All deep tasks before final");
    }
}
