use ants_project::ant::{Ant, AntsType};
use ants_project::ants_game_manager::AntsGameManager;
use ants_project::cli_args::SimulationConfig;
use rayon::prelude::*;
// Import magique pour le parallélisme
use std::time::Instant;

#[derive(Debug, Clone)]
struct SimulationResult {
    alpha: f32,
    gamma: f32,
    epsilon: f32,
    ticks: u64,
}

fn main() {
    let start_total = Instant::now();
    println!("🚀 Démarrage du Méta-Optimiseur (Mode TURBO - In-Memory)");

    // 1. Définition des hyperparamètres à tester
    let alphas = vec![0.1, 0.3, 0.5, 0.7, 0.9];
    let gammas = vec![0.8, 0.9, 0.95, 0.99];
    let epsilons = vec![0.01, 0.05, 0.1, 0.2];

    // Création de toutes les combinaisons
    let mut combinations = Vec::new();
    for &alpha in &alphas {
        for &gamma in &gammas {
            for &epsilon in &epsilons {
                combinations.push((alpha, gamma, epsilon));
            }
        }
    }

    println!(
        "⚡ Lancement de {} simulations en parallèle...",
        combinations.len()
    );

    // 2. EXÉCUTION PARALLÈLE (C'est ici que la magie opère)
    // .par_iter() remplace .iter() et distribue le travail sur tous les coeurs CPU
    let mut results: Vec<SimulationResult> = combinations
        .par_iter()
        .map(|&(alpha, gamma, epsilon)| run_single_simulation(alpha, gamma, epsilon))
        .collect();

    let duration = start_total.elapsed();
    println!("✅ Terminé en {:.2?}", duration);

    // 3. Analyse des résultats (Tri)
    results.sort_by_key(|r| r.ticks);

    if let Some(best) = results.first() {
        println!("\n🏆 MEILLEURE CONFIGURATION :");
        println!("   Alpha   : {}", best.alpha);
        println!("   Gamma   : {}", best.gamma);
        println!("   Epsilon : {}", best.epsilon);
        println!("   Temps   : {} ticks", best.ticks);
    }
}

// Cette fonction exécute une simulation complète SANS affichage, purement mathématique
fn run_single_simulation(alpha: f32, gamma: f32, epsilon: f32) -> SimulationResult {
    // Configuration optimisée pour le test
    let config = SimulationConfig {
        grid_width: 30,
        grid_height: 30,
        num_explorers: 10,
        num_pickers: 20,
        num_fighters: 0,
        alpha,
        gamma,
        epsilon,
        max_ticks: 100_000,  // Sécurité anti-boucle infinie
        simulation_speed: 0, // Inutile ici mais requis par la struct
        // Paramètres standards
        reward_food: 1000.0,
        reward_nest: 1000.0,
        reward_death: -100.0,
        reward_default: -1.0,
        nest_capacity: 100,
        pheromone_evaporation: 0.999,
        use_gui: false,
        output_file: None,
    };

    // Création des fourmis (Rapide, en mémoire)
    let mut ants = Vec::with_capacity(30);
    for _ in 0..config.num_explorers {
        ants.push(Ant::new(AntsType::EXPLORER));
    }
    for _ in 0..config.num_pickers {
        ants.push(Ant::new(AntsType::PICKER));
    }

    // Initialisation du Manager
    let mut manager = AntsGameManager::new_game_mode_random(
        config.grid_width,
        config.grid_height,
        ants,
        config.clone(),
    );

    // BOUCLE DE SIMULATION PURE
    // Pas de sleep, pas d'affichage, juste du calcul CPU brut
    let mut tick = 0;
    while tick < config.max_ticks {
        manager.game_step();
        tick += 1;

        // Condition de fin (à adapter selon ta logique de victoire)
        if manager.is_game_finished() {
            break;
        }
    }

    SimulationResult {
        alpha,
        gamma,
        epsilon,
        ticks: tick,
    }
}
