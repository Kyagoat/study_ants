use crate::grid::Grid;
use crate::pheromone::Action;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AntsType {
    EXPLORER,
    FIGHTER,
    PICKER,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AntsMode {
    FINDING,
    RETURNING,
}

#[derive(Clone, Debug)]
pub struct Ant {
    pub ant_type: AntsType,
    pub maximal_charge: u32,
    pub current_charge: u32,
    pub seconds_for_movement: u32,
    pub cooldown: u32,
    pub scope: u32,
    pub mode: AntsMode,
    pub position: Option<(u32, u32)>,
}

impl Ant {
    pub fn new(ant_type: AntsType) -> Self {
        // Valeur initiale de charge pour toutes les fourmis
        const DEFAULT_CHARGE: u32 = 0;
        let (max_charge, speed, scope) = match ant_type {
            AntsType::EXPLORER => (10, 5, 1),
            AntsType::FIGHTER => (10, 5, 1),
            AntsType::PICKER => (100, 10, 0),
        };
        Ant {
            ant_type,
            maximal_charge: max_charge,
            current_charge: DEFAULT_CHARGE,
            seconds_for_movement: speed,
            scope,
            mode: AntsMode::FINDING,
            position: None,
            cooldown: 0,
        }
    }

    pub fn get_target_position(&self, action: Action) -> (u32, u32) {
        // Utiliser (0,0) comme position par défaut si la fourmi n'est pas encore sur la carte
        let (x, y) = self.position.unwrap_or((0, 0));

        match action {
            Action::Up => (x, y.saturating_sub(1)), // Éviter de déborder vers le haut (y négatif)
            Action::Down => (x, y + 1), // La vérification de la limite haute est faite par le Manager
            Action::Left => (x.saturating_sub(1), y), // Éviter de déborder vers la gauche
            Action::Right => (x + 1, y), // La vérification de la limite droite est faite par le Manager
            Action::Stay => (x, y),
        }
    }

    pub fn move_to(&mut self, x: u32, y: u32) {
        self.position = Some((x, y));
    }

    pub fn spawn_at_nest(&mut self, grid: &Grid) {
        if let Some(nest_pos) = grid.get_nest_position() {
            self.position = Some(nest_pos);
        }
    }
}
