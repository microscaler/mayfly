use scheduler::{Scheduler, syscall::SystemCall, task::TaskContext};
use serial_test::file_serial;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

#[test]
#[file_serial]
fn test_task_log_and_sleep_with_may() {
    let mut scheduler = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    thread::scope(|s| {
        let handle = unsafe { scheduler.start(s, barrier.clone()) };

        unsafe {
            scheduler.spawn(|ctx: TaskContext| {
                ctx.syscall(SystemCall::Log("start task A".into()));
                ctx.syscall(SystemCall::Sleep(Duration::from_millis(100)));
                ctx.syscall(SystemCall::Log("resume task A".into()));
                ctx.syscall(SystemCall::Done);
            });
        }

        unsafe {
            scheduler.spawn(|ctx: TaskContext| {
                ctx.syscall(SystemCall::Log("start task B".into()));
                ctx.syscall(SystemCall::Sleep(Duration::from_millis(100)));
                ctx.syscall(SystemCall::Log("resume task B".into()));
                ctx.syscall(SystemCall::Done);
            });
        }

        barrier.wait();
        let _ = handle.join().unwrap();
    });
}
