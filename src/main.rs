use loom_db::{LoomGraph, Node, NodeMetadata, ConceptData};
use std::path::Path;

fn main() {
    println!("🇨🇭 LoomDB: Time Travel Test\n");
    let memory_file = "mizuki_time_test.json";
    let mut brain;

    // 1. CARREGAR OU CRIAR
    if Path::new(memory_file).exists() {
        println!("📂 Carregando cérebro existente...");
        brain = LoomGraph::load_from_file(memory_file).unwrap();
        brain.wake_up();
    } else {
        println!("✨ Criando novo cérebro (Tick 0)...");
        brain = LoomGraph::new(0.90); // Decay agressivo de 0.90
        
        // Memória Original (Gênesis)
        brain.add_node(Node::Concept(NodeMetadata::new(), ConceptData {
            name: "Genesis".into(),
            definition: "Memória original.".into()
        }));
    }

    // 2. MOSTRAR ESTADO ATUAL
    let tick_atual = brain.current_tick;
    println!("⏰ Tempo Atual do Cérebro: Tick {}", tick_atual);
    
    // Mostra a ativação da memória "Genesis" (Nó 0)
    // Se a memória existir (indices > 0), pegamos a primeira
    if !brain.nodes.is_empty() {
        let ativacao = brain.get_activation(0);
        println!("📊 Ativação 'Genesis' AGORA: {:.4}", ativacao);
    }

    // 3. AVANÇAR O TEMPO (Passar 5 ticks)
    println!("\n⏳ Passando 5 ticks de tempo...");
    for _ in 0..5 {
        brain.tick();
    }

    // 4. CRIAR NOVA MEMÓRIA (No futuro)
    let nova_memoria = format!("Memória do Tick {}", brain.current_tick);
    println!("➕ Adicionando: '{}'", nova_memoria);
    
    brain.add_node(Node::Concept(NodeMetadata::new(), ConceptData {
        name: nova_memoria,
        definition: "Criada no futuro.".into()
    }));

    // 5. SALVAR E SAIR
    brain.save_to_file(memory_file).unwrap();
    println!("💾 Estado salvo. Rode novamente para ver o efeito!");
}