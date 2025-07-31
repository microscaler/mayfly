use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

use crate::task::TaskId;

/// Entry in the ready queue representing a runnable task.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ReadyEntry {
    /// Task priority. Lower values run first.
    pub pri: u8,
    /// Sequence counter to maintain FIFO ordering for equal priorities.
    pub seq: u64,
    /// Identifier of the runnable task.
    pub tid: TaskId,
}

impl Ord for ReadyEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .pri
            .cmp(&self.pri)
            .then_with(|| other.seq.cmp(&self.seq))
    }
}

impl PartialOrd for ReadyEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Priority queue of runnable task IDs.
#[derive(Default)]
pub struct ReadyQueue {
    heap: BinaryHeap<ReadyEntry>,
    set: HashSet<TaskId>,
}

impl ReadyQueue {
    /// Create an empty ready queue.
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
            set: HashSet::new(),
        }
    }

    /// Push a task entry onto the queue.
    pub fn push(&mut self, entry: ReadyEntry) {
        if self.set.insert(entry.tid) {
            self.heap.push(entry);
        }
    }

    /// Returns `true` if the queue already contains `tid`.
    pub fn contains(&self, tid: TaskId) -> bool {
        self.set.contains(&tid)
    }

    /// Pop the next task ID from the queue.
    pub fn pop(&mut self) -> Option<TaskId> {
        self.heap.pop().map(|entry| {
            self.set.remove(&entry.tid);
            entry.tid
        })
    }

    /// Returns `true` if the queue has no tasks.
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// Returns the number of tasks in the queue.
    pub fn len(&self) -> usize {
        self.heap.len()
    }
}

impl ReadyQueue {
    /// Push a task ID without checking for duplicates. Used only for tests.
    pub fn force_push(&mut self, entry: ReadyEntry) {
        self.heap.push(entry);
    }
}
