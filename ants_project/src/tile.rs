#[derive(Clone, Debug, PartialEq)]
pub enum TileType {
    Default,
    Wall,
    Nest {
        stored_food: u32,
        explorer_capacity: u32,
        picker_capacity: u32,
        fighter_capacity: u32,
    },
    FoodSource {
        amount: u32,
    },
    DeathZone,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    pub position: (u32, u32),
    pub tile_type: TileType,
}

impl Tile {
    pub fn new(x: u32, y: u32, tile_type: TileType, food_source: Option<u32>) -> Self {
        // Si un montant de nourriture explicite est fourni, l'utiliser
        if let Some(forced_amount) = food_source {
            return Tile {
                position: (x, y),
                tile_type: TileType::FoodSource {
                    amount: forced_amount,
                },
            };
        }

        // Sinon, utiliser le tile_type passé en paramètre
        // Pour FoodSource, la valeur doit déjà être configurée (générée aléatoirement ailleurs)
        Tile {
            position: (x, y),
            tile_type,
        }
    }

    pub fn food_amount(&self) -> Option<u32> {
        if let TileType::FoodSource { amount } = self.tile_type {
            Some(amount)
        } else {
            None
        }
    }

    pub fn is_walkable(&self) -> bool {
        !matches!(self.tile_type, TileType::Wall)
    }

    pub fn is_lethal(&self) -> bool {
        matches!(self.tile_type, TileType::DeathZone)
    }

    pub fn has_food(&self) -> bool {
        matches!(self.tile_type, TileType::FoodSource { amount } if amount > 0)
    }

    pub fn is_nest(&self) -> bool {
        matches!(self.tile_type, TileType::Nest { .. })
    }

    pub fn add_food_to_nest(&mut self, amount: u32) {
        if let TileType::Nest { stored_food, .. } = &mut self.tile_type {
            *stored_food += amount;
        }
    }
}
