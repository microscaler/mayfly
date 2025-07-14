## 📄 Tiffany: Temporal Semantic Memory

---

### ❓ Why Now?

Modern agentic systems (Claude, Gemini, LangChain) fail to manage evolving context and long-term decision history. They suffer from:

* **Forgetfulness** — LLMs lose memory of past decisions
* **Semantic drift** — agents repeat outdated behaviors even after correction
* **Overwrites** — memory updates destroy context instead of preserving lineage

The result: agents misplace files, rewrite deleted components, or resurrect deprecated code weeks later.

---

### ❌ Before Tiffany

Without temporal semantic memory, agents behave like stateless optimizers with short-term recall:

| Problem                | Consequence                                               |
| ---------------------- | --------------------------------------------------------- |
| No fact lineage        | Decisions appear out of nowhere, difficult to trace       |
| Flat embeddings only   | “Vector soup” causes incoherent or misprioritized recalls |
| No time context        | Recent changes are indistinguishable from stale ones      |
| No memory of evolution | Agents forget refactor plans, naming conventions, etc.    |

Even when documents are RAG'ed from disk, there is no reliable sense of what the **current truth** is or how that truth changed.

---

### ✅ What Tiffany Introduces

Tiffany encodes memory into a **temporal, versioned semantic graph**, enriched with embeddings for similarity queries. We introduce:

* 🧠 **Semantic Memory Graph** — Nodes for `File`, `Function`, `Fact`, `AgentPlan`, `Patch`
* 🧭 **Temporal Versioning** — `(:Fact)-[:UPDATED_BY]->(:Fact)` chains for all symbols
* 🧬 **Vector Snapshots** — `VecStore` stores embeddings per versioned semantic node
* 📅 **Active Timeline View** — Tiffany always knows: what is now, what used to be, and why it changed

---

### 🛠️ How It Works

At runtime:

1. **Ingest**: All canvas commits, LLM plans, and tool actions are processed into a memory graph.
2. **Index**: Semantically meaningful nodes (structs, functions, locations) are embedded and indexed.
3. **Version**: Any update produces a new version node with `UPDATED_BY` and `VALID_FROM` timestamps.
4. **Query**: During plan creation, agents retrieve:

    * The current live version
    * Past decisions affecting it
    * Nearby semantic neighbors (graph)
    * Similar embeddings (vector)

---

### 🔁 Real Example

You once told the agent:

> "We now keep tests under `crates/*/tests/`, not in `/tests/`."

In other systems:

* This info is overwritten or ignored in new sessions.
* Agent repeats the error again, forgetting you ever fixed it.

In Tiffany:

* A `MemoryFact` about `test_directory_convention` is updated.
* The old version is retained, the new is marked active.
* Any new planning task involving test file creation:

    * Finds the new fact
    * Sees the reason
    * Ranks it higher due to recency and lineage

---

### 🔮 After Tiffany: New Possibilities

| Feature                       | Enabled by Temporal Memory                      |
| ----------------------------- | ----------------------------------------------- |
| Stable multi-agent cognition  | Shared memory timelines with ownership tracking |
| Error root cause analysis     | Trace semantic changes to exact moments         |
| Better ReAct loop performance | Replans adapt to known past plans               |
| Real refactor memory          | Remember every rename, move, and semantic merge |

---

### 📦 Implementation Modules

* `graphdb` – Semantic memory graph storage (Neo4j or embedded)
* `vector_store` – Vector memory aligned to graph nodes
* `memory/indexer` – Watches canvas + WAL to generate memory updates
* `reasonact` – Consumes memory during planning

