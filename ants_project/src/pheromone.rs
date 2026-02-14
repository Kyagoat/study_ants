// src/pheromones.rs
use crate::grid::Grid;
use std::collections::HashMap;

// Énumération des cinq actions possibles pour une fourmi
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Stay = 4,
}

impl Action {
    pub fn all() -> impl Iterator<Item = Action> {
        [
            Action::Up,
            Action::Down,
            Action::Left,
            Action::Right,
            Action::Stay,
        ]
        .iter()
        .copied()
    }

    // Convertir l'énumération en indice pour le stockage dans le tableau de Q-values
    pub fn to_usize(&self) -> usize {
        *self as usize
    }
}

#[derive(Clone)]
pub struct PheromoneMap {
    pub(crate) width: u32,
    pub(crate) height: u32,
    data: Vec<Vec<[f32; 5]>>,
    pending_updates: HashMap<(u32, u32, usize), f32>,
}

impl PheromoneMap {
    pub fn new(width: u32, height: u32) -> Self {
        PheromoneMap {
            width,
            height,
            data: vec![vec![[0.0; 5]; height as usize]; width as usize],
            pending_updates: HashMap::new(),
        }
    }

    pub fn get_q(&self, x: u32, y: u32, action: Action) -> f32 {
        if x >= self.width || y >= self.height {
            return -1000.0; // Hors map
        }
        self.data[x as usize][y as usize][action.to_usize()]
    }

    // Trouver la meilleure action en évitant les murs et en exploitation de la connaissance
    pub fn get_best_action(&self, x: u32, y: u32, grid: &Grid) -> Action {
        let mut best_action = Action::Stay; // Fallback si bloquée
        let mut max_val = -f32::INFINITY;

        // MODIFICATION : On définit manuellement les actions de mouvement uniquement
        let moving_actions = [Action::Up, Action::Down, Action::Left, Action::Right];

        // On itère sur moving_actions au lieu de Action::all()
        for &action in moving_actions.iter() {
            // Simuler la position de destination pour cette action
            let (nx, ny) = match action {
                Action::Up => (x, y.saturating_sub(1)),
                Action::Down => (x, y + 1),
                Action::Left => (x.saturating_sub(1), y),
                Action::Right => (x + 1, y),
                _ => (x, y), // Cas impossible ici
            };

            // Ignorer si hors map ou mur
            if nx >= self.width || ny >= self.height || !grid.is_walkable(nx, ny) {
                continue;
            }

            let val = self.get_q(x, y, action);

            // Ici on prend strictement supérieur, donc la première action (Up) gagne en cas d'égalité 0
            if val > max_val {
                max_val = val;
                best_action = action;
            }
        }

        // Si max_val est toujours -Infini, c'est qu'elle est enfermée par des murs
        if max_val == -f32::INFINITY {
            return Action::Stay;
        }

        best_action
    }
    // Obtenir la valeur Q maximale de l'état suivant
    pub fn get_max_q(&self, x: u32, y: u32, _grid: &Grid) -> f32 {
        if x >= self.width || y >= self.height {
            return 0.0;
        }
        let mut max_val = -f32::INFINITY;
        for q in self.data[x as usize][y as usize].iter() {
            if *q > max_val {
                max_val = *q;
            }
        }
        if max_val == -f32::INFINITY {
            0.0
        } else {
            max_val
        }
    }

    // Ajouter une modification au buffer sans toucher la grille immédiatement
    pub fn queue_update(&mut self, x: u32, y: u32, action: Action, delta: f32) {
        let key = (x, y, action.to_usize());
        *self.pending_updates.entry(key).or_insert(0.0) += delta;
    }

    // Appliquer tous les changements en attente et appliquer l'évaporation
    pub fn apply_tick(&mut self, evaporation_rate: f32) {
        // Appliquer les mises à jour en attente au tableau de Q-values
        for ((x, y, act_idx), val) in self.pending_updates.drain() {
            self.data[x as usize][y as usize][act_idx] += val;
        }

        // Appliquer l'évaporation à toutes les phéromones
        for col in self.data.iter_mut() {
            for row in col.iter_mut() {
                for val in row.iter_mut() {
                    // Si le taux est 0.01 (1%), on multiplie par 0.99 (99% restant)
                    *val *= 1.0 - evaporation_rate;

                    if val.abs() < 0.001 {
                        *val = 0.0;
                    }
                }
            }
        }
    }
}
