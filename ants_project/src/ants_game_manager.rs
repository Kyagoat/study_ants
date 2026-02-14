use crate::ant::{Ant, AntsMode, AntsType};
use crate::cli_args::SimulationConfig;
use crate::grid::Grid;
use crate::pheromone::{Action, PheromoneMap};
use crate::tile::{Tile, TileType};
use rand::Rng;

#[derive(Clone)]
pub struct GameStateSnapshot {
    pub grid: Grid,
    pub ants: Vec<Ant>,
    pub pheromones_food: PheromoneMap,
    pub pheromones_nest: PheromoneMap,
}

pub struct QLearningParams {
    pub alpha: f32,
    pub gamma: f32,
    pub epsilon: f32,
}

pub struct AntsGameManager {
    pub grid: Grid,
    pub ants: Vec<Ant>,
    pub pheromones_food: PheromoneMap,
    pub pheromones_nest: PheromoneMap,
    pub rl_params: QLearningParams,
    pub config: SimulationConfig,
    pub history: Vec<GameStateSnapshot>,
    pub current_tick_index: usize,
}

impl AntsGameManager {
    pub fn new(
        width: u32,
        height: u32,
        tiles: Vec<Tile>,
        ants: Vec<Ant>,
        config: SimulationConfig,
    ) -> Self {
        let grid = Grid::new_with_tiles(width, height, tiles);

        // On crée l'état initial
        let pheromones_food = PheromoneMap::new(width, height);
        let pheromones_nest = PheromoneMap::new(width, height);

        let mut manager = AntsGameManager {
            grid: grid.clone(),
            ants: ants.clone(),
            pheromones_food: pheromones_food.clone(),
            pheromones_nest: pheromones_nest.clone(),
            rl_params: QLearningParams {
                alpha: config.alpha,
                gamma: config.gamma,
                epsilon: config.epsilon,
            },
            config,
            history: Vec::new(),
            current_tick_index: 0,
        };

        // Sauvegarder l'état initial (tick 0)
        manager.save_snapshot();
        manager
    }

    pub fn new_game_mode_random(
        width: u32,
        height: u32,
        mut ants: Vec<Ant>,
        config: SimulationConfig,
    ) -> Self {
        let grid = Grid::new_random(width, height);

        for ant in &mut ants {
            ant.spawn_at_nest(&grid);
        }

        let pheromones_food = PheromoneMap::new(width, height);
        let pheromones_nest = PheromoneMap::new(width, height);

        let mut manager = AntsGameManager {
            grid: grid.clone(),
            ants: ants.clone(),
            pheromones_food: pheromones_food.clone(),
            pheromones_nest: pheromones_nest.clone(),
            rl_params: QLearningParams {
                alpha: config.alpha,
                gamma: config.gamma,
                epsilon: config.epsilon,
            },
            config,
            history: Vec::new(),
            current_tick_index: 0,
        };

        // Sauvegarder l'état initial (tick 0)
        manager.save_snapshot();
        manager
    }

    fn save_snapshot(&mut self) {
        // Si on est revenu dans le passé et qu'on a modifié quelque chose (ou qu'on continue),
        // on supprime le futur alternatif.
        if self.current_tick_index < self.history.len().saturating_sub(1) {
            self.history.truncate(self.current_tick_index + 1);
        }

        self.history.push(GameStateSnapshot {
            grid: self.grid.clone(),
            ants: self.ants.clone(),
            pheromones_food: self.pheromones_food.clone(),
            pheromones_nest: self.pheromones_nest.clone(),
        });

        // Mettre à jour l'index pour pointer sur le dernier élément
        self.current_tick_index = self.history.len() - 1;
    }

    pub fn restore_snapshot(&mut self, index: usize) {
        if index < self.history.len() {
            let snapshot = &self.history[index];
            self.grid = snapshot.grid.clone();
            self.ants = snapshot.ants.clone();
            self.pheromones_food = snapshot.pheromones_food.clone();
            self.pheromones_nest = snapshot.pheromones_nest.clone();
            self.current_tick_index = index;
        }
    }

    pub fn game_step(&mut self) {
        // Synchroniser les paramètres Q-Learning depuis la configuration actuelle
        self.rl_params.alpha = self.config.alpha;
        self.rl_params.gamma = self.config.gamma;
        self.rl_params.epsilon = self.config.epsilon;

        let width = self.grid.get_width();
        let height = self.grid.get_height();

        // Calculer la densité de fourmis sur chaque case pour éviter l'empilement excessif
        let mut ant_density = self.compute_ant_density();

        // Gérer le spawn intelligent des fourmis en sortant du nid
        self.manage_smart_spawn(&ant_density, width);

        let mut i = 0;
        while i < self.ants.len() {
            // Ignorer les fourmis qui ne sont pas encore sur la carte
            if self.ants[i].position.is_none() {
                i += 1;
                continue;
            }

            // Gérer le cooldown pour que les fourmis ne se déplacent pas à chaque tick
            if self.ants[i].cooldown > 0 {
                self.ants[i].cooldown -= 1;
                i += 1;
                continue;
            }
            self.ants[i].cooldown = self.ants[i].seconds_for_movement;

            let (x, y) = self.ants[i].position.unwrap();
            let mode = self.ants[i].mode;
            let scope = self.ants[i].scope;

            // Sélectionner la prochaine action via la stratégie Epsilon-Greedy (exploration vs exploitation)
            let (chosen_action, q_curr) = self.choose_action(x, y, mode);
            let (nx, ny) = self.ants[i].get_target_position(chosen_action);

            // Vérifier si le mouvement est valide et autorisé
            let is_out = nx >= width || ny >= height;
            let mut move_allowed = !is_out && self.grid.is_walkable(nx, ny);
            let mut is_lethal = false;

            if !is_out {
                is_lethal = self.grid.is_lethal(nx, ny);
                // Vérifier que la case cible n'est pas saturée (max 10 fourmis par case)
                let target_idx = (ny * width + nx) as usize;
                if ant_density.get(target_idx).copied().unwrap_or(0) >= 10 {
                    move_allowed = false;
                }
            }

            // Les fourmis avec vision détectent les zones mortelles et refusent d'avancer
            if is_lethal && scope > 0 {
                move_allowed = false;
            }

            // Calculer la récompense en fonction du type de case visée
            let reward = self.calculate_reward(is_lethal, mode, nx, ny);

            let map = match mode {
                AntsMode::FINDING => &self.pheromones_food,
                _ => &self.pheromones_nest,
            };

            // Calculer la valeur Q maximale de l'état suivant pour la formule de Bellman
            let max_next_q = if is_out || is_lethal {
                0.0
            } else {
                map.get_max_q(nx, ny, &self.grid)
            };

            // Calculer la correction Delta selon la formule Q-Learning: Alpha * (Reward + Gamma * MaxNext - Current)
            let delta =
                self.rl_params.alpha * (reward + self.rl_params.gamma * max_next_q - q_curr);

            match mode {
                AntsMode::FINDING => self
                    .pheromones_food
                    .queue_update(x, y, chosen_action, delta),
                AntsMode::RETURNING => {
                    self.pheromones_nest
                        .queue_update(x, y, chosen_action, delta)
                }
            };

            // Exécuter le mouvement si autorisé, ou tuer la fourmi si elle entre dans une zone mortelle
            if move_allowed {
                if is_lethal {
                    // La fourmi meurt et disparait de la carte
                    let idx = (y * width + x) as usize;
                    if idx < ant_density.len() {
                        ant_density[idx] = ant_density[idx].saturating_sub(1);
                    }
                    self.ants[i].position = None;
                } else {
                    // Déplacer la fourmi et mettre à jour la densité
                    let old_idx = (y * width + x) as usize;
                    let new_idx = (ny * width + nx) as usize;
                    if old_idx < ant_density.len() {
                        ant_density[old_idx] = ant_density[old_idx].saturating_sub(1);
                    }
                    if new_idx < ant_density.len() {
                        ant_density[new_idx] += 1;
                    }

                    self.ants[i].move_to(nx, ny);

                    // Gérer les interactions: manger une nourriture, déposer au nid ou booster phéromones
                    Self::handle_interactions(
                        &mut self.grid,
                        &mut self.ants[i],
                        nx,
                        ny,
                        &mut self.pheromones_food,
                        &mut self.pheromones_nest,
                        &self.config,
                    );
                }
            }
            i += 1;
        }

        // Appliquer l'évaporation et tous les mises à jour de phéromones en attente
        self.pheromones_food
            .apply_tick(self.config.pheromone_evaporation);
        self.pheromones_nest
            .apply_tick(self.config.pheromone_evaporation);
        self.save_snapshot();
    }

    fn compute_ant_density(&self) -> Vec<u8> {
        let width = self.grid.get_width();
        let height = self.grid.get_height();
        let mut density = vec![0u8; (width * height) as usize];

        for ant in &self.ants {
            if let Some((ax, ay)) = ant.position {
                let idx = (ay * width + ax) as usize;
                if idx < density.len() {
                    density[idx] += 1;
                }
            }
        }
        density
    }

    fn manage_smart_spawn(&mut self, ant_density: &[u8], width: u32) {
        // Récupérer la limite du nombre de fourmis actives depuis la configuration
        let max_active_ants = self.config.nest_capacity as usize;
        const MIN_EXPLORERS_ACTIVE: usize = 3;

        let active_explorers = self
            .ants
            .iter()
            .filter(|a| a.position.is_some() && a.ant_type == AntsType::EXPLORER)
            .count();
        let active_total = self.ants.iter().filter(|a| a.position.is_some()).count();

        // Arrêter le spawn si le nid est saturé ou si la limite globale de fourmis actives est atteinte
        if active_total >= max_active_ants {
            return;
        }

        let nest_pos = match self.grid.get_nest_position() {
            Some(pos) => pos,
            None => return,
        };

        let nest_idx = (nest_pos.1 * width + nest_pos.0) as usize;
        if ant_density.get(nest_idx).copied().unwrap_or(0) >= 10 {
            return;
        }

        // Privilégier la sortie des explorateurs si leur nombre est inférieur au minimum
        let target_type = if active_explorers < MIN_EXPLORERS_ACTIVE {
            Some(AntsType::EXPLORER)
        } else {
            None
        };

        let ant_index_to_spawn = self.ants.iter().position(|a| {
            a.position.is_none() && (target_type.is_none() || a.ant_type == target_type.unwrap())
        });

        // Déployer la fourmi trouvée en la plaçant au nid
        if let Some(idx) = ant_index_to_spawn {
            self.ants[idx].position = Some(nest_pos);
            self.ants[idx].mode = AntsMode::FINDING;
            self.ants[idx].current_charge = 0;
            self.ants[idx].cooldown = 2;
        } else if target_type.is_some() {
            // Si pas d'explorateur disponible, déployer n'importe quelle autre fourmi inactive
            if let Some(idx) = self.ants.iter().position(|a| a.position.is_none()) {
                self.ants[idx].position = Some(nest_pos);
                self.ants[idx].mode = AntsMode::FINDING;
            }
        }
    }

    fn choose_action(&self, x: u32, y: u32, mode: AntsMode) -> (Action, f32) {
        let mut rng = rand::thread_rng();
        let map = match mode {
            AntsMode::FINDING => &self.pheromones_food,
            AntsMode::RETURNING => &self.pheromones_nest,
        };

        if rng.gen::<f32>() < self.rl_params.epsilon {
            let action = match rng.gen_range(0..4) {
                0 => Action::Up,
                1 => Action::Down,
                2 => Action::Left,
                _ => Action::Right,
            };
            (action, map.get_q(x, y, action))
        } else {
            let best = map.get_best_action(x, y, &self.grid);
            (best, map.get_q(x, y, best))
        }
    }

    fn handle_interactions(
        grid: &mut Grid,
        ant: &mut Ant,
        nx: u32,
        ny: u32,
        phero_food: &mut PheromoneMap,
        phero_nest: &mut PheromoneMap,
        config: &SimulationConfig,
    ) {
        // Calculer le boost immédiat basé sur la récompense configurée pour trouver de la nourriture
        let immediate_boost = config.reward_food * 0.5;

        match ant.mode {
            AntsMode::FINDING => {
                if grid.has_food(nx, ny) {
                    if let Some(tile) = grid.get_mut_tile((nx, ny)) {
                        if let TileType::FoodSource { amount } = &mut tile.tile_type {
                            if *amount > 0 {
                                *amount = amount.saturating_sub(1);
                                ant.current_charge = ant.maximal_charge;
                                ant.mode = AntsMode::RETURNING;
                                phero_food.queue_update(nx, ny, Action::Stay, immediate_boost);
                            }
                        }
                    }
                }
            }
            AntsMode::RETURNING => {
                if grid.is_nest(nx, ny) {
                    grid.add_food_to_nest(ant.current_charge);
                    ant.current_charge = 0;
                    ant.mode = AntsMode::FINDING;
                    phero_nest.queue_update(nx, ny, Action::Stay, immediate_boost);
                }
            }
        }
    }

    pub fn calculate_reward(&self, is_lethal: bool, mode: AntsMode, nx: u32, ny: u32) -> f32 {
        if is_lethal {
            return self.config.reward_death;
        }

        match mode {
            AntsMode::FINDING if self.grid.has_food(nx, ny) => self.config.reward_food,
            AntsMode::RETURNING if self.grid.is_nest(nx, ny) => self.config.reward_nest,
            _ => self.config.reward_default,
        }
    }

    pub fn is_game_finished(&self) -> bool {
        // La simulation s'arrête quand toutes les fourmis sont mortes ou qu'aucune nourriture n'est disponible sur la carte
        self.ants.iter().all(|ant| ant.position.is_none()) || !self.grid.is_food_remaining()
    }
}
