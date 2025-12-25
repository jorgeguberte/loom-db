//! # LoomDB Core 2.1 (Gold Master)
//! Graph Memory Engine with Bio-Mimetic Mechanics & O(1) Architecture.
//!
//! Features:
//! - HashMap-based Storage (UUID Stability)
//! - Adjacency List Topology
//! - Lazy Decay with Projected Search Ranking
//! - Recursive Ripple Effect (Optimized)
//! - Dream Protocol (LTP Consolidation)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use uuid::Uuid;
use wasm_bindgen::prelude::*;

// ============================================================================
// 1. ESTRUTURAS DE DADOS (Topology & Storage)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub target: Uuid,
    pub weight: f32,
    pub edge_type: String, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub id: Uuid,
    pub activation: f32,
    pub stability: f32,
    pub last_tick: u64,
}

impl NodeMetadata {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            activation: 1.0,
            stability: 1.0,
            last_tick: 0,
        }
    }
}

// -- Tipos de Dados dos Nós --
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeData {
    pub summary: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptData {
    pub name: String,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateData {
    pub valence: f32,
    pub arousal: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    Episode(NodeMetadata, EpisodeData),
    Concept(NodeMetadata, ConceptData),
    State(NodeMetadata, StateData),
}

impl Node {
    pub fn meta(&self) -> &NodeMetadata {
        match self {
            Node::Episode(m, _) => m,
            Node::Concept(m, _) => m,
            Node::State(m, _) => m,
        }
    }

    pub fn meta_mut(&mut self) -> &mut NodeMetadata {
        match self {
            Node::Episode(m, _) => m,
            Node::Concept(m, _) => m,
            Node::State(m, _) => m,
        }
    }

    pub fn extract_text(&self) -> String {
        match self {
            Node::Episode(_, d) => d.summary.clone(),
            Node::Concept(_, d) => format!("{} {}", d.name, d.definition),
            Node::State(_, _) => "".to_string(),
        }
    }
}

// ============================================================================
// 2. O MOTOR (LoomGraph)
// ============================================================================

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct LoomGraph {
    // Storage Primário: O(1) Access
    #[wasm_bindgen(skip)]
    pub nodes: HashMap<Uuid, Node>,

    // Topologia: O(1) Neighbor Lookup
    #[wasm_bindgen(skip)]
    pub adjacency: HashMap<Uuid, Vec<Connection>>,

    // Índice de Busca
    #[wasm_bindgen(skip)]
    pub index: HashMap<String, Vec<Uuid>>,

    #[wasm_bindgen(skip)]
    pub current_tick: u64,
    #[wasm_bindgen(skip)]
    pub decay_rate: f32,
    #[wasm_bindgen(skip)]
    pub last_saved: Option<DateTime<Utc>>,
}

// ----------------------------------------------------------------------------
// API PÚBLICA (WASM)
// ----------------------------------------------------------------------------
#[wasm_bindgen]
impl LoomGraph {
    #[wasm_bindgen(constructor)]
    pub fn new(decay_rate: f32) -> Self {
        Self {
            nodes: HashMap::new(),
            adjacency: HashMap::new(),
            index: HashMap::new(),
            current_tick: 0,
            decay_rate,
            last_saved: None,
        }
    }

    // --- INGESTÃO DE DADOS ---

    #[wasm_bindgen]
    pub fn add_concept(&mut self, name: String, definition: String) -> String {
        let node = Node::Concept(NodeMetadata::new(), ConceptData { name, definition });
        let id = node.meta().id.to_string();
        self.add_node_internal(node);
        id
    }

    #[wasm_bindgen]
    pub fn add_episode(&mut self, summary: String) -> String {
        let node = Node::Episode(NodeMetadata::new(), EpisodeData { 
            summary, 
            timestamp: Utc::now() 
        });
        let id = node.meta().id.to_string();
        self.add_node_internal(node);
        id
    }

    #[wasm_bindgen]
    pub fn add_state(&mut self, valence: f32, arousal: f32) -> String {
        let node = Node::State(NodeMetadata::new(), StateData { valence, arousal });
        let id = node.meta().id.to_string();
        self.add_node_internal(node);
        id
    }

    // --- CONEXÕES ---

    #[wasm_bindgen]
    pub fn connect(&mut self, source_id: &str, target_id: &str, weight: f32) -> bool {
        let s_uuid = match Uuid::parse_str(source_id) { Ok(u) => u, Err(_) => return false };
        let t_uuid = match Uuid::parse_str(target_id) { Ok(u) => u, Err(_) => return false };

        if self.nodes.contains_key(&s_uuid) && self.nodes.contains_key(&t_uuid) {
            self.adjacency.entry(s_uuid).or_insert(Vec::new()).push(Connection {
                target: t_uuid,
                weight,
                edge_type: "Associated".to_string(),
            });
            return true;
        }
        false
    }

    // --- BUSCA & RECUPERAÇÃO ---

    #[wasm_bindgen]
    pub fn search(&mut self, query: &str) -> String {
        let results = self.search_native(query);
        serde_json::to_string(&results).unwrap_or("[]".to_string())
    }

    #[wasm_bindgen]
    pub fn get_node_info(&self, id_str: &str) -> String {
        if let Ok(uuid) = Uuid::parse_str(id_str) {
            if let Some(node) = self.nodes.get(&uuid) {
                return serde_json::to_string(node).unwrap_or("{}".to_string());
            }
        }
        "{}" .to_string()
    }

    // --- SIMULAÇÃO & TEMPO ---

    #[wasm_bindgen]
    pub fn tick(&mut self) {
        self.current_tick += 1;
    }

    #[wasm_bindgen]
    pub fn stimulate(&mut self, id_str: &str, force: f32) -> bool {
        if let Ok(uuid) = Uuid::parse_str(id_str) {
            if self.nodes.contains_key(&uuid) {
                self.boost_node(uuid, force, 3); // Depth = 3 (Ripple Effect)
                return true;
            }
        }
        false
    }

    #[wasm_bindgen]
    pub fn wake_up(&mut self) {
        if let Some(last_time) = self.last_saved {
            let now = Utc::now();
            let minutes = (now - last_time).num_minutes();
            if minutes > 0 { self.current_tick += minutes as u64; }
        }
        self.last_saved = Some(Utc::now());
    }

    // --- DREAM PROTOCOL ---

    #[wasm_bindgen]
    pub fn dream(&mut self) -> String {
        let mut promoted = 0;
        self.current_tick += 480; // +8 horas

        // Iterar valores mutáveis do HashMap é seguro
        for node in self.nodes.values_mut() {
            let meta = node.meta_mut();
            
            // Consolidação (LTP)
            if meta.activation > 0.7 {
                let gain = 0.5 * (1.0 - (meta.stability / 100.0));
                meta.stability += gain;
                promoted += 1;
            }
            
            // Washout (Limpeza de Adenosina)
            let baseline = (meta.stability / 100.0).min(0.2);
            meta.activation = meta.activation * 0.3 + baseline;
        }

        // Poda Segura
        let removed = self.prune_low_stability(1.2);

        format!("Ciclo REM: {} consolidadas, {} removidas.", promoted, removed)
    }

    // --- EXPORT/IMPORT ---

    #[wasm_bindgen]
    pub fn export_backup(&self) -> String {
        serde_json::to_string(&self).unwrap_or("{}".to_string())
    }

    #[wasm_bindgen]
    pub fn import_backup(json: &str) -> LoomGraph {
        serde_json::from_str(json).unwrap_or_else(|_| LoomGraph::new(0.95))
    }

    #[wasm_bindgen]
    pub fn get_context(&mut self, min_activation: f32) -> String {
        self.get_context_prompt(min_activation)
    }

    #[wasm_bindgen]
    pub fn prune_low_stability(&mut self, threshold: f32) -> usize {
        let to_remove: Vec<Uuid> = self.nodes.iter()
            .filter(|(_, n)| n.meta().stability < threshold && n.meta().activation < 0.1)
            .map(|(id, _)| *id)
            .collect();

        if to_remove.is_empty() { return 0; }

        for id in &to_remove {
            if let Some(node) = self.nodes.remove(id) {
                // Limpa Index
                let text = node.extract_text().to_lowercase();
                for token in text.split_whitespace() {
                    let clean = token.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
                    if let Some(list) = self.index.get_mut(&clean) {
                        list.retain(|&uuid| uuid != *id);
                    }
                }
            }
            // Limpa Adjacency (Saída)
            self.adjacency.remove(id);
        }

        // Limpa Adjacency (Entrada - Deep Clean)
        for edges in self.adjacency.values_mut() {
            edges.retain(|conn| !to_remove.contains(&conn.target));
        }

        to_remove.len()
    }
}

// ----------------------------------------------------------------------------
// API INTERNA (Rust Only)
// ----------------------------------------------------------------------------
impl LoomGraph {
    fn add_node_internal(&mut self, node: Node) {
        let id = node.meta().id;
        let text = node.extract_text().to_lowercase();
        
        if !text.is_empty() {
            let tokens: Vec<&str> = text.split_whitespace().collect();
            for token in tokens {
                let clean = token.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
                if !clean.is_empty() {
                    self.index.entry(clean).or_insert(Vec::new()).push(id);
                }
            }
        }

        let mut n = node;
        n.meta_mut().last_tick = self.current_tick;
        self.nodes.insert(id, n);
    }

    pub fn boost_node(&mut self, id: Uuid, amount: f32, depth: u8) {
        if depth == 0 { return; }

        // 1. Boost Local (Mutable Borrow)
        if let Some(node) = self.nodes.get_mut(&id) {
            let tick = self.current_tick;
            let decay = self.decay_rate;
            let meta = node.meta_mut();
            
            // Lazy Decay
            if meta.last_tick < tick {
                let delta = (tick - meta.last_tick) as f32;
                let effective_decay = decay.powf(delta / meta.stability);
                meta.activation *= effective_decay;
                meta.last_tick = tick;
            }

            let real_boost = (1.0 - meta.activation) * amount;
            meta.activation += real_boost;
            meta.stability += (50.0 - meta.stability) * (amount * 0.05);
        } else {
            return; 
        }

        // 2. Coleta Vizinhos (Clone leve apenas da lista deste nó)
        let neighbors = if let Some(list) = self.adjacency.get(&id) {
            list.clone() 
        } else {
            return;
        };

        // 3. Propagação Recursiva
        for conn in neighbors {
            let ripple = amount * conn.weight * 0.5;
            if ripple.abs() > 0.01 {
                self.boost_node(conn.target, ripple, depth - 1);
            }
        }
    }


    pub fn search_native(&mut self, query: &str) -> Vec<(String, f32)> {
        let clean = query.trim().to_lowercase();
        if clean.is_empty() { return Vec::new(); }

        let mut candidates = HashSet::new();

        for (key, uuids) in &self.index {
            if key.contains(&clean) {
                for id in uuids { candidates.insert(*id); }
            }
        }

        let mut results = Vec::new();
        let tick = self.current_tick;

        for id in candidates {
            if let Some(node) = self.nodes.get(&id) {
                // Cálculo PROJETADO (Sem mutar o estado)
                let meta = node.meta();
                
                // Se tick > last_tick, calcula quanto cairia
                let projected_activation = if tick > meta.last_tick {
                    let delta = (tick - meta.last_tick) as f32;
                    let effective_decay = self.decay_rate.powf(delta / meta.stability);
                    meta.activation * effective_decay
                } else {
                    meta.activation
                };

                results.push((id.to_string(), projected_activation));
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }

    fn sanitize_xml(input: &str) -> String {
        input.replace("&", "&amp;")
             .replace("<", "&lt;")
             .replace(">", "&gt;")
             .replace("\"", "&quot;")
             .replace("'", "&apos;")
    }

    pub fn get_context_prompt(&mut self, min_activation: f32) -> String {
        let mut buffer = String::new();
        buffer.push_str("<active_memories>\n");
        
        let mut active_nodes: Vec<&Node> = self.nodes.values()
            .filter(|n| n.meta().activation > min_activation)
            .collect();
        
        active_nodes.sort_by(|a, b| b.meta().activation.partial_cmp(&a.meta().activation).unwrap());

        if active_nodes.is_empty() {
            buffer.push_str("  <memory>No relevant active memories.</memory>\n");
        } else {
            for node in active_nodes {
                let meta = node.meta();
                match node {
                    Node::Concept(_, d) => {
                        buffer.push_str(&format!(
                            "  <memory type='concept' activation='{:.2}' stability='{:.2}'>\n    <name>{}</name>\n    <definition>{}</definition>\n  </memory>\n",
                            meta.activation, meta.stability, 
                            Self::sanitize_xml(&d.name), 
                            Self::sanitize_xml(&d.definition)
                        ));
                    },
                    Node::Episode(_, d) => {
                        buffer.push_str(&format!(
                            "  <memory type='episode' activation='{:.2}' stability='{:.2}' time='{}'>\n    <summary>{}</summary>\n  </memory>\n",
                            meta.activation, meta.stability, 
                            d.timestamp.to_rfc3339(), 
                            Self::sanitize_xml(&d.summary)
                        ));
                    },
                    Node::State(_, d) => {
                        buffer.push_str(&format!(
                            "  <state activation='{:.2}' stability='{:.2}'>\n    <mood valence='{:.2}' arousal='{:.2}' />\n  </state>\n",
                            meta.activation, meta.stability, d.valence, d.arousal
                        ));
                    }
                }
            }
        }
        buffer.push_str("</active_memories>");
        buffer
    }
    
    // Persistência CLI
    pub fn save_to_file(&mut self, filepath: &str) -> std::io::Result<()> {
        self.last_saved = Some(Utc::now());
        let file = File::create(filepath)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)?;
        Ok(())
    }

    pub fn load_from_file(filepath: &str) -> std::io::Result<Self> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let brain = serde_json::from_reader(reader)?;
        Ok(brain)
    }
}