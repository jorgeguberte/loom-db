//! # LoomDB
//!
//! A strict-schema, bio-inspired graph memory engine for AI agents.
//! LoomDB is designed to mimic the human brain's memory mechanics, including:
//! - **Forgetting Curve**: Memories decay over time if not accessed.
//! - **Spread Activation**: Activation flows from one memory to related memories.
//! - **Strict Typing**: Distinguishes between Episodes, Concepts, and Emotional States.
//!
//! ## Example
//!
//! ```rust
//! use loom_db::{LoomGraph, Node, NodeMetadata, ConceptData, Edge};
//!
//! // 1. Initialize the Brain
//! let mut brain = LoomGraph::new(0.95);
//!
//! // 2. Add a concept
//! brain.add_concept("Rust".to_string(), "Systems Language".to_string());
//!
//! // 3. Time passes
//! brain.tick();
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

// ============================================================================
// 1. THE GRID (Schemas & Structs)
// ============================================================================

/// Metadata shared by all node types.
/// Tracks the "energy" and "strength" of a memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Unique identifier for the node.
    pub id: Uuid,
    /// Current activation level (0.0 to 1.0). High activation = "top of mind".
    pub activation: f32,
    /// Long-term stability factor. Higher stability = slower decay.
    /// Simulates Long-Term Potentiation (LTP).
    pub stability: f32,
    /// The last tick when this node was accessed or updated.
    pub last_tick: u64,
}

impl NodeMetadata {
    /// Creates a new metadata instance with full activation and default stability.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            activation: 1.0,
            stability: 1.0,
            last_tick: 0,
        }
    }
}

/// Data specific to an episodic memory (an event that happened).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeData {
    /// A text summary of the event.
    pub summary: String,
    /// When the event occurred (in real-world time).
    pub timestamp: DateTime<Utc>,
}

/// Data specific to a concept (semantic memory).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptData {
    /// The name of the concept (e.g., "Rust").
    pub name: String,
    /// The definition or description of the concept.
    pub definition: String,
}

/// Data specific to an emotional state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateData {
    /// Valence: Positive vs Negative (-1.0 to 1.0).
    pub valence: f32,
    /// Arousal: Calm vs Excited (0.0 to 1.0).
    pub arousal: f32,
}

/// The fundamental unit of memory in the graph.
/// Can be an Episode, a Concept, or a State.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    /// Represents a specific event in time.
    Episode(NodeMetadata, EpisodeData),
    /// Represents general knowledge or a definition.
    Concept(NodeMetadata, ConceptData),
    /// Represents an internal emotional state.
    State(NodeMetadata, StateData),
}

impl Node {
    /// Returns a reference to the node's metadata.
    pub fn meta(&self) -> &NodeMetadata {
        match self {
            Node::Episode(m, _) => m,
            Node::Concept(m, _) => m,
            Node::State(m, _) => m,
        }
    }

    /// Returns a mutable reference to the node's metadata.
    pub fn meta_mut(&mut self) -> &mut NodeMetadata {
        match self {
            Node::Episode(m, _) => m,
            Node::Concept(m, _) => m,
            Node::State(m, _) => m,
        }
    }

    /// Extracts text content for indexing purposes.
    /// Returns the summary for episodes, name + definition for concepts, and empty string for states.
    pub fn extract_text(&self) -> String {
        match self {
            Node::Episode(_, d) => d.summary.clone(),
            Node::Concept(_, d) => format!("{} {}", d.name, d.definition),
            Node::State(_, _) => "".to_string(), // Estados são sentimentos "mudos" (não indexáveis por texto)
        }
    }
}

/// Represents a connection between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Edge {
    /// Temporal sequence: Source happened before Target.
    Preceded(Uuid, Uuid),
    /// Reference: Source (Episode) mentioned Target (Concept).
    Mentioned(Uuid, Uuid),
    /// Causality: Source caused Target to be recalled.
    Evoked(Uuid, Uuid),
    /// Semantic Association: Source is related to Target with a specific weight.
    Associated(Uuid, Uuid, f32),
    /// Inhibition: Source suppresses Target with a specific weight.
    Inhibited(Uuid, Uuid, f32),
}

// ============================================================================
// 2. THE ENGINE (The Loom)
// ============================================================================

/// The main graph database structure.
/// Holds all nodes, edges, and state required for the memory simulation.
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct LoomGraph {
    /// List of all memory nodes.
    #[wasm_bindgen(skip)]
    pub nodes: Vec<Node>,
    /// List of all connections between nodes.
    #[wasm_bindgen(skip)]
    pub edges: Vec<Edge>,
    /// Global time counter for the simulation.
    #[wasm_bindgen(skip)]
    pub current_tick: u64,
    /// The base rate at which memory activation decays per tick (e.g., 0.95).
    #[wasm_bindgen(skip)]
    pub decay_rate: f32,
    /// Inverted index for text search (Word -> List of Node Indices).
    #[wasm_bindgen(skip)]
    pub index: HashMap<String, Vec<usize>>,
    /// Map of UUID to Node Index for O(1) lookup.
    #[wasm_bindgen(skip)]
    pub node_map: HashMap<Uuid, usize>,
    /// Timestamp of the last save/load operation, used for "wake up" calculations.
    #[wasm_bindgen(skip)]
    pub last_saved: Option<DateTime<Utc>>,
}

// ----------------------------------------------------------------------------
// API PÚBLICA (WASM/JS Friendly)
// ----------------------------------------------------------------------------
#[wasm_bindgen]
impl LoomGraph {
    /// Creates a new LoomGraph with a specified decay rate.
    ///
    /// # Arguments
    ///
    /// * `decay_rate` - The factor by which activation decays each tick (0.0 to 1.0).
    ///   Typical values are around 0.95.
    #[wasm_bindgen(constructor)]
    pub fn new(decay_rate: f32) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            current_tick: 0,
            decay_rate,
            index: HashMap::new(),
            node_map: HashMap::new(),
            last_saved: None,
        }
    }

    // --- 1. CONCEITO (Conhecimento Semântico) ---
    // JS: brain.add_concept("Rust", "Linguagem segura")
    /// Adds a new Concept node to the graph.
    /// Returns the UUID of the new node.
    #[wasm_bindgen]
    pub fn add_concept(&mut self, name: String, definition: String) -> String {
        let node = Node::Concept(NodeMetadata::new(), ConceptData { 
            name, 
            definition 
        });
        let id = node.meta().id.to_string();
        self.add_node(node); 
        id
    }

    // --- 2. EPISÓDIO (Memória Episódica / Evento) ---
    // JS: brain.add_episode("O usuário disse que gosta de pizza")
    // Nota: O Rust preenche o timestamp automaticamente com Utc::now()
    /// Adds a new Episode node to the graph.
    /// Automatically sets the timestamp to the current UTC time.
    /// Returns the UUID of the new node.
    #[wasm_bindgen]
    pub fn add_episode(&mut self, summary: String) -> String {
        let node = Node::Episode(NodeMetadata::new(), EpisodeData { 
            summary, 
            timestamp: Utc::now() 
        });
        let id = node.meta().id.to_string();
        self.add_node(node);
        id
    }

    // --- 3. ESTADO (Emoção / Sentimento) ---
    // JS: brain.add_state(0.8, 0.5)  -> (Valence=Positive, Arousal=Medium)
    /// Adds a new State node to the graph.
    /// Returns the UUID of the new node.
    #[wasm_bindgen]
    pub fn add_state(&mut self, valence: f32, arousal: f32) -> String {
        let node = Node::State(NodeMetadata::new(), StateData { 
            valence, 
            arousal 
        });
        let id = node.meta().id.to_string();
        self.add_node(node);
        id
    }

    // --- BUSCA & UTILITÁRIOS ---

    /// Searches for nodes containing the query text.
    /// Returns a JSON string representing the list of matching nodes, sorted by relevance.
    #[wasm_bindgen]
    pub fn search(&mut self, query: &str) -> String {
        let results = self.search_native(query); 
        serde_json::to_string(&results).unwrap_or("[]".to_string())
    }

    /// Generates a prompt context based on active memories.
    /// Only includes memories with activation higher than `min_activation`.
    #[wasm_bindgen]
    pub fn get_context(&mut self, min_activation: f32) -> String {
        self.get_context_prompt(min_activation)
    }

    /// Advances the simulation by one tick.
    /// This triggers decay calculations lazily when nodes are next accessed.
    #[wasm_bindgen]
    pub fn tick(&mut self) {
        self.current_tick += 1;
    }

    /// Simulates the passage of time based on real-world time passed since the last save.
    /// Updates the `current_tick` accordingly.
    #[wasm_bindgen]
    pub fn wake_up(&mut self) {
        if let Some(last_time) = self.last_saved {
            let now = Utc::now();
            let minutes_passed = (now - last_time).num_minutes();
            
            if minutes_passed > 0 {
                self.current_tick += minutes_passed as u64;
            }
        }
        self.last_saved = Some(Utc::now());
    }

    // --- PERSISTÊNCIA WEB (Import/Export Strings) ---

    /// Exports the entire graph state to a JSON string.
    #[wasm_bindgen]
    pub fn export_backup(&self) -> String {
        serde_json::to_string(&self).unwrap_or("{}".to_string())
    }

    /// Imports the graph state from a JSON string.
    /// Returns a new LoomGraph instance. If parsing fails, returns a new empty graph.
    #[wasm_bindgen]
    pub fn import_backup(json: &str) -> LoomGraph {
        serde_json::from_str(json).unwrap_or_else(|_| LoomGraph::new(0.95))
    }

    /// Retrieves detailed JSON info about a node by its internal index.
    #[wasm_bindgen]
    pub fn get_node_info(&self, index: usize) -> String {
        // Tenta pegar o nó pelo índice numérico
        if let Some(node) = self.nodes.get(index) {
            // O serde_json faz a mágica de converter Enum (Episode/Concept/State) para JSON
            serde_json::to_string(node).unwrap_or("{}".to_string())
        } else {
            "{}".to_string() // Retorna objeto vazio se o ID não existir
        }
    }

    // JS chama: connect("uuid_string_a", "uuid_string_b", 0.9)
    /// Creates an association between two nodes identified by their UUID strings.
    /// Returns true if successful, false if IDs are invalid or not found.
    #[wasm_bindgen]
    pub fn connect(&mut self, source_id: &str, target_id: &str, weight: f32) -> bool {
        // Tenta converter as Strings recebidas para UUIDs reais
        let source_uuid = match Uuid::parse_str(source_id) {
            Ok(id) => id,
            Err(_) => return false, // ID inválido
        };

        let target_uuid = match Uuid::parse_str(target_id) {
            Ok(id) => id,
            Err(_) => return false,
        };

        // Verifica se os UUIDs existem no mapa
        if self.node_map.contains_key(&source_uuid) && self.node_map.contains_key(&target_uuid) {
            self.edges.push(Edge::Associated(source_uuid, target_uuid, weight));
            return true;
        }
        
        false
    }
    

    // Vamos expor o Boost também para testarmos a reação em cadeia manualmente
    // JS chama: stimulate("uuid_string", 1.0)
    /// Stimulates (boosts) a node by its UUID string.
    /// This triggers spread activation to connected nodes.
    /// Returns true if successful.
    #[wasm_bindgen]
    pub fn stimulate(&mut self, id_str: &str, force: f32) -> bool {
        if let Ok(uuid) = Uuid::parse_str(id_str) {
            if let Some(&idx) = self.node_map.get(&uuid) {
                self.boost_node(idx, force, true);
                return true;
            }
        }
        false
    }

    /// Removes nodes that have both low stability and low activation.
    /// Returns the number of nodes removed.
    #[wasm_bindgen]
    pub fn prune_low_stability(&mut self, threshold: f32) -> usize {
        let before = self.nodes.len();
        
        // Find nodes that are both low stability and low activation (safe to prune)
        let to_remove: Vec<Uuid> = self.nodes.iter()
            .filter(|n| n.meta().stability < threshold && n.meta().activation < 0.1)
            .map(|n| n.meta().id)
            .collect();

        if to_remove.is_empty() { return 0; }

        for id in to_remove {
            self.nodes.retain(|n| n.meta().id != id);
            self.edges.retain(|e| {
                match e {
                    Edge::Preceded(s, t) | Edge::Mentioned(s, t) | Edge::Evoked(s, t) | Edge::Associated(s, t, _) | Edge::Inhibited(s, t, _) => *s != id && *t != id
                }
            });
        }

        self.rebuild_node_map();
        let after = self.nodes.len();
        before - after
    }

    fn rebuild_node_map(&mut self) {
        self.node_map.clear();
        for (idx, node) in self.nodes.iter().enumerate() {
            self.node_map.insert(node.meta().id, idx);
        }
    }
}

// ----------------------------------------------------------------------------
// API INTERNA (Rust Only) - O "Motor" Real
// ----------------------------------------------------------------------------
impl LoomGraph {
    /// Internal: Adds a node and indexes its text.
    /// This is the low-level method used by `add_concept`, `add_episode`, etc.
    pub fn add_node(&mut self, node: Node) {
        let idx = self.nodes.len();
        let id = node.meta().id;
        
        // Só indexamos texto se houver texto (Estados são ignorados aqui)
        let text = node.extract_text().to_lowercase();
        if !text.is_empty() {
            let tokens: Vec<&str> = text.split_whitespace().collect();
            for token in tokens {
                let clean = token.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
                if !clean.is_empty() {
                    self.index.entry(clean).or_insert(Vec::new()).push(idx);
                }
            }
        }

        self.node_map.insert(id, idx);
        let mut n = node;
        n.meta_mut().last_tick = self.current_tick;
        self.nodes.push(n);
    }

    /// Internal: Native Rust search implementation.
    /// Returns a vector of (Node Index, Activation Score).
    pub fn search_native(&mut self, query: &str) -> Vec<(usize, f32)> {
        let clean_query = query.to_lowercase();
        let clean_query = clean_query.trim(); // Removido o shadowing desnecessário na mesma linha

        if clean_query.is_empty() {
            return Vec::new();
        }

        // --- FASE 1: COLETA (Immutable Borrow) ---
        // Aqui só lemos. Não chamamos nada que altera o self.
        let mut candidate_indices = Vec::new();
        
        for (key, indices) in &self.index {
            if key.contains(clean_query) {
                // Copiamos os IDs para a nossa lista temporária
                candidate_indices.extend(indices);
            }
        }
        // ✨ AQUI O BORROW IMUTÁVEL MORRE ✨
        // Como o loop acabou, o Rust solta o 'self.index'.

        // Limpeza: Removemos duplicatas para não processar o mesmo nó duas vezes
        // (Ex: se buscou "t" e achou "Rust" e "Test", o mesmo ID pode aparecer 2x)
        candidate_indices.sort_unstable();
        candidate_indices.dedup();

        // --- FASE 2: CÁLCULO (Mutable Borrow) ---
        // Agora estamos livres para chamar métodos que alteram o self (get_activation)
        let mut results = Vec::new();
        
        for idx in candidate_indices {
            let activation = self.get_activation(idx);
            results.push((idx, activation));
        }

        // Ordenação final
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        results
    }

    /// Calculates the current activation of a node, applying decay if necessary.
    /// This method is lazy: it updates the node's state only when accessed.
    pub fn get_activation(&mut self, node_idx: usize) -> f32 {
        let tick_now = self.current_tick;
        let decay = self.decay_rate;
        let node = &mut self.nodes[node_idx];
        let meta = node.meta_mut();

        if meta.last_tick < tick_now {
            let delta_t = (tick_now - meta.last_tick) as i32;
            // Stability acts as a decay dampener. 
            // Real decay = decay^(delta_t / stability)
            let effective_decay = decay.powf(delta_t as f32 / meta.stability);
            meta.activation *= effective_decay;
            meta.last_tick = tick_now;
        }
        meta.activation
    }

    /// Boosts a node's activation and optionally spreads energy to connected nodes.
    /// Implements the "Spread Activation" and "Long-Term Potentiation" mechanics.
    pub fn boost_node(&mut self, node_idx: usize, amount: f32, propagate: bool) {
        let current_activation = self.get_activation(node_idx);
        let meta = self.nodes[node_idx].meta_mut();
        
        // Asymptotic boost
        let real_boost = (1.0 - current_activation) * amount;
        meta.activation += real_boost;
        
        // LTP: Boosting helps stabilize the memory
        // Stability grows asymptotically towards a high cap (e.g., 50.0)
        let stability_gain = amount * 0.1; 
        meta.stability += (50.0 - meta.stability) * stability_gain;

        meta.last_tick = self.current_tick;
        
        if propagate {
            let node_id = meta.id;
            // Capture nodes and mapping to avoid borrow conflicts during recursion if we were using indices
            // But we use UUIDs + loop over cloned edges which is safe but slightly slow. 
            // For WASM scale it's fine.
            let edges_snapshot = self.edges.clone(); 
            for edge in edges_snapshot {
                if let Edge::Associated(source, target, weight) = edge {
                    if source == node_id {
                        if let Some(&neighbor_idx) = self.node_map.get(&target) {
                            // Ripple effect is proportional to boost and edge weight
                            // Weight can now be negative (Inhibition)
                            let ripple_effect = amount * weight * 0.5;
                            
                            if ripple_effect > 0.0 {
                                self.boost_node(neighbor_idx, ripple_effect, false);
                            } else if ripple_effect < 0.0 {
                                // Suppression logic
                                let updated_node = &mut self.nodes[neighbor_idx];
                                let n_meta = updated_node.meta_mut();
                                
                                // Reduce activation by the negative ripple
                                n_meta.activation = (n_meta.activation + ripple_effect).max(0.0);
                                n_meta.last_tick = self.current_tick;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Generates an XML string representing the current context of active memories.
    /// This is useful for passing memory context to LLMs.
    pub fn get_context_prompt(&mut self, min_activation: f32) -> String {
        let mut buffer = String::new();
        buffer.push_str("<active_memories>\n");
        
        let mut active_indices: Vec<usize> = (0..self.nodes.len())
            .filter(|&idx| self.get_activation(idx) > min_activation)
            .collect();

        active_indices.sort_by(|&a, &b| {
            let val_a = self.nodes[a].meta().activation;
            let val_b = self.nodes[b].meta().activation;
            val_b.partial_cmp(&val_a).unwrap()
        });

        if active_indices.is_empty() {
            buffer.push_str("  <memory>No relevant active memories.</memory>\n");
        } else {
            for idx in active_indices {
                let node = &self.nodes[idx];
                let meta = node.meta();
                match node {
                    Node::Concept(_, data) => {
                        buffer.push_str(&format!(
                            "  <memory type='concept' activation='{:.2}' stability='{:.2}'>\n    <name>{}</name>\n    <definition>{}</definition>\n  </memory>\n",
                            meta.activation, meta.stability, data.name, data.definition
                        ));
                    },
                    Node::Episode(_, data) => {
                        buffer.push_str(&format!(
                            "  <memory type='episode' activation='{:.2}' stability='{:.2}' time='{}'>\n    <summary>{}</summary>\n  </memory>\n",
                            meta.activation, meta.stability, data.timestamp.to_rfc3339(), data.summary
                        ));
                    },
                    Node::State(_, data) => {
                        buffer.push_str(&format!(
                            "  <state activation='{:.2}' stability='{:.2}'>\n    <mood valence='{:.2}' arousal='{:.2}' />\n  </state>\n",
                            meta.activation, meta.stability, data.valence, data.arousal
                        ));
                    }
                }
            }
        }
        buffer.push_str("</active_memories>");
        buffer
    }


    // Persistência CLI (Desktop)
    /// Saves the current state of the LoomGraph to a JSON file.
    pub fn save_to_file(&mut self, filepath: &str) -> std::io::Result<()> {
        self.last_saved = Some(Utc::now());
        let file = File::create(filepath)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)?;
        Ok(())
    }

    /// Loads the LoomGraph state from a JSON file.
    pub fn load_from_file(filepath: &str) -> std::io::Result<Self> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let brain = serde_json::from_reader(reader)?;
        Ok(brain)
    }

    


}