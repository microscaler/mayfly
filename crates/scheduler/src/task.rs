use crate::syscall::SystemCall;
use crossbeam::channel::Sender;

/// Unique identifier for a task.
pub type TaskId = u64;

/// A wrapper around a running coroutine and its metadata.
pub struct Task {
    /// Unique identifier for the task.
    pub tid: TaskId,
    /// Scheduling priority (0 = highest).
    pub pri: u8,
    /// Coroutine handle backing the task.
    pub handle: may::coroutine::JoinHandle<()>,
    /// Current lifecycle state of the task.
    pub state: TaskState,
    /// Whether the task has been cancelled.
    pub cancelled: bool,
}

/// Reason for task completion.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TaskCompletionReason {
    WorkDone,
    WorkSkipped,
    Cancelled,
    Failed,
}

/// Represents the lifecycle state of a task.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TaskState {
    /// The task is waiting for dependencies to be resolved.
    PendingDependencies,
    /// The task is ready to be scheduled.
    Ready,
    /// The task is currently running.
    Running,
    /// The task completed, with a reason.
    Finished(TaskCompletionReason),
}

/// Shared context passed into each task.
///
/// Allows tasks to submit system calls to the scheduler.
#[derive(Clone)]
pub struct TaskContext {
    pub tid: TaskId,
    pub syscall_tx: Sender<(TaskId, SystemCall)>,
}

impl TaskContext {
    /// Submit a system call from the current task.
    pub fn syscall(&self, call: SystemCall) {
        self.syscall_tx
            .send((self.tid, call))
            .expect("Failed to send system call");
        // Yield after sending the syscall so the scheduler can handle it
        // promptly. `may::coroutine::yield_now` already falls back to
        // `std::thread::yield_now` when not in a coroutine context.
        may::coroutine::yield_now();
    }

    /// Yield back to the scheduler without performing a system call.
    pub fn yield_now(&self) {
        self.syscall(SystemCall::Yield);
    }

    /// Check if the current task has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        // In a real implementation, this would check the scheduler's state for this task.
        // For now, this is a placeholder and always returns false.
        // This should be replaced with a mechanism to query the scheduler for the cancelled flag.
        false
    }
}
