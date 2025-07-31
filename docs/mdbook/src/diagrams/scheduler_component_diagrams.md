### Diagram D1 – Basic Task Switching (ReadyQueue only)

```mermaid
sequenceDiagram
    participant Scheduler
    participant ReadyQueue as FIFO
    actor TaskA
    actor TaskB

    Scheduler->>ReadyQueue: dequeue TaskA
    ReadyQueue-->>Scheduler: TaskA
    Scheduler->>TaskA: resume()
    TaskA-->>Scheduler: yield (None)
    Scheduler->>ReadyQueue: enqueue(TaskA)

    Scheduler->>ReadyQueue: dequeue TaskB
    ReadyQueue-->>Scheduler: TaskB
    Scheduler->>TaskB: resume()
    TaskB-->>Scheduler: yield (None)
    Scheduler->>ReadyQueue: enqueue(TaskB)
```


### Diagram D2 – Nested Coroutines (Trampolining)

```mermaid
sequenceDiagram
    participant CallStack as CallStack_LIFO
    actor TaskMain
    actor SubCoro
    
    TaskMain-->>CallStack: push(TaskMain.gen)
    TaskMain->>SubCoro: start sub-coroutine
    SubCoro-->>CallStack: yield value
    CallStack->>TaskMain: pop parent
    TaskMain->>TaskMain: resume with value
```

### Diagram D3 – I/O Blocking and Wakeup

```mermaid
sequenceDiagram
    participant Scheduler
    participant WaitMap as EventMap
    participant ReadyQueue
    actor TaskIO
    
    TaskIO-->>Scheduler: yield ReadWait(fd=7)
    Scheduler->>WaitMap: record TaskIO blocked on fd=7
    
    Note right of WaitMap: fd=7 becomes readable
    WaitMap->>Scheduler: notify fd=7 ready
    Scheduler->>ReadyQueue: enqueue(TaskIO)
    ReadyQueue-->>Scheduler: TaskIO ready to run
```


### Diagram D4 – Sleep Timer
```mermaid
sequenceDiagram
    participant Scheduler
    participant WaitMap as SleepMap
    participant ReadyQueue
    actor TaskSleep
    
    TaskSleep-->>Scheduler: yield Sleep(3s)
    Scheduler->>WaitMap: record TaskSleep with wake_time = now + 3s
    
    Note right of WaitMap: 3 seconds pass
    WaitMap->>Scheduler: wake TaskSleep
    Scheduler->>ReadyQueue: enqueue(TaskSleep)
```

### Diagram D5 – Join / WaitTask
```mermaid
sequenceDiagram
    participant Scheduler
    participant WaitMap as JoinMap
    actor TaskA as Task_Parent
    actor TaskB as Task_Child
    
    TaskA-->>Scheduler: yield NewTask(TaskB)
    Scheduler->>TaskA: resume with TaskB.tid
    
    TaskA-->>Scheduler: yield WaitTask(TaskB)
    Scheduler->>WaitMap: record TaskA waiting on TaskB
    
    Note right of TaskB: TaskB runs and completes
    TaskB-->>Scheduler: StopIteration
    Scheduler->>WaitMap: notify TaskA to resume
```


### ✅ Agent Task Graph (Per Agent / DAG View)

Type: graph TD (or flowchart TD)
Purpose: Visualize how one FAR agent's current task decomposes into subtasks, dependencies, joins, etc.

```mermaid
graph TD
    RootTask[Plan Story Code]
    Sub1[Setup Repo]
    Sub2[Create main.rs]
    Sub3[Implement main_fn]
    Sub4[Add Tests]
    Join[WaitTask: Finalize]

    RootTask --> Sub1
    RootTask --> Sub2
    Sub2 --> Sub3
    Sub3 --> Sub4
    Sub1 --> Join
    Sub4 --> Join

```

### ✅ FAR Cluster View: Distributed Agents & Scheduler

Type: graph LR or flowchart LR
Purpose: Show how agents across blades interact with a central orchestrator + local schedulers

```mermaid
flowchart LR
    Orchestrator[[Central Scheduler]]
    subgraph Blade A
        A1[Agent 001\n - builder]
        SchedA[Local Scheduler]
    end
    subgraph Blade B
        A2[Agent 002\n - Tester]
        SchedB[Local Scheduler]
    end

    Orchestrator --> SchedA --> A1
    Orchestrator --> SchedB --> A2

```

### ✅ Skill Invocation Trace (ReAct-aware timeline)

Purpose: Show how a ReAct loop invokes skills with context, yields, retries, and memory access

```mermaid
sequenceDiagram
    actor ReActLoop
    participant SkillManager
    participant SkillEdit
    participant MemoryStore
    participant WAL

    ReActLoop->>SkillManager: invoke("Edit main.rs")
    SkillManager->>SkillEdit: run()
    SkillEdit->>MemoryStore: lookup(path="main.rs")
    MemoryStore-->>SkillEdit: file AST
    SkillEdit-->>WAL: write(PatchGenerated)
    SkillEdit-->>SkillManager: return Patch
    SkillManager-->>ReActLoop: Patch ready

```

✅ 11. Agent Memory Structure (Graph + Vector)

Purpose: Show semantic and episodic memory layers, link to disk/durable store

```mermaid
graph TD
  MemRoot[Memory: Agent]
  Sem[Semantic Graph]
  Episodic[Task Log / WAL]
  Vectors[VectorStore - code, plans]
GraphDB[(Neo4j)]
VecDB[(Qdrant/Faiss)]

MemRoot --> Sem --> GraphDB
MemRoot --> Episodic --> WAL
MemRoot --> Vectors --> VecDB
```

✅ 12. Coroutines Timeline (Concurrent Task Steps)

Purpose: Show wall-clock execution of several tasks with yield/resume gaps

```mermaid
gantt
    title Task Concurrency Timeline
    dateFormat  HH:mm:ss
    section Task_Main
    run      :active, 00:00:00, 5s
    sleep    : 00:00:05, 3s
    resume   : 00:00:08, 4s
    section Task_Sub
    start    :active, 00:00:02, 3s
    block_io : 00:00:05, 4s
    finish   : 00:00:09, 2s

```


✅ System Call Map

Purpose: Visualize all supported system calls and their dispatch targets (WAL, IO, Scheduler)

```mermaid
graph LR
    YieldNewTask --> Scheduler
    YieldReadWait --> IOSelector
    YieldSleep --> TimerWheel
    YieldGetTid --> Scheduler
    YieldMemoryQuery --> MemoryStore
    YieldLog --> PAL

```

✅ Task WAL Replayer Flow

Purpose: Shows how the replay engine reconstructs a task from the WAL stream, rehydrating state and optionally stepping through each yield point.

```mermaid
flowchart TD
    WAL[WAL Stream: task_187.log] --> Parser
    Parser --> EventQueue
    EventQueue --> Scheduler
    Scheduler --> TaskReplayer
    TaskReplayer --> CallStack
    TaskReplayer --> PAL[Replay Mode: emit PAL ghost events]
    TaskReplayer --> Memory[Optional Memory Patching]

    style WAL fill:#dff,stroke:#0af
    style PAL fill:#eee,stroke:#f08
    style Memory fill:#f9f,stroke:#808

```

✅ Panic Propagation via Wait/Join Graph

Purpose: If a child task panics, show how the error might propagate to parent or dependents if not isolated.

```mermaid
graph TD
    A[Task_Main\n - tid 101]
    B[Task_Git\n - tid 202]
    C[Task_Edit\n - tid 203]
    D[Task_Tests\n - tid 204]

    A -->|JoinTask| B
    A -->|JoinTask| C
    C -->|JoinTask| D

    D -.->|panic| C
    C -.->|propagate panic| A

    style D fill:#faa,stroke:#800
    style C fill:#ffe,stroke:#aa0

```

✅ Bonus 3: Skill Invocation Heatmap

Purpose: Visualize which system tools/skills are being used most frequently, based on PAL telemetry stream.

```mermaid
graph LR
    Skill_Edit["📝 Edit File"]:::hot
    Skill_Grep["🔍 Grep"]:::warm
    Skill_Test["✅ Run Tests"]:::hot
    Skill_Git["🌿 Git Ops"]:::warm
    Skill_Plan["🧠 Plan/Refactor"]:::cold
    Skill_Fmt["🧹 Format Code"]:::cold

    style Skill_Edit fill:#f88,stroke:#800
    style Skill_Test fill:#f88,stroke:#800
    style Skill_Grep fill:#fb6,stroke:#aa0
    style Skill_Git fill:#fb6,stroke:#aa0
    style Skill_Plan fill:#ccf,stroke:#44f
    style Skill_Fmt fill:#ccf,stroke:#44f

```

