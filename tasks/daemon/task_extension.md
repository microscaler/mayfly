# 📦 Codex Task Protocol: Mayfly Daemon

This task file defines the responsibilities and roadmap for the `daemon` crate within Mayfly. The daemon serves as the long-running runtime process that embeds the scheduler and optionally exposes interfaces for remote job submission, lifecycle control, and system metrics.

---

## 🧠 Purpose

The `daemon` crate turns Mayfly from a library into a service.
It should:

* Bootstrap and run the scheduler
* Spawn internal system tasks (telemetry, WAL flushers, etc.)
* Handle Unix signals for shutdown
* Optionally expose interfaces like IPC, gRPC, or A2A job ingestion

---

## 📋 Core Tasks

### ✅ Bootstrapping

* [ ] Implement `main()` entrypoint
* [ ] Call `Scheduler::new()` or builder variant with sane defaults
* [ ] Run `sched.run_blocking()` and handle completion

### 🔌 Graceful Shutdown

* [ ] Register `ctrlc` or `signal_hook` handler for `SIGTERM` / `SIGINT`
* [ ] On signal, call `scheduler.shutdown(timeout)`
* [ ] Emit logs/PAL events for `ShutdownBegin` and `ShutdownComplete`

### ⚙️ Spawn System Tasks

* [ ] Spawn WAL flusher (`looptask_wal_flush()` every N ms)
* [ ] Spawn metrics pusher (`looptask_metrics()`)
* [ ] Use `spawn_system()` and verify priority = 0

### 📡 Metrics and Health

* [ ] Expose Prometheus `/metrics` HTTP server (via `hyper` or `tiny_http`)
* [ ] Ensure metrics include scheduler-level counters and histograms
* [ ] Provide `/__health` endpoint returning 200 OK when ready

### 📬 Optional Interfaces (Feature Flags)

* [ ] Feature: `ipc` – accept jobs via Unix socket
* [ ] Feature: `grpc` – accept jobs over gRPC (Tonic)
* [ ] Feature: `a2a` – accept and emit jobs as A2A protocol messages

### 🔐 Logging and Tracing

* [ ] Configure `tracing_subscriber` in main
* [ ] Respect `RUST_LOG` and output to stderr

---

## 🚧 Advanced / Future

* [ ] CLI flags for concurrency, logging, quantum, etc.
* [ ] CLI `--dump-state` to emit snapshot of tasks and queues
* [ ] Run in supervisor mode for multi-node mesh discovery

---

## 🧪 Test Guidelines

* Use `serial_test::serial` to isolate integration tests
* Simulate SIGINT via `nix::sys::signal::kill(getpid(), SIGINT)`
* Validate metric exposure using HTTP client + assert

---

## 🧾 Output Requirements

* All functions and structs must have doc-comments
* All `main.rs` logic must be instrumented with `#[instrument]`
* PAL events must be emitted for:

    * Scheduler boot
    * Scheduler shutdown begin/complete
    * System task start and exit

---

Happy daemoning 🐝
