use loom_db::{LoomGraph};
use std::path::Path;

fn main() {
    println!("🇨🇭 LoomDB: Time Travel Test\n");
    let memory_file = "mizuki_time_test.json";
    let mut brain;

    // 1. CARREGAR OU CRIAR
    if Path::new(memory_file).exists() {
        println!("📂 Carregando cérebro existente...");
        brain = LoomGraph::load_from_file(memory_file).unwrap();
        brain.wake_up(); // Atualiza tempo baseado no relógio do sistema
    } else {
        println!("✨ Criando novo cérebro (Tick 0)...");
        brain = LoomGraph::new(0.90); // Decay agressivo de 0.90
        
        // Memória Original (Gênesis)
        brain.add_concept("Genesis".into(), "Memória original.".into());
    }

    // 2. MOSTRAR ESTADO ATUAL
    let tick_atual = brain.current_tick;
    println!("⏰ Tempo Atual do Cérebro: Tick {}", tick_atual);
    
    // Mostra a ativação da memória "Genesis" (usando busca para ver valor projetado com decay)
    let results = brain.search_native("Genesis");
    if let Some((_, activation)) = results.first() {
         println!("📊 Ativação 'Genesis' AGORA: {:.4}", activation);
    } else {
        println!("📊 Memória 'Genesis' não encontrada ou desbotada.");
    }

    // 3. AVANÇAR O TEMPO (Passar 5 ticks)
    println!("\n⏳ Passando 5 ticks de tempo...");
    for _ in 0..5 {
        brain.tick();
    }

    // 4. CRIAR NOVA MEMÓRIA (No futuro)
    let nova_memoria = format!("Memória do Tick {}", brain.current_tick);
    println!("➕ Adicionando: '{}'", nova_memoria);
    
    brain.add_concept(nova_memoria, "Criada no futuro.".into());

    // 5. SALVAR E SAIR
    brain.save_to_file(memory_file).unwrap();
    println!("💾 Estado salvo. Rode novamente para ver o efeito!");
}
