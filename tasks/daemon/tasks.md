# Tiffany Daemon Crate Tasks

## Goal
Build the binary entrypoint for the `tinkerbell` agent, responsible for:
- Config file loading
- Signal handling
- Starting the scheduler runtime loop

## Tasks

### Initial Setup

- [ ] Create `bin/tinkerbell.rs` to:
    - Load config
    - Initialize logging and metrics
    - Call `daemon::run()`
    - Set up tracing via `tracing_subscriber` honoring `RUST_LOG`

- [ ] In `lib.rs`, implement `init()` and `run()`
- [ ] Add placeholder config loader (`config.rs`) using `serde` + `toml`
- [ ] Plan file-watcher support for hot-reload of config (inotify / fsevents)
- [ ] Replace placeholder daemon tests with a check that init and run return successfully

### Scheduler Bootstrapping

- [ ] Construct the scheduler with sane defaults
- [ ] Drive it using `run_blocking()`

### Graceful Shutdown

- [ ] Register `signal_hook` handlers for `SIGINT` and `SIGTERM`
- [ ] Emit PAL events for `ShutdownBegin` and `ShutdownComplete`
- [ ] Call `scheduler.shutdown(timeout)` when a signal is received

### System Tasks

- [ ] Implement `looptask_wal_flush()` and spawn it with priority `0`
- [ ] Implement `looptask_metrics()` and spawn it with priority `0`

### Metrics and Health Endpoints

- [ ] Serve Prometheus metrics at `/metrics`
- [ ] Provide a basic health check at `/__health`

### Optional Interfaces (Feature Flags)

- [ ] `ipc` feature: accept jobs via Unix socket
- [ ] `grpc` feature: accept jobs over gRPC
- [ ] `a2a` feature: emit and receive A2A protocol messages

### Future Extensions

- [x] CLI flags for concurrency, logging, and quantum
- [x] `--dump-state` flag to output the scheduler snapshot
- [ ] Add integration test to spawn the daemon and shut it down gracefully
