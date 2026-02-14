use crate::tile::Tile;
use crate::tile::TileType;
use rand::Rng;

#[derive(Clone)]
pub struct Grid {
    tiles: Vec<Tile>,
    width: u32,
    height: u32,
}

impl Grid {
    // Grille vide par défaut
    pub fn new(width: u32, height: u32) -> Self {
        let mut tiles = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                tiles.push(Tile::new(x, y, TileType::Default, None));
            }
        }
        Grid {
            tiles,
            width,
            height,
        }
    }

    pub fn new_random(width: u32, height: u32) -> Self {
        // Créer la grille initiale avec toutes les cases en par défaut
        let mut tiles = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                tiles.push(Tile::new(x, y, TileType::Default, None));
            }
        }

        let mut rng = rand::thread_rng();
        let total = width * height;

        // Générer des quantités aléatoires raisonnables pour chaque type d'obstacle
        let food_tiles_number = rng.gen_range(1..4);
        let remaining_after_food = total.saturating_sub(food_tiles_number);
        let wall_tiles_number = if remaining_after_food > 0 {
            rng.gen_range(0..=(total / 4))
        } else {
            0
        };
        let remaining_after_walls = remaining_after_food.saturating_sub(wall_tiles_number);
        let death_tiles_number = if remaining_after_walls > 0 {
            rng.gen_range(0..=(remaining_after_walls / 10))
        } else {
            0
        };

        // Placer le nid à une position aléatoire
        let nest_x = if width > 0 {
            rng.gen_range(0..width)
        } else {
            0
        };
        let nest_y = if height > 0 {
            rng.gen_range(0..height)
        } else {
            0
        };
        let nest_tile = Tile::new(
            nest_x,
            nest_y,
            TileType::Nest {
                stored_food: 0,
                explorer_capacity: rng.gen_range(0..10),
                picker_capacity: rng.gen_range(0..10),
                fighter_capacity: rng.gen_range(0..10),
            },
            None,
        );
        let nest_idx = (nest_y * width + nest_x) as usize;
        tiles[nest_idx] = nest_tile;

        // Placer les tuiles de nourriture avec des quantités aléatoires
        Self::place_items(
            &mut tiles,
            width,
            height,
            food_tiles_number,
            nest_idx,
            TileType::FoodSource { amount: 0 },
        );

        // Placer les murs qui bloquent la circulation
        Self::place_items(
            &mut tiles,
            width,
            height,
            wall_tiles_number,
            nest_idx,
            TileType::Wall,
        );

        // Placer les zones mortelles qui tuent les fourmis
        Self::place_items(
            &mut tiles,
            width,
            height,
            death_tiles_number,
            nest_idx,
            TileType::DeathZone,
        );

        Grid {
            tiles,
            width,
            height,
        }
    }

    fn place_items(
        tiles: &mut Vec<Tile>, // La grille qu'on modifie
        width: u32,
        height: u32,
        count: u32,           // Nombre d'éléments à placer
        forbidden_idx: usize, // L'index du nid pour ne pas y placer d'objets
        item_type: TileType,  // Le type d'élément à placer (mur, zone mortelle, nourriture, etc)
    ) {
        let mut rng = rand::thread_rng();
        let mut placed = 0;

        // Limite de tentatives pour éviter une boucle infinie si la grille est pleine
        let mut attempts = 0;

        while placed < count && attempts < (count * 100) {
            attempts += 1;

            // Choisir une position aléatoire sur la carte
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);
            let idx = (y * width + x) as usize;

            // Vérifier que la position n'est pas le nid
            if idx == forbidden_idx {
                continue;
            }

            // Vérifier que la case est vide (pas encore occupée)
            if matches!(tiles[idx].tile_type, TileType::Default) {
                // Créer la vraie tuile selon le type demandé
                let final_type = match item_type {
                    // Pour la nourriture, générer un montant aléatoire
                    TileType::FoodSource { .. } => TileType::FoodSource {
                        amount: rng.gen_range(100..10000),
                    },
                    // Pour les murs ou la zone de mort, on copie juste le type tel quel
                    TileType::Wall => TileType::Wall,
                    TileType::DeathZone => TileType::DeathZone,

                    // Sinon, utiliser le type par défaut
                    _ => TileType::Default,
                };

                tiles[idx] = Tile::new(x, y, final_type, None);
                placed += 1;
            }
        }
    }

    pub fn print_grid(&self) {
        println!("Grid {}x{}:", self.width, self.height);
        for y in 0..self.height {
            for x in 0..self.width {
                let tile = &self.tiles[(y * self.width + x) as usize];
                let ch = match &tile.tile_type {
                    TileType::Default => '.',
                    TileType::Wall => 'W',
                    TileType::DeathZone => 'X',
                    TileType::FoodSource { .. } => 'F',
                    TileType::Nest { .. } => 'N',
                };
                print!("{}", ch);
            }
            println!();
        }
    }

    pub fn new_with_tiles(width: u32, height: u32, tiles: Vec<Tile>) -> Self {
        // Créer la grille de base avec toutes les cases en défaut
        let mut grid_tiles = Vec::new();
        for y in 0..height {
            for x in 0..width {
                grid_tiles.push(Tile::new(x, y, TileType::Default, None));
            }
        }

        // Placer les tuiles spécifiées à leurs positions respectives
        for tile in tiles {
            let (x, y) = tile.position;
            if x < width && y < height {
                let index = (y * width + x) as usize;
                grid_tiles[index] = tile;
            }
            // Ignorer silencieusement les tuiles hors limites
        }

        Grid {
            tiles: grid_tiles,
            width,
            height,
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_tile(&self, pos: (u32, u32)) -> Option<&Tile> {
        let (x, y) = pos;
        if x < self.width && y < self.height {
            Some(&self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    pub fn get_mut_tile(&mut self, pos: (u32, u32)) -> Option<&mut Tile> {
        let (x, y) = pos;
        if x < self.width && y < self.height {
            Some(&mut self.tiles[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    pub fn get_nest_position(&self) -> Option<(u32, u32)> {
        for tile in &self.tiles {
            if tile.is_nest() {
                return Some(tile.position);
            }
        }
        None
    }

    pub fn get_nest(&self) -> Option<&Tile> {
        let pos = self.get_nest_position()?;
        self.get_tile(pos)
    }

    pub fn get_mut_nest(&mut self) -> Option<&mut Tile> {
        let pos = self.get_nest_position()?;
        self.get_mut_tile(pos)
    }

    pub fn get_food_from_nest(&mut self) -> Option<u32> {
        self.get_nest()?.food_amount()
    }

    pub fn add_food_to_nest(&mut self, amount: u32) {
        self.get_mut_nest()
            .expect("Nest must exist")
            .add_food_to_nest(amount);
    }

    pub fn get_walls_positions(&self) -> Vec<(u32, u32)> {
        let mut walls = Vec::new();
        for tile in &self.tiles {
            if let TileType::Wall = tile.tile_type {
                walls.push(tile.position);
            }
        }
        walls
    }

    pub fn is_walkable(&self, x: u32, y: u32) -> bool {
        // Si la tuile existe, on demande à la tuile. Sinon (hors map), c'est false.
        self.get_tile((x, y))
            .map_or(false, |tile| tile.is_walkable())
    }

    pub fn is_lethal(&self, x: u32, y: u32) -> bool {
        // Si la tuile existe, on demande à la tuile. Sinon (hors map), c'est false.
        self.get_tile((x, y)).map_or(false, |tile| tile.is_lethal())
    }

    pub fn has_food(&self, x: u32, y: u32) -> bool {
        // Si la tuile existe, on demande à la tuile. Sinon (hors map), c'est false.
        self.get_tile((x, y)).map_or(false, |tile| tile.has_food())
    }

    pub fn is_nest(&self, x: u32, y: u32) -> bool {
        // Si la tuile existe, on demande à la tuile. Sinon (hors map), c'est false.
        self.get_tile((x, y)).map_or(false, |tile| tile.is_nest())
    }

    pub fn is_food_remaining(&self) -> bool {
        for tile in &self.tiles {
            if let TileType::FoodSource { amount } = tile.tile_type {
                if amount > 0 {
                    return true;
                }
            }
        }
        false
    }
}
