# LoomDB
A strict-schema, bio-inspired graph memory engine for AI agents. Engineered in Rust. WASM-ready. Optimized for the Edge.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Status: Alpha](https://img.shields.io/badge/status-alpha-orange)

## The Philosophy
LoomDB is a specialized memory substrate designed for Cognitive Architectures. Unlike generic graph databases that prioritize flexibility, LoomDB prioritizes structure, predictability, and biological mimicry.

It is built to run anywhere: from high-performance servers to Edge environments and Browsers (via WebAssembly).

## Core Principles
1. **Strict Schema Architecture**: No generic "property bags". Nodes are strictly typed (`Episode`, `Concept`, `State`) at the compiler level to ensure zero-overhead access.

2. **Bio-Inspired Mechanics**: Implements the Ebbinghaus Forgetting Curve. Memories decay naturally over time unless reinforced, preventing context bloat.

3. **Edge-Native Design**: Designed to be lightweight and embeddable. The entire database state can be serialized and transferred between client and server effortlessly.

4. **Stateful Context**: Retrieval is not just semantic; it is temporal. A node's relevance is determined by its current activation level in the simulation.

## Features
- **WASM Compatible**: Core logic is pure Rust (agnostic of OS), making it perfect for running AI agents directly in the browser or Edge Functions.
- **Lazy Activation Decay**: Activation updates occur only upon access using timestamp deltas. O(1) cost for time progression.
- **Asymptotic Boosting**: Implements the Power Law of Learning. Strong memories are harder to boost, preventing overfitting/bias loops.
- **Spread Activation (Ripple Effect)**: Weighted propagation of activation through associative edges, simulating "intuition" without explicit queries.
- **Context-Aware Search**: Inverted index that ranks results by a hybrid score of Text Match + Current Activation (Projected).
- **Dream Protocol**: A consolidation mechanism (LTP) that reinforces stable memories and prunes weak ones (synaptic pruning).

## Usage
**Installation**

Add this to your `Cargo.toml`:

```toml
[dependencies]
loom_db = { path = "." } 
# Ensure features are compatible with WASM if targeting web
uuid = { version = "1.10", features = ["v4", "serde", "js"] } 
chrono = { version = "0.4", features = ["serde", "wasmbind"] }
```

**Quick Start (Rust)**

```rust
use loom_db::LoomGraph;

fn main() {
    // 1. Initialize the Brain (Decay Rate: 0.90 per tick)
    let mut brain = LoomGraph::new(0.90);

    // 2. Create Concepts (Returns UUID string)
    let rust_id = brain.add_concept(
        "Rust".into(),
        "Systems Language".into()
    );

    // 3. Time Passes (Memories fade...)
    // Simulate 5 ticks of time passing
    for _ in 0..5 { brain.tick(); }

    // 4. Check Activation
    // Search projects the current activation based on time elapsed
    let results = brain.search_native("Rust");
    if let Some((_, activation)) = results.first() {
        println!("Rust Activation: {:.4}", activation);
        // Output: ~0.59 (0.90^5)
    }

    // 5. Reinforce Memory
    // Stimulate the node to boost its activation and stability
    brain.stimulate(&rust_id, 0.5);
}
```

## API Overview

### Ingestion
- `add_concept(name, definition)`: Adds a semantic concept.
- `add_episode(summary)`: Adds an episodic memory with a timestamp.
- `add_state(valence, arousal)`: Adds an emotional state node.

### Topology
- `connect(source_id, target_id, weight)`: Creates a directed edge between nodes.
- `stimulate(id, force)`: Boosts a node's activation and triggers the Ripple Effect (spread activation) to neighbors.

### Retrieval & Maintenance
- `search(query)`: Returns JSON results ranked by relevance (Semantic + Temporal).
- `get_context(min_activation)`: Generates an XML prompt context of active memories for LLMs.
- `dream()`: Runs the consolidation cycle. Promotes high-activation nodes to higher stability (Long Term Potentiation) and decays/prunes others.
- `wake_up()`: Syncs the internal tick counter with real-world time (if persisted).

## Mechanics Explained
1. **The Activation Formula**: LoomDB uses a time-based decay formula inspired by biological synapses:
   $$A_{t} = A_{t-1} \times (DecayRate)^{\Delta t}$$
   Where $\Delta t$ is the number of "ticks" since the last access. This is calculated lazily.

2. **Spread Activation (The "Ripple")**: When a node is boosted, energy flows to neighbors based on edge weight:
   $$Impact_{neighbor} = Boost_{source} \times Weight_{edge} \times DampingFactor$$
   This allows the system to surface relevant context without explicit queries.

3. **Dream Protocol**: During "sleep" (the `dream()` call), the system performs:
    - **Consolidation**: Memories with high activation gain stability.
    - **Washout**: Activation is dampened to prepare for the next cycle (simulating Adenosine clearance).
    - **Pruning**: Low stability, low activation nodes are permanently removed.

## License
MIT
