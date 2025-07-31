# 🔌 Task: Introduce `IoSource` Trait & Registry

## Goal
Abstract file-descriptor readiness so any FD (pipe, TCP, eventfd, etc.)
can be registered and waited on by tasks via `SystemCall::IoWait(fd)`.

## Acceptance
* New trait `IoSource` in `scheduler::io`:

  ```rust
  pub trait IoSource: Send + Sync {
      fn raw_fd(&self) -> std::os::unix::io::RawFd;
      fn id(&self) -> u64; // unique id for WaitMap
  }


* `Scheduler` exposes:

  ```rust
  pub fn register_io(&mut self, src: Arc<dyn IoSource>);
  ```

* Unit-test registers a pipe FD, writes to the writer end,
  and proves the reader task wakes from `SystemCall::IoWait(id)`.

---

### Steps

1. **Create `crates/scheduler/src/io.rs`** and define the trait.

2. **Add field** in `Scheduler`:

   ```rust
   #[cfg(feature = "async-io")]
   poll: mio::Poll,
   sources: HashMap<u64, Arc<dyn IoSource>>,
   ```

3. **`register_io()`**: insert into map and `poll.registry().register(...)`.

4. Update `WaitMap::io_waiters` to key by `source.id()` (not raw fd).

5. Add a small test `io_register.rs` using `pipe()` from `nix`.

