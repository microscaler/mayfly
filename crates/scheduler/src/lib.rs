//! Coroutine-based cooperative task scheduler (may runtime).
//!
//! **Channel model:** The syscall channel is **MPSC** (many tasks → one run loop). Tasks run on the
//! thread that called `spawn`; the run loop must run on that same thread, or use the spawn-request
//! API so a dedicated scheduler thread owns spawning. See [design-mayfly-channels-mpmc](../../../docs/design-mayfly-channels-mpmc.md)
//! for MPMC options (multiple consumer threads require shared/partitioned state).

mod clock;
pub mod io;
mod pal;
pub mod ready_queue;
pub mod scheduler;
pub mod syscall;
pub mod task;
mod wait_map;

pub use io::IoSource;
pub use pal::{TaskEvent, emit as pal_emit};
pub use ready_queue::{ReadyEntry, ReadyQueue};
pub use scheduler::{Scheduler, SpawnRequest, SchedulerState};
pub use syscall::SystemCall;
pub use task::TaskContext;
pub use task::{Task, TaskId};
pub use wait_map::WaitMap;
