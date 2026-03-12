use scheduler::Scheduler;

#[test]
fn compile() {
    let mut sched = Scheduler::new();
    assert!(sched.ready_is_empty());
    let order = sched.run();
    assert!(order.is_empty());
}
