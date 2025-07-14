---

To improve **ADR-0008: Persistent Agent Memory Strategy**, we should clearly address the critical findings from the deep dive research, specifically:

### Enhancements Based on Research:

* **Memory Types & Differentiation**:
  Clearly differentiate **semantic**, **episodic**, and **procedural** memory in the ADR, outlining separate storage and retrieval mechanisms.

* **Temporal Drift Management**:
  Incorporate strategies to handle temporal drift and evolving memory states (e.g., versioned semantic memory, memory chaining).

* **Memory Versioning and Consistency**:
  Outline a structured approach for version-aware semantic memory, including methods for retrieving current vs historical memory states.

* **Agent Memory Limitations**:
  Address known limitations from current systems (e.g., LangChain, Claude, Gemini), explicitly highlighting how Tiffany addresses them.

* **Semantic Graph & Vector Integration**:
  Recommend combining graph-based structures with embedding vectors to enable semantic similarity searches and flexible memory retrieval.

### Recommended ADR Improvements:

The ADR should include new sections or extended details covering:

1. **Detailed Breakdown of Memory Types**:

    * Semantic: facts and evolving knowledge, stored in Neo4j or similar graph DB.
    * Episodic: event-based logs, leveraging the WAL structure.
    * Procedural: structured actions or skills, potentially encoded as Rust traits or callable actions.

2. **Temporal Versioning Strategy**:

    * Incorporate memory entries with timestamps and semantic versioning.
    * Propose use of a temporal query mechanism to retrieve memory states at specific points in time.

3. **Handling Evolving Preferences & Context**:

    * Explicitly describe how the agent would handle shifts like directory structure changes, using semantic versioning or decision lineage.
    * Suggest using decision-chains (similar to vector pathing) for explaining why a current decision differs from historical actions.

4. **Integration with Vector Embeddings**:

    * Recommend combining Neo4j with vector embedding stores (e.g., using Pinecone or local vector DB) for richer semantic querying and retrieval.

5. **Limitations & Comparative Analysis**:

    * Include a short comparative matrix highlighting issues in systems like LangChain, Claude, and Gemini and how this ADR’s design overcomes them.

If you agree, I can proceed to edit the ADR accordingly.
