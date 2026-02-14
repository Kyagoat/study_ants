// On utilise les modules exposés par la lib
use ants_project::ant::{Ant, AntsType};
use ants_project::ants_game_manager::AntsGameManager;
use ants_project::cli_args::SimulationConfig;
use ants_project::interface::Interface;

fn main() -> Result<(), eframe::Error> {
    // Parse les arguments de la ligne de commande
    let config = SimulationConfig::from_args();

    // Valider la configuration
    if let Err(e) = config.validate() {
        eprintln!("Erreur de configuration: {}", e);
        eprintln!("Utilisez --help pour voir les options disponibles");
        std::process::exit(1);
    }

    // Mode GUI ou CLI
    if config.use_gui {
        let options = eframe::NativeOptions::default();
        eframe::run_native(
            "Ant Simulator",
            options,
            // On utilise Interface depuis la lib
            Box::new(move |_cc| Ok(Box::new(Interface::new_with_config(config.clone())))),
        )
    } else {
        println!("Mode CLI actif. Simulation en cours...");

        let mut ants = Vec::new();
        for _ in 0..config.num_explorers {
            ants.push(Ant::new(AntsType::EXPLORER));
        }
        for _ in 0..config.num_pickers {
            ants.push(Ant::new(AntsType::PICKER));
        }
        for _ in 0..config.num_fighters {
            ants.push(Ant::new(AntsType::FIGHTER));
        }

        let mut manager = AntsGameManager::new_game_mode_random(
            config.grid_width,
            config.grid_height,
            ants,
            config.clone(),
        );

        let mut tick = 0;
        while tick < config.max_ticks {
            manager.game_step();
            tick += 1;
            if manager.is_game_finished() {
                break;
            }
        }
        println!("{}", tick);
        Ok(())
    }
}
