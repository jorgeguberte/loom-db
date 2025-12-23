use std::fs::File;
use std::io::BufReader;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap; // Importante para o índice
use uuid::Uuid;

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

    // Helper para extrair texto indexável do nó
    pub fn extract_text(&self) -> String {
        match self {
            Node::Episode(_, d) => d.summary.clone(),
            Node::Concept(_, d) => format!("{} {}", d.name, d.definition),
            Node::State(_, _) => "".to_string(), // Emoções puras não têm texto indexável por enquanto
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
#[derive(Serialize, Deserialize)]
pub struct LoomGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub current_tick: u64,
    pub decay_rate: f32,
    // O Córtex Associativo: Mapeia "palavra" -> "lista de índices no vetor nodes"
    pub index: HashMap<String, Vec<usize>>, 
    pub node_map: HashMap<Uuid, usize>,
}

impl LoomGraph {
    pub fn new(decay_rate: f32) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            current_tick: 0,
            decay_rate,
            index: HashMap::new(),
            node_map: HashMap::new(), // Inicializa o mapa
        }
    }

    pub fn tick(&mut self) {
        self.current_tick += 1;
    }

    pub fn add_node(&mut self, node: Node) {
        let idx = self.nodes.len();
        let id = node.meta().id; // Pega o ID antes de mover
        
        // 1. Indexar o conteúdo textual
        let text = node.extract_text().to_lowercase();
        let tokens: Vec<&str> = text.split_whitespace().collect();
        
        for token in tokens {
            let clean = token.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
            if !clean.is_empty() {
                self.index.entry(clean).or_insert(Vec::new()).push(idx);
            }
        }

        // 2. Mapear UUID -> Index
        self.node_map.insert(id, idx);

        // 3. Adicionar ao vetor
        let mut n = node;
        n.meta_mut().last_tick = self.current_tick;
        self.nodes.push(n);
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

    //Agora aceita um parâmetro `propagate` para evitar loops infinitos
    pub fn boost_node(&mut self, node_idx: usize, amount: f32, propagate: bool) {
        // 1. Aplica o Boost no nó alvo (Assintótico)
        let current_activation = self.get_activation(node_idx);
        let meta = self.nodes[node_idx].meta_mut();

        // Boost assintótico: (1.0 - atual) * força
        let real_boost = (1.0 - current_activation) * amount;
        meta.activation += real_boost;
        meta.last_tick = self.current_tick;
        
        // 2. SPREAD ACTIVATION (A Onda) 🌊
        // Se `propagate` for true, espalha energia para os vizinhos
        if propagate {
            let node_id = meta.id;
            
            // Clonamos as arestas para não brigar com o Borrow Checker
            // (Numa versão ultra-otimizada faríamos diferente, mas assim é seguro)
            let edges_snapshot = self.edges.clone(); 
            
            for edge in edges_snapshot {
                // Verifica conexões onde este nó é a Origem
                if let Edge::Associated(source, target, weight) = edge {
                    if source == node_id {
                        // Achamos um vizinho!
                        if let Some(&neighbor_idx) = self.node_map.get(&target) {
                            // A energia dissipa: Boost Original * Peso da Conexão * Fator de Amortecimento (0.5)
                            // Exemplo: Boost 0.5 * Peso 0.9 * Amortecimento 0.5 = Vizinho ganha 0.225
                            let ripple_effect = amount * weight * 0.5;
                            
                            // Chama boost recursivamente, mas com propagate=false para parar no 1º nível (evita loop infinito)
                            self.boost_node(neighbor_idx, ripple_effect, false);
                        }
                    }
                }
                // Poderíamos adicionar lógica para outros tipos de aresta aqui (Ex: Evoked -> Emoção)
            }
        }

    }

    // A BUSCA DO AGENTE
    // Retorna índices dos nós encontrados, ORDENADOS por Ativação (Mais relevante primeiro)
    pub fn search(&mut self, query: &str) -> Vec<(usize, f32)> {
        let clean_query = query.to_lowercase();
        let clean_query = clean_query.trim();

        // A MUDANÇA MÁGICA ESTÁ AQUI: .cloned()
        // Pegamos a lista de índices e fazemos uma cópia para a nossa mão.
        // Isso liberta o 'self.index' para podermos usar o 'self' logo abaixo.
        if let Some(indices) = self.index.get(clean_query).cloned() {
            let mut results = Vec::new();
            
            for idx in indices {
                // Agora o compilador sabe que não estamos mais segurando o HashMap
                // Podemos mutar o self livremente!
                let activation = self.get_activation(idx);
                results.push((idx, activation));
            }

            results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            results.dedup_by_key(|k| k.0);
            return results;
        }

        Vec::new()
    }

    // ========================================================================
    // PERSISTÊNCIA (Save/Load)
    // ========================================================================

    /// Salva o cérebro inteiro num arquivo JSON
    pub fn save_to_file(&self, filepath: &str) -> std::io::Result<()> {
        let file = std::fs::File::create(filepath)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self)?;
        Ok(())
    }

    /// Carrega um cérebro existente de um arquivo JSON
    pub fn load_from_file(filepath: &str) -> std::io::Result<Self> {
        let file = std::fs::File::open(filepath)?;
        let reader = std::io::BufReader::new(file);
        let brain = serde_json::from_reader(reader)?;
        Ok(brain)
    }

    // ========================================================================
    // CONTEXT BUILDER (A Voz)
    // ========================================================================

    // Gera um prompt XML formatado para LLMs (Gemini/GPT/Claude)
    pub fn get_context_prompt(&mut self, min_activation: f32) -> String {
        let mut buffer = String::new();
        
        buffer.push_str("<current_state>\n");
        
        // 1. Filtrar memórias relevantes (Acima do Threshold)
        // Precisamos coletar os índices primeiro para iterar
        let mut active_indices: Vec<usize> = (0..self.nodes.len())
            .filter(|&idx| self.get_activation(idx) > min_activation)
            .collect();

        // 2. Ordenar por relevância (Mais ativos primeiro)
        active_indices.sort_by(|&a, &b| {
            let val_a = self.nodes[a].meta().activation;
            let val_b = self.nodes[b].meta().activation;
            val_b.partial_cmp(&val_a).unwrap()
        });

        // 3. Gerar XML
        if active_indices.is_empty() {
            buffer.push_str("  <memory>No relevant active memories.</memory>\n");
        } else {
            for idx in active_indices {
                let node = &self.nodes[idx];
                let meta = node.meta();
                
                // Formata com base no tipo
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
}