use crate::tile::{Tile, TileType};
use eframe::egui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapEditorTileType {
    Default,
    Wall,
    Nest,
    FoodSource,
    DeathZone,
}

impl MapEditorTileType {
    pub fn to_tile_type(&self) -> TileType {
        match self {
            MapEditorTileType::Default => TileType::Default,
            MapEditorTileType::Wall => TileType::Wall,
            MapEditorTileType::Nest => TileType::Nest {
                stored_food: 0,
                explorer_capacity: 10,
                picker_capacity: 10,
                fighter_capacity: 10,
            },
            MapEditorTileType::FoodSource => TileType::FoodSource { amount: 1000 },
            MapEditorTileType::DeathZone => TileType::DeathZone,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MapEditorTileType::Default => "Vide",
            MapEditorTileType::Wall => "Mur",
            MapEditorTileType::Nest => "Nid",
            MapEditorTileType::FoodSource => "Nourriture",
            MapEditorTileType::DeathZone => "Danger",
        }
    }

    pub fn color(&self) -> egui::Color32 {
        match self {
            MapEditorTileType::Default => egui::Color32::from_gray(40), // Un peu plus foncé
            MapEditorTileType::Wall => egui::Color32::GRAY,
            MapEditorTileType::Nest => egui::Color32::GOLD,
            MapEditorTileType::FoodSource => egui::Color32::GREEN,
            MapEditorTileType::DeathZone => egui::Color32::from_rgb(139, 0, 0),
        }
    }

    pub fn all() -> impl Iterator<Item = MapEditorTileType> {
        [
            MapEditorTileType::Default,
            MapEditorTileType::Wall,
            MapEditorTileType::Nest,
            MapEditorTileType::FoodSource,
            MapEditorTileType::DeathZone,
        ]
        .iter()
        .copied()
    }
}

pub struct MapEditor {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Vec<MapEditorTileType>>,
    pub selected_tile_type: MapEditorTileType,
    pub nest_count: u32,
}

impl MapEditor {
    pub fn new(width: u32, height: u32) -> Self {
        let tiles = vec![vec![MapEditorTileType::Default; width as usize]; height as usize];
        MapEditor {
            width,
            height,
            tiles,
            selected_tile_type: MapEditorTileType::Wall, // Wall par défaut, plus pratique
            nest_count: 0,
        }
    }

    pub fn set_tile(&mut self, x: u32, y: u32, tile_type: MapEditorTileType) {
        if x < self.width && y < self.height {
            let current = self.tiles[y as usize][x as usize];

            // Gestion du compteur de nids
            if current == MapEditorTileType::Nest && tile_type != MapEditorTileType::Nest {
                self.nest_count = self.nest_count.saturating_sub(1);
            }
            if current != MapEditorTileType::Nest && tile_type == MapEditorTileType::Nest {
                self.nest_count += 1;
            }

            self.tiles[y as usize][x as usize] = tile_type;
        }
    }

    pub fn to_tiles(&self) -> Vec<Tile> {
        let mut tiles = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let tile_type = self.tiles[y as usize][x as usize].to_tile_type();
                tiles.push(Tile::new(x, y, tile_type, None));
            }
        }
        tiles
    }

    pub fn fill_all(&mut self, tile_type: MapEditorTileType) {
        self.nest_count = 0;
        for y in 0..self.height {
            for x in 0..self.width {
                if tile_type == MapEditorTileType::Nest {
                    if self.nest_count == 0 {
                        self.set_tile(x, y, tile_type);
                    }
                } else {
                    self.set_tile(x, y, tile_type);
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.fill_all(MapEditorTileType::Default);
    }

    pub fn is_valid(&self) -> bool {
        if self.nest_count != 1 {
            return false;
        }
        let has_food = self
            .tiles
            .iter()
            .any(|row| row.iter().any(|&t| t == MapEditorTileType::FoodSource));
        has_food
    }

    pub fn get_validation_error(&self) -> Option<String> {
        if self.nest_count == 0 {
            return Some("❌ Placez 1 NID (case jaune)".to_string());
        }
        if self.nest_count > 1 {
            return Some(format!("❌ Trop de NIDS ({}/1)", self.nest_count));
        }

        let has_food = self
            .tiles
            .iter()
            .any(|row| row.iter().any(|&t| t == MapEditorTileType::FoodSource));
        if !has_food {
            return Some("❌ Placez de la NOURRITURE (case verte)".to_string());
        }

        None
    }
}

pub fn show_map_editor(ui: &mut egui::Ui, editor: &mut MapEditor, _base_cell_size: f32) -> bool {
    let mut launch_clicked = false;

    // 1. BARRE D'OUTILS EN HAUT
    ui.horizontal(|ui_inner| {
        ui_inner.label("Outils :");
        if ui_inner.button("🗑️ Tout effacer").clicked() {
            editor.clear();
        }
        if ui_inner.button("⬜ Remplir vide").clicked() {
            editor.fill_all(MapEditorTileType::Default);
        }
    });
    ui.separator();

    // 2. SÉLECTION DU TYPE DE TUILE
    ui.horizontal_wrapped(|ui_inner| {
        for tile_type in MapEditorTileType::all() {
            let is_selected = editor.selected_tile_type == tile_type;
            let button = egui::Button::new(tile_type.label())
                .fill(tile_type.color())
                .stroke(if is_selected {
                    egui::Stroke::new(2.0, egui::Color32::WHITE)
                } else {
                    egui::Stroke::NONE
                });

            if ui_inner.add(button).clicked() {
                editor.selected_tile_type = tile_type;
            }
        }
    });
    ui.separator();

    // 3. ZONE BASSE (BOUTON LANCER + VALIDATION)
    // On utilise TopBottomPanel ou simplement ui.vertical pour pousser le bouton en bas
    // Mais ici, l'astuce egui est d'allouer l'espace restant pour la grille APRES avoir dessiné le bas.
    // Pour faire simple : on utilise un layout "bottom_up" pour dessiner le bouton d'abord en bas.

    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui_bottom| {
        ui_bottom.add_space(10.0);

        // --- BOUTON LANCER (Dessiné tout en bas) ---
        if editor.is_valid() {
            // Gros bouton vert
            let btn = egui::Button::new(
                egui::RichText::new("🚀 LANCER LA PARTIE")
                    .size(20.0)
                    .strong(),
            )
            .fill(egui::Color32::from_rgb(0, 100, 0))
            .min_size(egui::vec2(200.0, 40.0));

            if ui_bottom.add(btn).clicked() {
                launch_clicked = true;
            }
        } else {
            // Bouton gris désactivé avec la raison
            let err = editor.get_validation_error().unwrap_or_default();
            ui_bottom.add_enabled(
                false,
                egui::Button::new(err).min_size(egui::vec2(200.0, 40.0)),
            );
        }

        ui_bottom.add_space(10.0);

        // Stats juste au-dessus du bouton
        ui_bottom.label(format!("Nids: {}/1", editor.nest_count));
        ui_bottom.separator();

        // 4. GRILLE CENTRALE (Prend tout l'espace restant au-dessus du bouton)
        // On repasse en layout normal pour dessiner la grille
        // ATTENTION: ui_bottom est un UI temporaire. On doit revenir au scope parent ou utiliser allocate_ui_at_rect.
        // Mais `with_layout` consomme l'espace. La meilleure façon est d'utiliser `allocate_response` manuellement
        // ou simplement de dessiner la grille dans l'espace restant "available_rect_before_wrap".

        let available_size = ui_bottom.available_size(); // L'espace restant AU DESSUS du bouton
        if available_size.x <= 0.0 || available_size.y <= 0.0 {
            return;
        }

        let (mut response, painter) =
            ui_bottom.allocate_painter(available_size, egui::Sense::click_and_drag());

        // --- CALCUL DU CENTRAGE ---
        let width_f = editor.width as f32;
        let height_f = editor.height as f32;

        // Taille des cases auto-adaptative pour tout faire tenir
        let cell_size = (available_size.x / width_f)
            .min(available_size.y / height_f)
            .min(40.0); // Max 40px pour pas que ce soit énorme

        // Calcul des offsets pour centrer
        let grid_w_px = width_f * cell_size;
        let grid_h_px = height_f * cell_size;
        let offset_x = response.rect.min.x + (available_size.x - grid_w_px) / 2.0;
        let offset_y = response.rect.min.y + (available_size.y - grid_h_px) / 2.0;

        // Dessin du fond de la grille
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(offset_x, offset_y),
                egui::Vec2::new(grid_w_px, grid_h_px),
            ),
            0.0,
            egui::Color32::from_gray(20),
        );

        // Dessin des cases
        for y in 0..editor.height {
            for x in 0..editor.width {
                let rect = egui::Rect::from_min_size(
                    egui::pos2(
                        offset_x + x as f32 * cell_size,
                        offset_y + y as f32 * cell_size,
                    ),
                    egui::Vec2::splat(cell_size),
                )
                .shrink(1.0);

                let tile = editor.tiles[y as usize][x as usize];
                painter.rect_filled(rect, 2.0, tile.color());
            }
        }

        // --- GESTION DES CLICS / DESSIN ---
        // On permet de cliquer OU de glisser
        if response.clicked() || (response.dragged() && response.is_pointer_button_down_on()) {
            if let Some(pos) = response.interact_pointer_pos() {
                // On inverse la logique pour trouver la case
                let rel_x = pos.x - offset_x;
                let rel_y = pos.y - offset_y;

                if rel_x >= 0.0 && rel_y >= 0.0 {
                    let grid_x = (rel_x / cell_size).floor() as u32;
                    let grid_y = (rel_y / cell_size).floor() as u32;

                    // Sécurité bornes
                    if grid_x < editor.width && grid_y < editor.height {
                        editor.set_tile(grid_x, grid_y, editor.selected_tile_type);
                        response.mark_changed(); // Indique à egui de redessiner vite
                    }
                }
            }
        }
    });

    launch_clicked
}
