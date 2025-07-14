mayfly/tasks/scheduler/dependency_graphs.md
# Product Requirements Document: DAG-Based Task Dependencies for Scheduler

## Overview

This PRD proposes an enhancement to the Tiffany Scheduler to support explicit, robust, and scalable task dependencies using a Directed Acyclic Graph (DAG) model. This will enable deterministic execution of complex workflows, maximize concurrency, and eliminate flakiness in tests and production systems.

---

## Goals

- Allow tasks to declare dependencies on one or more other tasks at spawn time.
- Ensure a task is only scheduled for execution when all its dependencies have completed.
- Prevent cycles in the dependency graph (enforce acyclicity).
- Maximize concurrency: all tasks with satisfied dependencies should run as soon as possible.
- Provide clear, ergonomic APIs for declaring dependencies.
- Maintain or improve test determinism and reproducibility.

---

## Non-Goals

- Dynamic modification of dependencies after task creation (initial version).
- Support for cyclic dependencies (explicitly disallowed).
- Workflow-level features (e.g., retries, conditional branches) beyond dependency management.

---

## User Stories

1. **As a developer**, I want to spawn a task that depends on multiple other tasks, so that it only runs after all its prerequisites are complete.
2. **As a test author**, I want to express test order and dependencies explicitly, so my tests are deterministic and robust.
3. **As a system integrator**, I want the scheduler to maximize parallelism by running all ready tasks as soon as their dependencies are satisfied.

---

## Requirements

### API Changes

- Add a new method to the scheduler:
    ```rust
    fn spawn_with_deps<F>(&mut self, deps: &[TaskId], f: F) -> TaskId
    where
        F: FnOnce(TaskContext) + Send + 'static;
    ```
- Existing `spawn` and `spawn_with_priority` methods remain, defaulting to no dependencies.

### Scheduler Internals

#### **Dependency Tracking Structures**

- Add the following fields to the `Scheduler` struct:
    ```rust
    // For each task, the set of tasks it depends on
    dependencies: HashMap<TaskId, HashSet<TaskId>>,
    // For each task, the set of tasks that depend on it
    dependents: HashMap<TaskId, HashSet<TaskId>>,
    // For each task, the count of unresolved dependencies
    unresolved_deps: HashMap<TaskId, usize>,
    ```

#### **Task Spawning**

- When spawning a task with dependencies:
    1. Validate that all dependencies exist and are not self-referential.
    2. Check for cycles (see below).
    3. Record dependencies and dependents.
    4. If unresolved dependency count is zero, push to ready queue; otherwise, do not schedule yet.

    ```rust
    fn spawn_with_deps<F>(&mut self, deps: &[TaskId], f: F) -> TaskId
    where
        F: FnOnce(TaskContext) + Send + 'static,
    {
        let tid = self.next_id;
        self.next_id += 1;

        // Cycle detection
        if self.detect_cycle(tid, deps) {
            panic!("Cycle detected in task dependencies");
        }

        // Register dependencies
        let dep_set: HashSet<TaskId> = deps.iter().copied().collect();
        let unresolved = dep_set.len();
        self.dependencies.insert(tid, dep_set.clone());
        self.unresolved_deps.insert(tid, unresolved);

        // Register dependents
        for &dep in deps {
            self.dependents.entry(dep).or_default().insert(tid);
        }

        // Only schedule if no unresolved dependencies
        if unresolved == 0 {
            self.ready.push(...);
        }

        // Spawn coroutine as usual
        ...
        tid
    }
    ```

#### **Completion Handling**

- When a task completes:
    1. For each dependent, decrement its unresolved dependency count.
    2. If a dependent’s count reaches zero, push it to the ready queue.

    ```rust
    fn on_task_complete(&mut self, tid: TaskId) {
        if let Some(dependents) = self.dependents.remove(&tid) {
            for dep_tid in dependents {
                if let Some(count) = self.unresolved_deps.get_mut(&dep_tid) {
                    *count -= 1;
                    if *count == 0 {
                        self.ready.push(...); // Now ready to run
                    }
                }
            }
        }
    }
    ```

#### **Cycle Detection**

- Use DFS or Kahn’s algorithm to check for cycles when adding a new task:
    ```rust
    fn detect_cycle(&self, new_tid: TaskId, deps: &[TaskId]) -> bool {
        // Pseudocode: for each dep, walk its dependencies recursively
        // If new_tid is found in any path, a cycle exists
        ...
    }
    ```

#### **Ready Queue**

- Only tasks with zero unresolved dependencies are ever pushed to the ready queue.

#### **Cleanup**

- On task completion, remove its entries from `dependencies`, `dependents`, and `unresolved_deps`.

---

#### **Code Sketch: Scheduler Fields**

```rust
pub struct Scheduler {
    ...
    dependencies: HashMap<TaskId, HashSet<TaskId>>,
    dependents: HashMap<TaskId, HashSet<TaskId>>,
    unresolved_deps: HashMap<TaskId, usize>,
    ...
}
```

#### **Code Sketch: Spawning with Dependencies**

```rust
fn spawn_with_deps<F>(&mut self, deps: &[TaskId], f: F) -> TaskId
where
    F: FnOnce(TaskContext) + Send + 'static,
{
    // ... see above for details ...
}
```

#### **Code Sketch: Task Completion**

```rust
fn on_task_complete(&mut self, tid: TaskId) {
    // ... see above for details ...
}
```

#### **Test Example**

```rust
let a = sched.spawn(...); // No dependencies
let b = sched.spawn_with_deps(&[a], ...); // Depends on a
let c = sched.spawn_with_deps(&[a], ...); // Depends on a
let d = sched.spawn_with_deps(&[b, c], ...); // Depends on b and c
```

---

- **Cycle Detection Test:**
    ```rust
    let a = sched.spawn(...);
    let b = sched.spawn_with_deps(&[a], ...);
    // This should panic or error:
    let _ = sched.spawn_with_deps(&[b, a], ...); // Would create a cycle if a depends on b
    ```

- **Maximal Concurrency Test:**
    - Spawn several tasks with no dependencies and assert they all run in parallel (order does not matter).

---

- **Cleanup:**
    - On task completion, remove all references to the task from dependency tracking maps to avoid memory leaks.

---

### Testing

- Add/modify tests to:
    - Spawn tasks with multiple dependencies.
    - Assert correct execution order.
    - Assert that cycles are rejected.
    - Ensure maximal concurrency for independent tasks.

### Documentation

- Update README and crate docs to describe the new dependency model and APIs.
- Provide usage examples for common patterns (chains, trees, DAGs).

---

## Acceptance Criteria

- Tasks with dependencies only run after all dependencies complete.
- Cyclic dependencies are detected and rejected at spawn time.
- Existing tests pass; new tests cover DAG scenarios.
- Documentation is updated and clear.

---

## Out of Scope

- Dynamic dependency modification after spawn.
- Workflow-level constructs (e.g., conditional execution, retries).

---

## Risks & Mitigations

- **Risk:** Increased scheduler complexity.
    - **Mitigation:** Isolate DAG logic in a dedicated module; thorough testing.
- **Risk:** Performance overhead for large graphs.
    - **Mitigation:** Use efficient data structures (e.g., HashMap, HashSet).

---

## Open Questions

- Should dependencies be allowed to reference tasks that have not yet been spawned? (Initial version: No; all dependencies must be valid TaskIds at spawn time.)
- Should the API support named tasks for easier referencing? (Future enhancement.)

---

## Milestones

1. Design and implement dependency tracking structures.
2. Implement `spawn_with_deps` and update scheduler logic.
3. Add/modify tests for DAG execution.
4. Update documentation.
5. Code review and merge.
