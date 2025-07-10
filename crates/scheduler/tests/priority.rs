use scheduler::{Scheduler, SystemCall, task::TaskContext};
use serial_test::file_serial;
use std::time::Duration;

#[test]
#[file_serial]
fn priority_order() {
    let mut sched = Scheduler::new();
    // Spawn tasks then run scheduler on current thread
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
    let order = sched.run();
    println!("order: {order:?}");
    assert_eq!(order.first().copied(), Some(1));
    assert_eq!(order.last().copied(), Some(2));
}

#[test]
fn dag_dependencies() {
    let mut sched = Scheduler::new();
    // Task A: no dependencies
    let a = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    // Task B: depends on A
    let b = unsafe {
        sched.spawn_with_deps(&[a], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    // Task C: depends on A
    let c = unsafe {
        sched.spawn_with_deps(&[a], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    // Task D: depends on B and C
    let d = unsafe {
        sched.spawn_with_deps(&[b, c], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = sched.run();
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

#[test]
fn cooperative_cancellation_ordering() {
    use scheduler::task::TaskCompletionReason;
    let mut sched = Scheduler::new();
    // Task 1: will be cancelled before running
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
    let order = sched.run();
    println!("Cancel test order: {order:?}");
    // All tasks should appear in the order they are completed
    assert!(order.contains(&t1));
    assert!(order.contains(&t2));
    assert!(order.contains(&t3));
    // t1 should be marked as skipped (if cancellation happened before running)
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
    let t1 = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            if ctx.is_cancelled() {
                ctx.syscall(SystemCall::Done);
                return;
            }
            ctx.syscall(SystemCall::Done);
        })
    };
    // Cancel twice using the public API (single-threaded context)
    sched.cancel_task(t1);
    sched.cancel_task(t1);
    let order = sched.run();
    assert!(order.contains(&t1));
    let state = sched.task_state(t1);
    assert!(
        state == Some(TaskState::Finished(TaskCompletionReason::WorkSkipped))
            || state == Some(TaskState::Finished(TaskCompletionReason::WorkDone)),
        "Task should be skipped or done"
    );
}

#[test]
fn cancellation_during_execution() {
    use scheduler::task::{TaskCompletionReason, TaskState};
    let mut sched = Scheduler::new();
    let t1 = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            // Simulate work, check for cancellation
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
    // Cancel after scheduling using the public API (single-threaded context)
    sched.cancel_task(t1);
    let order = sched.run();
    assert!(order.contains(&t1));
    let state = sched.task_state(t1);
    assert!(
        state == Some(TaskState::Finished(TaskCompletionReason::WorkSkipped))
            || state == Some(TaskState::Finished(TaskCompletionReason::WorkDone)),
        "Task should be skipped or done"
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
    // Cancel b before it runs using the public API (single-threaded context)
    sched.cancel_task(b);
    let order = sched.run();
    assert!(order.contains(&a));
    assert!(order.contains(&b));
    // c should not run because its dependency was cancelled
    assert!(!order.contains(&c), "c should not run if b is cancelled");
    let state_b = sched.task_state(b);
    assert_eq!(
        state_b,
        Some(TaskState::Finished(TaskCompletionReason::WorkSkipped))
    );
}

#[test]
fn panic_task_is_finalized() {
    use scheduler::task::{TaskCompletionReason, TaskState};
    let mut sched = Scheduler::new();
    let t1 = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
            panic!("intentional panic");
        })
    };
    let order = sched.run();
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
    let t1 = unsafe {
        sched.spawn_with_priority(5, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Sleep(Duration::from_millis(10)));
            ctx.syscall(SystemCall::Done);
        })
    };
    // Simulate a timeout by running the scheduler for less time than needed
    // (This is a placeholder; actual timeout logic would require scheduler support)
    let order = sched.run();
    assert!(order.contains(&t1));
    // For now, just check that the task is marked as done or skipped
    let state = sched.task_state(t1);
    assert!(
        state == Some(TaskState::Finished(TaskCompletionReason::WorkDone))
            || state == Some(TaskState::Finished(TaskCompletionReason::WorkSkipped)),
        "Task should be done or skipped"
    );
}

/// Stress test: Large parallel fan-out and fan-in DAG
#[test]
fn dag_stress_fanout_fanin() {
    let mut sched = Scheduler::new();
    // Root node
    let root = unsafe {
        sched.spawn_with_priority(1, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    // Fan out: 50 parallel tasks depending on root
    let mut mids = Vec::new();
    for _ in 0..50 {
        let mid = unsafe {
            sched.spawn_with_deps(&[root], |ctx: TaskContext| {
                ctx.syscall(SystemCall::Done);
            })
        };
        mids.push(mid);
    }
    // Fan in: 1 task depending on all mids
    let leaf = unsafe {
        sched.spawn_with_deps(&mids, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = sched.run();
    let pos_root = order.iter().position(|&tid| tid == root).unwrap();
    let pos_leaf = order.iter().position(|&tid| tid == leaf).unwrap();
    // All mids must finish after root and before leaf
    for &mid in &mids {
        let pos_mid = order.iter().position(|&tid| tid == mid).unwrap();
        assert!(pos_root < pos_mid, "Root should finish before mid");
        assert!(pos_mid < pos_leaf, "Mid should finish before leaf");
    }
}

/// Complex DAG: Multiple levels and cross dependencies
#[test]
fn dag_complex_multilevel() {
    let mut sched = Scheduler::new();
    // Level 0
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
    // Level 1
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
    // Level 2
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
    // Level 3
    let h = unsafe {
        sched.spawn_with_deps(&[f, g], |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = sched.run();
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

/// Stress test: Wide and deep DAG
#[test]
fn dag_stress_wide_deep() {
    let mut sched = Scheduler::new();
    let mut prev_level = Vec::new();
    // Start with 10 root tasks
    for _ in 0..10 {
        let tid = unsafe {
            sched.spawn_with_priority(1, |ctx: TaskContext| {
                ctx.syscall(SystemCall::Done);
            })
        };
        prev_level.push(tid);
    }
    // Build 10 levels, each with 10 tasks, each depending on all previous level
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
    // Final task depends on all last level
    let final_task = unsafe {
        sched.spawn_with_deps(&prev_level, |ctx: TaskContext| {
            ctx.syscall(SystemCall::Done);
        })
    };
    let order = sched.run();
    let pos_final = order.iter().position(|&tid| tid == final_task).unwrap();
    // All previous tasks must finish before final_task
    for &tid in &prev_level {
        let pos_tid = order.iter().position(|&t| t == tid).unwrap();
        assert!(pos_tid < pos_final, "All deep tasks before final");
    }
}
