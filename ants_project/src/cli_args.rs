/// Module de gestion des paramètres en ligne de commande
use std::env;
use std::process;


#[derive(Clone, Debug)]
pub struct SimulationConfig {
    // --- Paramètres de grille ---
    pub grid_width: u32,
    pub grid_height: u32,

    // --- Paramètres de fourmis ---
    pub num_explorers: u32,
    pub num_fighters: u32,
    pub num_pickers: u32,

    // --- Paramètres Q-Learning ---
    pub alpha: f32,   // Facteur d'apprentissage (0.0-1.0)
    pub gamma: f32,   // Facteur d'actualisation (0.0-1.0)
    pub epsilon: f32, // Facteur ε-greedy (0.0-1.0)

    // --- Paramètres de simulation ---
    pub max_ticks: u64,        // Limite de temps (1 milliard par défaut)
    pub simulation_speed: u64, // Vitesse en ms (pour GUI)

    // --- Paramètres de récompenses ---
    pub reward_food: f32,    // Nourriture trouvée
    pub reward_nest: f32,    // Retour au nid
    pub reward_death: f32,   // Zone mortelle
    pub reward_default: f32, // Case normale

    // --- Paramètres de nid ---
    pub nest_capacity: u32,         // Capacité d'accueil du nid
    pub pheromone_evaporation: f32, // Taux d'évaporation (0.01 = 1%)

    // --- Mode d'exécution ---
    pub use_gui: bool,               // Utiliser l'interface graphique
    pub output_file: Option<String>, // Fichier de résultats
}

impl Default for SimulationConfig {
    fn default() -> Self {
        SimulationConfig {
            grid_width: 20,
            grid_height: 20,

            num_explorers: 2,
            num_fighters: 1,
            num_pickers: 3,

            alpha: 0.1,
            gamma: 0.99,
            epsilon: 0.05,

            max_ticks: 1_000_000_000,
            simulation_speed: 100,

            reward_food: 1000.0,
            reward_nest: 1000.0,
            reward_death: -100.0,
            reward_default: -1.0,

            nest_capacity: 100,
            pheromone_evaporation: 0.01,

            use_gui: true,
            output_file: None,
        }
    }
}

impl SimulationConfig {
    /// Parse les arguments de la ligne de commande
    pub fn from_args() -> Self {
        let mut config = SimulationConfig::default();
        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            // Pas d'argument = GUI par défaut
            return config;
        }

        let mut i = 1;
        while i < args.len() {
            let arg = &args[i];

            match arg.as_str() {
                // --- Mode ---
                "--gui" => config.use_gui = true,
                "--cli" => config.use_gui = false,

                // --- Grille ---
                "--width" => {
                    i += 1;
                    if i < args.len() {
                        config.grid_width = args[i].parse().unwrap_or(20);
                    }
                }
                "--height" => {
                    i += 1;
                    if i < args.len() {
                        config.grid_height = args[i].parse().unwrap_or(20);
                    }
                }

                // --- Fourmis ---
                "--explorers" => {
                    i += 1;
                    if i < args.len() {
                        config.num_explorers = args[i].parse().unwrap_or(2);
                    }
                }
                "--fighters" => {
                    i += 1;
                    if i < args.len() {
                        config.num_fighters = args[i].parse().unwrap_or(1);
                    }
                }
                "--pickers" => {
                    i += 1;
                    if i < args.len() {
                        config.num_pickers = args[i].parse().unwrap_or(3);
                    }
                }

                // --- Q-Learning ---
                "--alpha" => {
                    i += 1;
                    if i < args.len() {
                        config.alpha = args[i].parse().unwrap_or(0.1);
                    }
                }
                "--gamma" => {
                    i += 1;
                    if i < args.len() {
                        config.gamma = args[i].parse().unwrap_or(0.9);
                    }
                }
                "--epsilon" => {
                    i += 1;
                    if i < args.len() {
                        config.epsilon = args[i].parse().unwrap_or(0.05);
                    }
                }

                // --- Limite ---
                "--max-ticks" => {
                    i += 1;
                    if i < args.len() {
                        config.max_ticks = args[i].parse().unwrap_or(1_000_000_000);
                    }
                }

                // Configurer le fichier de sortie pour les résultats de simulation
                "--output" => {
                    i += 1;
                    if i < args.len() {
                        config.output_file = Some(args[i].clone());
                    }
                }

                "--help" => {
                    Self::print_help();
                    process::exit(0);
                }

                _ => {
                    eprintln!("Argument inconnu: {}", arg);
                }
            }

            i += 1;
        }

        config
    }

    pub fn print_help() {
        println!("Usage: ants_project [OPTIONS]");
        println!();
        println!("OPTIONS:");
        println!("  --gui                  Utiliser l'interface graphique (défaut)");
        println!("  --cli                  Mode ligne de commande");
        println!("  --width <N>            Largeur de la grille (défaut: 20)");
        println!("  --height <N>           Hauteur de la grille (défaut: 20)");
        println!("  --explorers <N>        Nombre d'explorateurs (défaut: 2)");
        println!("  --fighters <N>         Nombre de combattantes (défaut: 1)");
        println!("  --pickers <N>          Nombre de récolteuses (défaut: 3)");
        println!("  --alpha <F>            Facteur d'apprentissage (défaut: 0.1)");
        println!("  --gamma <F>            Facteur d'actualisation (défaut: 0.9)");
        println!("  --epsilon <F>          Facteur ε-greedy (défaut: 0.05)");
        println!("  --max-ticks <N>        Limite de temps en ticks (défaut: 1000000000)");
        println!("  --output <FILE>        Fichier de résultats");
        println!("  --help                 Afficher cette aide");
        println!();
        println!("EXEMPLES:");
        println!("  ants_project --gui --width 30 --height 30");
        println!("  ants_project --cli --alpha 0.2 --gamma 0.8 --output results.txt");
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.grid_width == 0 || self.grid_height == 0 {
            return Err("La grille doit avoir des dimensions > 0".to_string());
        }

        if self.alpha < 0.0 || self.alpha > 1.0 {
            return Err("alpha doit être entre 0.0 et 1.0".to_string());
        }

        if self.gamma < 0.0 || self.gamma > 1.0 {
            return Err("gamma doit être entre 0.0 et 1.0".to_string());
        }

        if self.epsilon < 0.0 || self.epsilon > 1.0 {
            return Err("epsilon doit être entre 0.0 et 1.0".to_string());
        }

        if self.pheromone_evaporation < 0.0 || self.pheromone_evaporation > 1.0 {
            return Err("pheromone_evaporation doit être entre 0.0 et 1.0".to_string());
        }

        Ok(())
    }
}
