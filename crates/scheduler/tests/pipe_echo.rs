#[cfg(feature = "async-io")]
use nix::unistd::{close, pipe, write};
#[cfg(feature = "async-io")]
use scheduler::{Scheduler, SystemCall, io::IoSource, task::TaskContext};
#[cfg(feature = "async-io")]
use serial_test::file_serial;
#[cfg(feature = "async-io")]
use std::os::unix::io::RawFd;
#[cfg(feature = "async-io")]
use std::sync::{Arc, Barrier};
#[cfg(feature = "async-io")]
use std::thread;
#[cfg(feature = "async-io")]
use std::time::Duration;

#[cfg(feature = "async-io")]
struct PipeReader {
    fd: RawFd,
    id: u64,
}

#[cfg(feature = "async-io")]
impl IoSource for PipeReader {
    fn raw_fd(&self) -> RawFd {
        self.fd
    }
    fn id(&self) -> u64 {
        self.id
    }
}

#[cfg(feature = "async-io")]
#[test]
#[file_serial]
fn pipe_echo() {
    let (rfd, wfd) = pipe().unwrap();
    let mut sched = Scheduler::new();
    let barrier = Arc::new(Barrier::new(2));
    let reader = Arc::new(PipeReader { fd: rfd, id: 1 });

    sched.register_io(reader);

    let order = thread::scope(|s| {
        let handle = unsafe { sched.start(s, barrier.clone()) };

        unsafe {
            sched.spawn(|ctx: TaskContext| {
                ctx.syscall(SystemCall::IoWait(1));
                ctx.syscall(SystemCall::Done);
            });
        }

        thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            write(wfd, &[1u8]).unwrap();
            let _ = close(wfd);
        });

        barrier.wait();
        let order = handle.join().unwrap();
        assert_eq!(order, vec![1]);
        order
    });

    let _ = close(rfd);
    assert_eq!(order, vec![1]);
}
