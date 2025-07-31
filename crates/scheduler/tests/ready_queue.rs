use scheduler::ready_queue::{ReadyEntry, ReadyQueue};
use serial_test::file_serial;

#[test]
#[file_serial]
fn test_ready_queue_fifo() {
    let mut q = ReadyQueue::new();
    q.push(ReadyEntry {
        pri: 10,
        seq: 0,
        tid: 1,
    });
    q.push(ReadyEntry {
        pri: 10,
        seq: 1,
        tid: 2,
    });
    assert_eq!(q.len(), 2);
    assert_eq!(q.pop(), Some(1));
    assert_eq!(q.pop(), Some(2));
    assert!(q.is_empty());
}

#[test]
#[file_serial]
fn test_ready_queue_no_duplicates() {
    let mut q = ReadyQueue::new();
    q.push(ReadyEntry {
        pri: 10,
        seq: 0,
        tid: 1,
    });
    q.push(ReadyEntry {
        pri: 10,
        seq: 1,
        tid: 1,
    });
    assert_eq!(q.len(), 1);
    assert_eq!(q.pop(), Some(1));
    assert!(q.is_empty());
}
