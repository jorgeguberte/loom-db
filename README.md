# LoomDB
A strict-schema, bio-inspired graph memory engine for AI agents. Engineered in Rust. WASM-ready. Optimized for the Edge.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Status: Alpha](https://img.shields.io/badge/status-alpha-orange)

## The Philosophy
LoomDB is a specialized memory substrate designed for Cognitive Architectures. Unlike generic graph databases that prioritize flexibility, LoomDB prioritizes structure, predictability, and biological mimicry.

It is built to run anywhere: from high-performance servers to Edge environments and Browsers (via WebAssembly).

## Core Principles
1. Strict Schema Architecture: No generic "property bags". Nodes are strictly typed (Episode, Concept, State) at the compiler level to ensure zero-overhead access.

2. Bio-Inspired Mechanics: Implements the Ebbinghaus Forgetting Curve. Memories decay naturally over time unless reinforced, preventing context bloat.

3. Edge-Native Design: Designed to be lightweight and embeddable. The entire database state can be serialized and transferred between client and server effortlessly.

4. Stateful Context: Retrieval is not just semantic; it is temporal. A node's relevance is determined by its current activation level in the simulation.

## Features
- WASM Compatible: Core logic is pure Rust (agnostic of OS), making it perfect for running AI agents directly in the browser or Edge Functions.

- Lazy Activation Decay: Activation updates occur only upon access using timestamp deltas. O(1) cost for time progression.

- Asymptotic Boosting: Implements the Power Law of Learning. Strong memories are harder to boost, preventing overfitting/bias loops.

- Spread Activation (Ripple Effect): Weighted propagation of activation through associative edges, simulating "intuition" without explicit queries.

- Context-Aware Search: Inverted index that ranks results by a hybrid score of Text Match + Current Activation.

## Usage
**Installation**

Add this to your Cargo.toml:

```toml
[dependencies]
loom_db = { path = "." } 
# uuid and chrono features must be compatible with WASM if targeting web
uuid = { version = "1.10", features = ["v4", "serde", "js"] } 
chrono = { version = "0.4", features = ["serde", "wasmbind"] }
```


**Quick Start (Rust)**
```rust
use loom_db::{LoomGraph, Node, NodeMetadata, ConceptData, Edge};

fn main() {
    // 1. Initialize the Brain (Decay Rate: 0.95 per tick)
    let mut brain = LoomGraph::new(0.95);

    // 2. Create Concepts
    let n_rust = Node::Concept(NodeMetadata::new(), ConceptData { 
        name: "Rust".into(), 
        definition: "Systems Language".into() 
    });
    // ... add nodes ...

    // 3. Connect them
    brain.edges.push(Edge::Associated(id_rust, id_safety, 0.9));

    // 4. Time Passes (Memories fade...)
    for _ in 0..10 { brain.tick(); }

    // 5. Trigger "Spread Activation"
    // Stimulate "Rust". The energy will ripple to "Safety".
    brain.boost_node(0, 0.8, true); 

    // 6. Check Activation
    let safety_activation = brain.get_activation(1);
    // Output: High activation, simulating associative thought.
}
```


## Mechanics Explained
1. The Activation FormulaLoomDB uses a time-based decay formula inspired by biological synapses:$$A_{t} = A_{t-1} \times (DecayRate)^{\Delta t}$$Where $\Delta t$ is the number of "ticks" since the last access. This is calculated lazily.

2. Spread Activation (The "Ripple")When a node is boosted with propagate = true, energy flows to neighbors based on edge weight:$$Impact_{neighbor} = Boost_{source} \times Weight_{edge} \times DampingFactor$$This allows the system to surface relevant context ("Safety") without explicit queries.

----

## Roadmap
- [x] Core Engine (Nodes, Edges, Tick System)

- [x] Lazy Decay & Persistence

- [x] Spread Activation

- [ ] WASM Bindings: Expose LoomGraph to JavaScript/TypeScript.

- [ ] Dream Protocol: Pruning process to remove weak memories during "sleep".

- [ ] Context Builder: Output formatter for LLM System Prompts.

## License
MIT
