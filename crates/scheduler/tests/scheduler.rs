use serial_test::file_serial;

#[test]
#[file_serial]
fn test_may_scheduler() {
    let mut sched = scheduler::Scheduler::new();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    std::thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };
        unsafe {
            sched.spawn(|_| {
                println!("hello from may coroutine!");
            });
        }
        barrier.wait();
        let _ = handle.join().unwrap();
    });
}
