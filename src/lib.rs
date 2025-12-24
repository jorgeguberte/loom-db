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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub id: Uuid,
    pub activation: f32,
    pub last_tick: u64,
}

impl NodeMetadata {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            activation: 1.0,
            last_tick: 0,
        }
    }
}

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
            Node::State(_, _) => "".to_string(), // Estados são sentimentos "mudos" (não indexáveis por texto)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Edge {
    Preceded(Uuid, Uuid),
    Mentioned(Uuid, Uuid),
    Evoked(Uuid, Uuid),
    Associated(Uuid, Uuid, f32),
    Inhibited(Uuid, Uuid, f32),
}

// ============================================================================
// 2. THE ENGINE (The Loom)
// ============================================================================

#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct LoomGraph {
    #[wasm_bindgen(skip)]
    pub nodes: Vec<Node>,
    #[wasm_bindgen(skip)]
    pub edges: Vec<Edge>,
    #[wasm_bindgen(skip)]
    pub current_tick: u64,
    #[wasm_bindgen(skip)]
    pub decay_rate: f32,
    #[wasm_bindgen(skip)]
    pub index: HashMap<String, Vec<usize>>,
    #[wasm_bindgen(skip)]
    pub node_map: HashMap<Uuid, usize>,
    #[wasm_bindgen(skip)]
    pub last_saved: Option<DateTime<Utc>>,
}

// ----------------------------------------------------------------------------
// API PÚBLICA (WASM/JS Friendly)
// ----------------------------------------------------------------------------
#[wasm_bindgen]
impl LoomGraph {
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

    #[wasm_bindgen]
    pub fn search(&mut self, query: &str) -> String {
        let results = self.search_native(query); 
        serde_json::to_string(&results).unwrap_or("[]".to_string())
    }

    #[wasm_bindgen]
    pub fn get_context(&mut self, min_activation: f32) -> String {
        self.get_context_prompt(min_activation)
    }

    #[wasm_bindgen]
    pub fn tick(&mut self) {
        self.current_tick += 1;
    }

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

    #[wasm_bindgen]
    pub fn export_backup(&self) -> String {
        serde_json::to_string(&self).unwrap_or("{}".to_string())
    }

    #[wasm_bindgen]
    pub fn import_backup(json: &str) -> LoomGraph {
        serde_json::from_str(json).unwrap_or_else(|_| LoomGraph::new(0.95))
    }

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
}

// ----------------------------------------------------------------------------
// API INTERNA (Rust Only) - O "Motor" Real
// ----------------------------------------------------------------------------
impl LoomGraph {
    // Método universal interno (usado pelo add_concept, add_episode, etc.)
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

    pub fn get_activation(&mut self, node_idx: usize) -> f32 {
        let tick_now = self.current_tick;
        let decay = self.decay_rate;
        let node = &mut self.nodes[node_idx];
        let meta = node.meta_mut();

        if meta.last_tick < tick_now {
            let delta_t = (tick_now - meta.last_tick) as i32;
            meta.activation *= decay.powi(delta_t);
            meta.last_tick = tick_now;
        }
        meta.activation
    }

    pub fn boost_node(&mut self, node_idx: usize, amount: f32, propagate: bool) {
        let current_activation = self.get_activation(node_idx);
        let meta = self.nodes[node_idx].meta_mut();
        let real_boost = (1.0 - current_activation) * amount;
        meta.activation += real_boost;
        meta.last_tick = self.current_tick;
        
        if propagate {
            let node_id = meta.id;
            let edges_snapshot = self.edges.clone(); 
            for edge in edges_snapshot {
                if let Edge::Associated(source, target, weight) = edge {
                    if source == node_id {
                        if let Some(&neighbor_idx) = self.node_map.get(&target) {
                            let ripple_effect = amount * weight * 0.5;
                            self.boost_node(neighbor_idx, ripple_effect, false);
                        }
                    }
                }
            }
        }
    }

    pub fn get_context_prompt(&mut self, min_activation: f32) -> String {
        let mut buffer = String::new();
        buffer.push_str("<current_state>\n");
        
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
                            "  <memory type='concept' activation='{:.2}'>\n    <name>{}</name>\n    <definition>{}</definition>\n  </memory>\n",
                            meta.activation, data.name, data.definition
                        ));
                    },
                    Node::Episode(_, data) => {
                        buffer.push_str(&format!(
                            "  <memory type='episode' activation='{:.2}' time='{}'>\n    <summary>{}</summary>\n  </memory>\n",
                            meta.activation, data.timestamp.to_rfc3339(), data.summary
                        ));
                    },
                    Node::State(_, data) => {
                        buffer.push_str(&format!(
                            "  <state activation='{:.2}'>\n    <mood valence='{:.2}' arousal='{:.2}' />\n  </state>\n",
                            meta.activation, data.valence, data.arousal
                        ));
                    }
                }
            }
        }
        buffer.push_str("</current_state>");
        buffer
    }

    // Persistência CLI (Desktop)
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