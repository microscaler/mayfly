use crossbeam::channel::{Receiver, RecvTimeoutError, Sender, unbounded};
use may::coroutine::JoinHandle;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::{Arc, Barrier};
use std::thread::{Scope, ScopedJoinHandle};
use std::time::{Duration, Instant};

#[cfg(feature = "async-io")]
use mio::{Events, Interest, Poll, Token, unix::SourceFd};

use crate::clock::TickClock;
#[cfg(feature = "async-io")]
use crate::io::IoSource;
use crate::pal::{self, TaskEvent};
use crate::ready_queue::ReadyEntry;
use crate::ready_queue::ReadyQueue;
use crate::syscall::SystemCall;
use crate::task::{Task, TaskContext, TaskId, TaskState};
use crate::wait_map::WaitMap;
use tracing::instrument;

/// Priority level reserved for internal system tasks.
pub const PRI_SYSTEM: u8 = 0;
/// Default priority assigned to user tasks.
pub const PRI_DEFAULT: u8 = 10;

/// Snapshot of runtime state used for diagnostics.
#[derive(Debug, Clone)]
pub struct SchedulerState {
    /// Next task identifier to be assigned.
    pub next_id: TaskId,
    /// Total number of active tasks.
    pub task_count: usize,
    /// Number of tasks currently in the ready queue.
    pub ready_queue_len: usize,
    /// Count of sleeping tasks waiting on a timer.
    pub sleepers: usize,

    /// Terminal states recorded for finished tasks.
    pub states: HashMap<TaskId, TaskState>,
}

/// Core runtime orchestrator managing runnable tasks, pending I/O events,
/// and join waiters.
pub struct Scheduler {
    next_id: TaskId,
    seq: u64,
    syscall_tx: Sender<(TaskId, SystemCall)>,
    syscall_rx: Receiver<(TaskId, SystemCall)>,
    #[cfg(not(feature = "async-io"))]
    io_tx: Sender<u64>,
    #[cfg(not(feature = "async-io"))]
    io_rx: Receiver<u64>,
    #[cfg(feature = "async-io")]
    poll: Poll,
    #[cfg(feature = "async-io")]
    sources: HashMap<Token, Arc<dyn IoSource>>,
    #[cfg(feature = "async-io")]
    next_token: usize,
    clock: TickClock,
    sleepers: BinaryHeap<Reverse<(Instant, TaskId)>>,
    timeout_waiters: BinaryHeap<Reverse<(Instant, TaskId, TaskId)>>,
    tasks: HashMap<TaskId, Task>,
    ready: ReadyQueue,
    wait_map: WaitMap,

    states: HashMap<TaskId, TaskState>,

    // Tasks that need to be finalized (e.g., cancelled before running)
    finalization_queue: Vec<TaskId>,

    // DAG dependency tracking
    dependencies: HashMap<TaskId, HashSet<TaskId>>, // tasks this task depends on
    dependents: HashMap<TaskId, HashSet<TaskId>>,   // tasks that depend on this task
    unresolved_deps: HashMap<TaskId, usize>,        // unresolved dependency count
}

impl Scheduler {
    /// Create a new Scheduler instance.
    pub fn new() -> Self {
        let (syscall_tx, syscall_rx) = unbounded();
        #[cfg(not(feature = "async-io"))]
        let (io_tx, io_rx) = unbounded();
        #[cfg(feature = "async-io")]
        let poll = Poll::new().expect("poll");
        Self {
            next_id: 1,
            seq: 0,
            syscall_tx,
            syscall_rx,
            #[cfg(not(feature = "async-io"))]
            io_tx,
            #[cfg(not(feature = "async-io"))]
            io_rx,
            #[cfg(feature = "async-io")]
            poll,
            #[cfg(feature = "async-io")]
            sources: HashMap::new(),
            #[cfg(feature = "async-io")]
            next_token: 0,
            clock: TickClock::new(Instant::now()),
            sleepers: BinaryHeap::new(),
            timeout_waiters: BinaryHeap::new(),
            tasks: HashMap::new(),
            ready: ReadyQueue::new(),
            wait_map: WaitMap::new(),
            states: HashMap::new(),
            finalization_queue: Vec::new(),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
            unresolved_deps: HashMap::new(),
        }
    }

    /// Return a handle that can be used to signal I/O readiness when
    /// `async-io` is disabled.
    #[cfg(not(feature = "async-io"))]
    pub fn io_handle(&self) -> Sender<u64> {
        self.io_tx.clone()
    }

    /// Register a new I/O source with the scheduler.
    #[cfg(feature = "async-io")]
    pub fn register_io(&mut self, src: Arc<dyn IoSource>) {
        let token = Token(self.next_token);
        self.next_token += 1;
        let fd = src.raw_fd();
        let mut source = SourceFd(&fd);
        self.poll
            .registry()
            .register(&mut source, token, Interest::READABLE)
            .expect("register io source");
        self.sources.insert(token, src);
    }

    /// Return the number of tasks currently in the ready queue.
    pub fn ready_len(&self) -> usize {
        self.ready.len()
    }

    /// Check if the ready queue is empty.
    pub fn ready_is_empty(&self) -> bool {
        self.ready.is_empty()
    }

    /// Spawn a new coroutine task with a specific priority.
    ///
    /// # Safety
    /// This function uses `may::coroutine::spawn`, which is unsafe because it may break Rust's safety guarantees
    /// if the spawned coroutine accesses data that is not properly synchronized or outlives its stack frame.
    /// The caller must ensure that the closure and its captured data are safe to use in this context.
    pub unsafe fn spawn_with_priority<F>(&mut self, pri: u8, f: F) -> TaskId
    where
        F: FnOnce(TaskContext) + Send + 'static,
    {
        let tid = self.next_id;
        self.next_id += 1;

        let ctx = TaskContext {
            tid,
            syscall_tx: self.syscall_tx.clone(),
        };

        let handle: JoinHandle<()> = unsafe { may::coroutine::spawn(move || f(ctx)) };

        self.states.insert(tid, TaskState::Running);
        self.tasks.insert(
            tid,
            Task {
                tid,
                pri,
                handle,
                state: TaskState::Running,
                cancelled: false,
            },
        );
        let entry = ReadyEntry {
            pri,
            seq: self.seq,
            tid,
        };
        self.seq += 1;
        self.ready.push(entry);
        tid
    }

    /// Spawn a new coroutine task with default priority (10).
    ///
    /// # Safety
    /// This function uses `may::coroutine::spawn`, which is unsafe because it may break Rust's safety guarantees
    /// if the spawned coroutine accesses data that is not properly synchronized or outlives its stack frame.
    /// The caller must ensure that the closure and its captured data are safe to use in this context.
    pub unsafe fn spawn<F>(&mut self, f: F) -> TaskId
    where
        F: FnOnce(TaskContext) + Send + 'static,
    {
        unsafe { self.spawn_with_priority(10, f) }
    }

    /// Spawn a new coroutine task with dependencies (DAG support).
    ///
    /// # Panics
    /// Panics if a dependency cycle is detected or if any dependency does not exist.
    ///
    /// # Safety
    /// This function uses `may::coroutine::spawn`, which is unsafe because it may break Rust's safety guarantees
    /// if the spawned coroutine accesses data that is not properly synchronized or outlives its stack frame.
    /// The caller must ensure that the closure and its captured data are safe to use in this context.
    pub unsafe fn spawn_with_deps<F>(&mut self, deps: &[TaskId], f: F) -> TaskId
    where
        F: FnOnce(TaskContext) + Send + 'static,
    {
        let tid = self.next_id;
        self.next_id += 1;

        // Validate dependencies exist and are not self-referential
        for &dep in deps {
            if dep == tid {
                panic!("Task cannot depend on itself");
            }
            if !self.tasks.contains_key(&dep) && !self.states.contains_key(&dep) {
                panic!("Dependency TaskId {dep} does not exist");
            }
        }

        // Cycle detection (DFS)
        fn has_cycle(
            dependencies: &HashMap<TaskId, HashSet<TaskId>>,
            start: TaskId,
            target: TaskId,
            visited: &mut HashSet<TaskId>,
        ) -> bool {
            if !visited.insert(start) {
                return false;
            }
            if let Some(deps) = dependencies.get(&start) {
                for &dep in deps {
                    if dep == target || has_cycle(dependencies, dep, target, visited) {
                        return true;
                    }
                }
            }
            false
        }
        for &dep in deps {
            let mut visited = HashSet::new();
            if has_cycle(&self.dependencies, dep, tid, &mut visited) {
                panic!("Cycle detected in task dependencies");
            }
        }

        // Register dependencies and dependents
        let dep_set: HashSet<TaskId> = deps.iter().copied().collect();
        let unresolved = dep_set.len();
        self.dependencies.insert(tid, dep_set.clone());
        self.unresolved_deps.insert(tid, unresolved);
        for &dep in deps {
            self.dependents.entry(dep).or_default().insert(tid);
        }

        let ctx = TaskContext {
            tid,
            syscall_tx: self.syscall_tx.clone(),
        };

        let handle: JoinHandle<()> = unsafe { may::coroutine::spawn(move || f(ctx)) };

        if unresolved == 0 {
            self.states.insert(tid, TaskState::Ready);
        } else {
            self.states.insert(tid, TaskState::PendingDependencies);
        }
        self.tasks.insert(
            tid,
            Task {
                tid,
                pri: PRI_DEFAULT,
                handle,
                state: TaskState::Running,
                cancelled: false,
            },
        );

        // Only schedule if no unresolved dependencies
        if unresolved == 0 {
            let entry = ReadyEntry {
                pri: PRI_DEFAULT,
                seq: self.seq,
                tid,
            };
            self.seq += 1;
            self.ready.push(entry);
        }

        tid
    }

    /// Spawn a new coroutine task reserved for internal system work.
    ///
    /// System tasks use \[`PRI_SYSTEM`\] to ensure they run before normal
    /// user tasks. See \[`spawn_with_priority`\] for details.
    ///
    /// # Safety
    /// This method ultimately delegates to \[`may::coroutine::spawn`\]. The
    /// caller must guarantee that the provided closure does not reference
    /// data which could be invalid once the task begins execution.
    pub unsafe fn spawn_system<F>(&mut self, f: F) -> TaskId
    where
        F: FnOnce(TaskContext) + Send + 'static,
    {
        unsafe { self.spawn_with_priority(PRI_SYSTEM, f) }
    }

    /// Run the scheduler loop, processing system calls from tasks.
    #[cfg(not(feature = "async-io"))]
    pub fn run(&mut self) -> Vec<TaskId> {
        let mut done_order = Vec::new();
        let mut idle_ticks = 0;
        const MAX_IDLE_TICKS: usize = 3;

        loop {
            let mut did_work = false;

            while let Some(&Reverse((wake_at, tid))) = self.sleepers.peek() {
                if wake_at <= self.clock.now() {
                    self.sleepers.pop();
                    self.push_ready(tid);
                    did_work = true;
                } else {
                    break;
                }
            }

            while let Some(&Reverse((wake_at, waiter, target))) = self.timeout_waiters.peek() {
                if wake_at <= self.clock.now() {
                    self.timeout_waiters.pop();
                    if self.wait_map.remove_waiter(target, waiter) {
                        self.push_ready(waiter);
                    }
                } else {
                    break;
                }
            }

            while let Some(&Reverse((wake_at, waiter, target))) = self.timeout_waiters.peek() {
                if wake_at <= self.clock.now() {
                    self.timeout_waiters.pop();
                    if self.wait_map.remove_waiter(target, waiter) {
                        self.push_ready(waiter);
                    }
                } else {
                    break;
                }
            }

            while let Ok((call_tid, syscall)) = self.syscall_rx.try_recv() {
                self.handle_syscall(call_tid, syscall, &mut done_order);
                did_work = true;
            }

            while let Ok(io_id) = self.io_rx.try_recv() {
                for tid in self.wait_map.complete_io(io_id) {
                    self.push_ready(tid);
                    did_work = true;
                }
            }

            let tid = match self.ready.pop() {
                Some(id) => id,
                None => {
                    if let Some(wake_at) = self.next_wake_instant() {
                        if wake_at > self.clock.now() {
                            let diff = wake_at.duration_since(self.clock.now());
                            self.clock.tick(diff);
                        }
                        continue;
                    }
                    match self.io_rx.recv_timeout(Duration::from_millis(5)) {
                        Ok(io_id) => {
                            for tid in self.wait_map.complete_io(io_id) {
                                self.push_ready(tid);
                            }
                            continue;
                        }
                        Err(_) => {}
                    }
                    // No more IO events, check for idle ticks
                    if !did_work
                        && self.tasks.is_empty()
                        && self.finalization_queue.is_empty()
                        && self.ready.is_empty()
                        && self.syscall_rx.is_empty()
                    {
                        idle_ticks += 1;
                        if idle_ticks >= MAX_IDLE_TICKS {
                            break;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(1));
                    } else {
                        idle_ticks = 0;
                    }
                    continue;
                }
            };

            if !self.tasks.contains_key(&tid) {
                continue;
            }

            // Only schedule if state is Ready
            if let Some(TaskState::Ready) = self.states.get(&tid) {
                // Finalization step for cancelled tasks: if cancelled, immediately finalize
                if let Some(task) = self.tasks.get(&tid)
                    && task.cancelled
                {
                    self.states.insert(
                        tid,
                        TaskState::Finished(crate::task::TaskCompletionReason::WorkSkipped),
                    );
                    done_order.push(tid);
                    self.tasks.remove(&tid);
                    continue;
                }
                self.states.insert(tid, TaskState::Running);
            } else {
                continue;
            }

            let _task = self.tasks.get_mut(&tid).expect("task not found");

            match self.syscall_rx.recv_timeout(Duration::from_millis(5)) {
                Ok((call_tid, syscall)) => {
                    if call_tid != tid && self.tasks.contains_key(&tid) {
                        self.push_ready(tid);
                    }
                    self.handle_syscall(call_tid, syscall, &mut done_order);
                }
                Err(RecvTimeoutError::Timeout) => {
                    tracing::warn!("scheduler idle timeout");
                }
                Err(RecvTimeoutError::Disconnected) => {}
            }
            // Finalize any tasks in the finalization queue
            while let Some(tid) = self.finalization_queue.pop() {
                if let Some(task) = self.tasks.get(&tid) {
                    if task.cancelled {
                        self.states.insert(
                            tid,
                            TaskState::Finished(crate::task::TaskCompletionReason::WorkSkipped),
                        );
                        done_order.push(tid);
                        self.tasks.remove(&tid);
                    }
                }
            }
        }
        done_order
    }

    #[cfg(feature = "async-io")]
    pub fn run(&mut self) -> Vec<TaskId> {
        let mut done_order = Vec::new();
        let mut events = Events::with_capacity(8);
        while !self.tasks.is_empty()
            || !self.finalization_queue.is_empty()
            || !self.ready.is_empty()
        {
            let timeout = if self.ready.is_empty() {
                if let Some(wake_at) = self.next_wake_instant() {
                    let now = self.clock.now();
                    if wake_at > now {
                        wake_at - now
                    } else {
                        Duration::ZERO
                    }
                } else {
                    Duration::from_secs(5)
                }
            } else {
                Duration::ZERO
            };

            if let Err(e) = self.poll.poll(&mut events, Some(timeout)) {
                tracing::warn!(?e, "poll error");
                break;
            }

            if events.is_empty()
                && self.ready.is_empty()
                && self.sleepers.is_empty()
                && self.timeout_waiters.is_empty()
                && timeout == Duration::from_secs(5)
            {
                break;
            }

            if events.is_empty() && self.ready.is_empty() && timeout > Duration::ZERO {
                self.clock.tick(timeout);
            }

            for ev in events.iter() {
                if let Some(src) = self.sources.get(&ev.token()) {
                    for tid in self.wait_map.complete_io(src.id()) {
                        self.push_ready(tid);
                    }
                }
            }
            events.clear();

            while let Some(&Reverse((wake_at, tid))) = self.sleepers.peek() {
                if wake_at <= self.clock.now() {
                    self.sleepers.pop();
                    self.push_ready(tid);
                } else {
                    break;
                }
            }

            while let Ok((call_tid, syscall)) = self.syscall_rx.try_recv() {
                self.handle_syscall(call_tid, syscall, &mut done_order);
            }

            let tid = match self.ready.pop() {
                Some(id) => id,
                None => continue,
            };

            if !self.tasks.contains_key(&tid) {
                continue;
            }

            // Only schedule if state is Ready
            if let Some(TaskState::Ready) = self.states.get(&tid) {
                // Finalization step for cancelled tasks: if cancelled, immediately finalize
                if let Some(task) = self.tasks.get(&tid)
                    && task.cancelled
                {
                    self.states.insert(
                        tid,
                        TaskState::Finished(crate::task::TaskCompletionReason::WorkSkipped),
                    );
                    done_order.push(tid);
                    self.tasks.remove(&tid);
                    continue;
                }
                self.states.insert(tid, TaskState::Running);
            } else {
                continue;
            }

            let _task = self.tasks.get_mut(&tid).expect("task not found");

            match self.syscall_rx.recv_timeout(Duration::from_secs(5)) {
                Ok((call_tid, syscall)) => {
                    if call_tid != tid && self.tasks.contains_key(&tid) {
                        self.push_ready(tid);
                    }
                    self.handle_syscall(call_tid, syscall, &mut done_order);
                }
                Err(RecvTimeoutError::Timeout) => {
                    tracing::warn!("scheduler idle timeout");
                    break;
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        // Finalize any tasks in the finalization queue
        while let Some(tid) = self.finalization_queue.pop() {
            if let Some(task) = self.tasks.get(&tid)
                && task.cancelled
            {
                self.states.insert(
                    tid,
                    TaskState::Finished(crate::task::TaskCompletionReason::WorkSkipped),
                );
                done_order.push(tid);
                self.tasks.remove(&tid);
            }
        }
        done_order
    }

    /// Start the scheduler loop on a dedicated thread.
    ///
    /// The provided `barrier` is used to coordinate when the loop begins.
    /// Tasks should be spawned before the barrier is released.
    ///
    /// # Safety
    /// This method spawns a thread that operates on `&mut self`. The caller
    /// must ensure no other references to `self` are used once the barrier is
    /// released and until the returned handle has completed.
    pub unsafe fn start<'scope>(
        &mut self,
        scope: &'scope Scope<'scope, '_>,
        barrier: Arc<Barrier>,
    ) -> ScopedJoinHandle<'scope, Vec<TaskId>> {
        struct Ptr(*mut Scheduler);
        unsafe impl Send for Ptr {}

        let ptr = Ptr(self as *mut Scheduler);
        scope.spawn(move || {
            let p = ptr;
            barrier.wait();
            // SAFETY: exclusive access is guaranteed after the barrier.
            unsafe { &mut *p.0 }.run()
        })
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    /// Cancel a task by TaskId, scheduling it for cooperative cancellation and finalization.
    ///
    /// # Usage
    /// - This method is intended for use in single-threaded or test contexts where you have
    ///   exclusive access to the scheduler instance.
    /// - In multi-threaded or concurrent scenarios (such as when using thread::scope or spawning
    ///   tasks that interact with the scheduler), prefer to issue a `SystemCall::Cancel(target)`
    ///   from within a task closure, rather than calling this method directly.
    ///
    /// # Behavior
    /// - If the task is pending or ready, it will be scheduled for finalization and appear in the
    ///   completion order with a `WorkSkipped` reason.
    /// - If the task is running, it will be marked as cancelled and should check for cancellation
    ///   cooperatively via `TaskContext::is_cancelled()`.
    /// - Double cancellation is safe and idempotent.
    /// - Cancelled tasks always appear in the completion order with the correct completion reason.
    ///
    /// # Example (single-threaded test)
    /// ```
    /// let mut sched = Scheduler::new();
    /// let tid = sched.spawn_with_priority(5, |ctx| {
    ///     if ctx.is_cancelled() { ctx.syscall(SystemCall::Done); return; }
    ///     ctx.syscall(SystemCall::Done);
    /// });
    /// sched.cancel_task(tid);
    /// let order = sched.run();
    /// assert!(order.contains(&tid));
    /// ```
    pub fn cancel_task(&mut self, tid: TaskId) {
        self.handle_syscall(tid, SystemCall::Cancel(tid), &mut Vec::new());
    }
    /// Directly push a task ID into the ready queue without deduplication.
    pub fn ready_push_duplicate_for_test(&mut self, tid: TaskId) {
        let pri = self.tasks.get(&tid).map(|t| t.pri).unwrap_or(10);
        self.ready.force_push(ReadyEntry {
            pri,
            seq: self.seq,
            tid,
        });
        self.seq += 1;
    }

    /// Insert `tid` into the ready queue respecting its priority.
    fn push_ready(&mut self, tid: TaskId) {
        if let Some(task) = self.tasks.get(&tid) {
            let entry = ReadyEntry {
                pri: task.pri,
                seq: self.seq,
                tid,
            };
            self.seq += 1;
            self.ready.push(entry);
        }
    }

    /// Return the earliest instant at which a sleeping task should be woken.
    fn next_wake_instant(&self) -> Option<Instant> {
        let sleep = self.sleepers.peek().map(|Reverse((when, _))| *when);
        let timeout = self
            .timeout_waiters
            .peek()
            .map(|Reverse((when, _, _))| *when);
        match (sleep, timeout) {
            (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    }

    /// Process a syscall emitted by `tid`, updating scheduler state and queueing follow-up work.
    fn handle_syscall(&mut self, tid: TaskId, syscall: SystemCall, done: &mut Vec<TaskId>) {
        let mut requeue = true;
        match syscall {
            SystemCall::Log(msg) => tracing::info!(task = %tid, "{}", msg),
            SystemCall::Sleep(dur) => {
                tracing::info!(task = %tid, "sleeping {:?}", dur);
                let wake_at = self.clock.now() + dur;
                self.sleepers.push(Reverse((wake_at, tid)));
                requeue = false;
            }
            SystemCall::Done => {
                tracing::info!(task = %tid, "task done");
                let state = if let Some(task) = self.tasks.remove(&tid) {
                    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        task.handle.join()
                    }));
                    match res {
                        Ok(Ok(_)) => {
                            if task.cancelled {
                                TaskState::Finished(crate::task::TaskCompletionReason::WorkSkipped)
                            } else {
                                TaskState::Finished(crate::task::TaskCompletionReason::WorkDone)
                            }
                        }
                        _ => {
                            pal::emit(TaskEvent::Failed(tid));
                            TaskState::Finished(crate::task::TaskCompletionReason::Failed)
                        }
                    }
                } else {
                    TaskState::Finished(crate::task::TaskCompletionReason::WorkDone)
                };
                self.states.insert(tid, state);
                let (waiters, _) = self.wait_map.complete(tid, state);
                for waiter in waiters {
                    self.push_ready(waiter);
                }
                // DAG dependency resolution: notify dependents
                self.on_task_complete(tid);
                done.push(tid);
                requeue = false;
            }
            SystemCall::Join(target) => {
                if self.tasks.contains_key(&target) {
                    self.wait_map.wait_for(target, tid);
                    requeue = false;
                }
            }
            SystemCall::JoinTimeout { target, dur } => {
                if self.tasks.contains_key(&target) {
                    self.wait_map.wait_for(target, tid);
                    let wake_at = self.clock.now() + dur;
                    self.timeout_waiters.push(Reverse((wake_at, tid, target)));
                    requeue = false;
                }
            }
            SystemCall::Cancel(target) => {
                if let Some(task) = self.tasks.get_mut(&target) {
                    task.cancelled = true;
                    // If the task is not running, ensure it is pushed to the ready queue or finalization queue for finalization
                    if let Some(TaskState::PendingDependencies) | Some(TaskState::Ready) =
                        self.states.get(&target)
                    {
                        // Only push if not already in the ready queue
                        if !self.ready.contains(target) {
                            self.finalization_queue.push(target);
                        }
                    }
                }
            }
            SystemCall::IoWait(io_id) => {
                self.wait_map.wait_io(io_id, tid);
                requeue = false;
            }
            SystemCall::Yield => {
                // Cooperative yield: no action required other than requeueing
            }
        }
        if requeue && self.tasks.contains_key(&tid) {
            self.push_ready(tid);
        }
    }

    /// DAG dependency resolution: called when a task completes.
    fn on_task_complete(&mut self, tid: TaskId) {
        // Notify dependents
        if let Some(dependents) = self.dependents.get(&tid).cloned() {
            for dep_tid in dependents {
                if let Some(count) = self.unresolved_deps.get_mut(&dep_tid) {
                    *count -= 1;
                    if *count == 0 {
                        // All dependencies resolved, schedule the task
                        self.states.insert(dep_tid, TaskState::Ready);
                        if let Some(task) = self.tasks.get(&dep_tid) {
                            let entry = ReadyEntry {
                                pri: task.pri,
                                seq: self.seq,
                                tid: dep_tid,
                            };
                            self.seq += 1;
                            self.ready.push(entry);
                        }
                    }
                }
            }
            self.dependents.remove(&tid);
        }
        // Clean up dependency tracking for tid
        self.dependencies.remove(&tid);
        self.unresolved_deps.remove(&tid);
    }

    /// Retrieve the recorded state of a task if known.
    pub fn task_state(&self, tid: TaskId) -> Option<TaskState> {
        self.states.get(&tid).copied()
    }

    /// Capture a diagnostic snapshot of the scheduler state.
    #[instrument(skip(self))]
    pub fn dump_state(&self) -> SchedulerState {
        SchedulerState {
            next_id: self.next_id,
            task_count: self.tasks.len(),
            ready_queue_len: self.ready.len(),
            sleepers: self.sleepers.len(),

            states: self.states.clone(),
        }
    }
}
