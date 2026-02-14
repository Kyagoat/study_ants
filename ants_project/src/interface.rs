use crate::ant::{Ant, AntsMode, AntsType};
use crate::ants_game_manager::AntsGameManager;
use crate::cli_args::SimulationConfig;
use crate::map_editor::MapEditor;
use crate::pheromone::PheromoneMap;
use eframe::egui;
use std::time::{Duration, Instant};

#[derive(PartialEq)]
enum AppState {
    DimensionInput,
    MapTypeSelection,
    MapEditor,
    AntTypeSelection,
    Game,
}

pub struct Interface {
    state: AppState,
    config: SimulationConfig,
    width_input: String,
    height_input: String,
    ants_game_manager: Option<AntsGameManager>,
    nb_explorers: usize,
    nb_pickers: usize,
    nb_fighters: usize,
    is_running: bool,
    simulation_started: bool,
    last_update: Instant,

    // Paramètres Q-Learning
    alpha_input: String,
    gamma_input: String,
    epsilon_input: String,

    // Éditeur de carte
    map_editor: Option<MapEditor>,

    // Options d'affichage
    show_pheromones_food: bool,
    show_pheromones_nest: bool,
}

impl Interface {
    pub fn new() -> Self {
        Self::new_with_config(SimulationConfig::default())
    }

    pub fn new_with_config(config: SimulationConfig) -> Self {
        Interface {
            state: AppState::DimensionInput,
            width_input: config.grid_width.to_string(),
            height_input: config.grid_height.to_string(),
            ants_game_manager: None,

            nb_explorers: config.num_explorers as usize,
            nb_pickers: config.num_pickers as usize,
            nb_fighters: config.num_fighters as usize,
            is_running: false,
            last_update: Instant::now(),

            alpha_input: config.alpha.to_string(),
            gamma_input: config.gamma.to_string(),
            epsilon_input: config.epsilon.to_string(),

            map_editor: None,

            show_pheromones_food: true,
            show_pheromones_nest: true,
            simulation_started: false,
            config,
        }
    }
}

impl eframe::App for Interface {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Gestion de la boucle de jeu automatique
        if self.state == AppState::Game && self.is_running {
            if self.last_update.elapsed() >= Duration::from_millis(self.config.simulation_speed) {
                if let Some(manager) = &mut self.ants_game_manager {
                    manager.game_step();
                }
                self.last_update = Instant::now();
            }
            ctx.request_repaint();
        }

        match self.state {
            AppState::DimensionInput => self.show_dimension_input(ctx),
            AppState::MapTypeSelection => self.show_map_type_selection(ctx),
            AppState::MapEditor => self.show_map_editor_screen(ctx),
            AppState::AntTypeSelection => self.show_ant_type_selection(ctx),
            AppState::Game => self.show_game(ctx),
        }
    }
}

impl Interface {
    fn show_dimension_input(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("Définir les dimensions de la grille");
                ui.add_space(30.0);

                ui.label("Largeur :");
                ui.text_edit_singleline(&mut self.width_input);

                ui.add_space(20.0);

                ui.label("Hauteur :");
                ui.text_edit_singleline(&mut self.height_input);

                ui.add_space(30.0);

                if ui
                    .button(egui::RichText::new("Continuer").size(18.0))
                    .clicked()
                {
                    if let (Ok(width), Ok(height)) = (
                        self.width_input.parse::<u32>(),
                        self.height_input.parse::<u32>(),
                    ) {
                        if width > 0 && height > 0 {
                            self.state = AppState::MapTypeSelection;
                        } else {
                            eprintln!("Les dimensions doivent être supérieures à 0");
                        }
                    } else {
                        eprintln!("Veuillez entrer des nombres valides");
                    }
                }
            });
        });
    }

    fn show_map_type_selection(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("Sélection du type de map");
                ui.add_space(30.0);

                if ui
                    .button(egui::RichText::new("Map Aléatoire").size(20.0))
                    .clicked()
                {
                    let width = self.width_input.parse::<u32>().unwrap_or(20);
                    let height = self.height_input.parse::<u32>().unwrap_or(20);

                    // Mise à jour de la config
                    self.config.grid_width = width;
                    self.config.grid_height = height;

                    // IMPORTANT : On met à None pour signaler plus tard qu'il faudra générer du random
                    self.ants_game_manager = None;

                    self.state = AppState::AntTypeSelection;
                }

                ui.add_space(20.0);

                if ui
                    .button(egui::RichText::new("Map Personnalisée").size(20.0))
                    .clicked()
                {
                    let width = self.width_input.parse::<u32>().unwrap_or(20);
                    let height = self.height_input.parse::<u32>().unwrap_or(20);

                    // Mise à jour config
                    self.config.grid_width = width;
                    self.config.grid_height = height;

                    self.map_editor = Some(crate::map_editor::MapEditor::new(width, height));
                    self.state = AppState::MapEditor;
                }

                ui.add_space(20.0);
                ui.separator();

                if ui.button("Retour aux Dimensions").clicked() {
                    self.state = AppState::DimensionInput;
                }
            });
        });
    }

    fn show_ant_type_selection(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.heading("Configuration de la Colonie");
                ui.add_space(30.0);

                // Sliders pour le nombre de fourmis
                ui.group(|ui| {
                    ui.heading("Explorateurs");
                    ui.add(egui::Slider::new(&mut self.nb_explorers, 0..=50));
                });
                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.heading("Récolteuses");
                    ui.add(egui::Slider::new(&mut self.nb_pickers, 0..=50));
                });
                ui.add_space(10.0);

                ui.group(|ui| {
                    ui.heading("Combattantes");
                    ui.add(egui::Slider::new(&mut self.nb_fighters, 0..=50));
                });

                ui.add_space(30.0);

                let total = self.nb_explorers + self.nb_pickers + self.nb_fighters;
                ui.label(format!("Total: {} fourmis", total));

                ui.add_space(20.0);

                if ui.button("Vers le Plateau de Jeu").clicked() {
                    // 1. Mise à jour de la configuration avec les sliders actuels
                    self.config.num_explorers = self.nb_explorers as u32;
                    self.config.num_pickers = self.nb_pickers as u32;
                    self.config.num_fighters = self.nb_fighters as u32;

                    // 2. Génération des fourmis
                    let ants = self.generate_ants();

                    // 3. Gestion du Manager (Création ou Mise à jour)
                    let mut manager =
                        if let Some(mut existing_manager) = self.ants_game_manager.take() {
                            // CAS 1 : Map Éditeur (Le manager existe déjà avec la grille)
                            existing_manager.ants = ants;

                            for ant in &mut existing_manager.ants {
                                ant.spawn_at_nest(&existing_manager.grid);
                            }

                            existing_manager.config = self.config.clone();
                            existing_manager
                        } else {
                            AntsGameManager::new_game_mode_random(
                                self.config.grid_width,
                                self.config.grid_height,
                                ants,
                                self.config.clone(),
                            )
                        };

                    // 4. Synchronisation initiale des paramètres Q-Learning
                    manager.rl_params.alpha = self.config.alpha;
                    manager.rl_params.gamma = self.config.gamma;
                    manager.rl_params.epsilon = self.config.epsilon;

                    self.ants_game_manager = Some(manager);

                    // 5. Transition vers le jeu
                    self.state = AppState::Game;
                    self.is_running = false; // Pause au démarrage
                    self.simulation_started = false; // Paramètres déverrouillés
                }

                ui.add_space(10.0);
                if ui.button("Retour").clicked() {
                    self.state = AppState::MapTypeSelection;
                }
            });
        });
    }

    fn generate_ants(&self) -> Vec<Ant> {
        let mut ants = Vec::new();

        // Générer les Explorateurs
        for _ in 0..self.nb_explorers {
            ants.push(Ant::new(AntsType::EXPLORER));
        }

        // Générer les Pickers
        for _ in 0..self.nb_pickers {
            ants.push(Ant::new(AntsType::PICKER));
        }

        // Générer les Fighters
        for _ in 0..self.nb_fighters {
            ants.push(Ant::new(AntsType::FIGHTER));
        }

        ants
    }

    fn show_map_editor_screen(&mut self, ctx: &egui::Context) {
        enum EditorAction {
            None,
            Launch {
                width: u32,
                height: u32,
                tiles: Vec<crate::tile::Tile>,
            },
            GoBack,
        }

        let mut action = EditorAction::None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Éditeur de Carte");
            ui.separator();

            if let Some(editor) = &mut self.map_editor {
                let auto_launch = crate::map_editor::show_map_editor(ui, editor, 30.0);
                let mut manual_launch = false;

                ui.separator();
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button("✓ Confirmer la Carte").clicked() {
                        manual_launch = true;
                    }

                    if ui.button("← Retour au Choix").clicked() {
                        action = EditorAction::GoBack;
                    }
                });

                if auto_launch || manual_launch {
                    // On extrait les données maintenant pour pouvoir fermer l'éditeur ensuite
                    action = EditorAction::Launch {
                        width: editor.width,
                        height: editor.height,
                        tiles: editor.to_tiles(),
                    };
                }
            }
        });

        match action {
            EditorAction::Launch {
                width,
                height,
                tiles,
            } => {
                let game_manager =
                    AntsGameManager::new(width, height, tiles, vec![], self.config.clone());
                self.ants_game_manager = Some(game_manager);
                self.map_editor = None;
                self.state = AppState::AntTypeSelection;
            }
            EditorAction::GoBack => {
                self.state = AppState::MapTypeSelection;
                self.map_editor = None;
            }
            EditorAction::None => {}
        }
    }

    fn show_game(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("controls_panel")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("Configuration & Contrôle");
                    });
                    ui.separator();

                    // Afficher les contrôles de lancement et pause de la simulation
                    ui.group(|ui| {
                        ui.heading("État de la Simulation");
                        ui.add_space(5.0);

                        if !self.simulation_started {
                            // Avant le lancement, montrer le bouton de démarrage en vert
                            ui.label("Prêt à démarrer. Réglez les paramètres ci-dessous.");
                            if ui
                                .button(
                                    egui::RichText::new("LANCER LA SIMULATION")
                                        .size(20.0)
                                        .color(egui::Color32::GREEN),
                                )
                                .clicked()
                            {
                                self.simulation_started = true;
                                self.is_running = true;
                            }
                        } else {
                            // Une fois lancée, afficher seulement les boutons pause/reprise
                            let btn_text = if self.is_running {
                                "PAUSE"
                            } else {
                                "REPRENDRE"
                            };
                            if ui
                                .button(egui::RichText::new(btn_text).size(20.0))
                                .clicked()
                            {
                                self.is_running = !self.is_running;
                            }
                            ui.label(
                                egui::RichText::new("Paramètres verrouillés")
                                    .color(egui::Color32::RED)
                                    .small(),
                            );
                        }

                        ui.add_space(10.0);
                        // La vitesse d'affichage reste modifiable même en jeu
                        ui.label("Vitesse (Calculs/Image) :");
                        ui.add(
                            egui::Slider::new(&mut self.config.simulation_speed, 1..=100)
                                .logarithmic(true),
                        );
                    });

                    if let Some(manager) = &mut self.ants_game_manager {
                        if !manager.history.is_empty() {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.heading("Timeline (Rewind)");

                            let max_tick = manager.history.len() - 1;
                            let mut current = manager.current_tick_index;

                            ui.label(format!("Tick: {} / {}", current, max_tick));

                            let slider =
                                ui.add(egui::Slider::new(&mut current, 0..=max_tick).text("Temps"));

                            // Si on bouge le slider, on met à jour et on pause
                            if slider.changed() {
                                self.is_running = false;
                                manager.restore_snapshot(current);
                            }

                            ui.horizontal(|ui| {
                                if ui.button("<< -1").clicked() && current > 0 {
                                    self.is_running = false;
                                    manager.restore_snapshot(current - 1);
                                }
                                if ui.button("+1 >>").clicked() && current < max_tick {
                                    self.is_running = false;
                                    manager.restore_snapshot(current + 1);
                                }
                            });
                        }
                    }

                    ui.add_space(10.0);

                    // Déterminer si les sliders peuvent être activés
                    let params_enabled = !self.simulation_started;

                    // Afficher les paramètres d'apprentissage du Q-Learning
                    ui.collapsing("Cerveau (Q-Learning)", |ui| {
                        ui.add_enabled_ui(params_enabled, |ui| {
                            ui.label("Alpha (Apprentissage) :");
                            ui.add(egui::Slider::new(&mut self.config.alpha, 0.0..=1.0));

                            ui.separator();
                            ui.label("Gamma (Vision) :");
                            ui.add(egui::Slider::new(&mut self.config.gamma, 0.0..=1.0));

                            ui.separator();
                            ui.label("Epsilon (Exploration) :");
                            ui.add(egui::Slider::new(&mut self.config.epsilon, 0.0..=1.0));
                        });
                    });

                    ui.add_space(10.0);

                    // Afficher les sliders pour configurer les récompenses
                    ui.collapsing("Récompenses", |ui| {
                        ui.add_enabled_ui(params_enabled, |ui| {
                            ui.label("Nourriture (+):");
                            ui.add(egui::Slider::new(
                                &mut self.config.reward_food,
                                100.0..=5000.0,
                            ));

                            ui.label("Retour Nid (+):");
                            ui.add(egui::Slider::new(
                                &mut self.config.reward_nest,
                                100.0..=5000.0,
                            ));

                            ui.separator();

                            ui.label("Coût Déplacement (-):");
                            ui.add(egui::Slider::new(
                                &mut self.config.reward_default,
                                -5.0..=0.0,
                            ));

                            ui.label("Mort (-):");
                            ui.add(egui::Slider::new(
                                &mut self.config.reward_death,
                                -500.0..=-10.0,
                            ));
                        });
                    });

                    ui.add_space(10.0);

                    // Afficher les options de visualisation
                    ui.collapsing("Visualisation", |ui| {
                        ui.checkbox(&mut self.show_pheromones_food, "Pistes Nourriture");
                        ui.checkbox(&mut self.show_pheromones_nest, "Pistes Retour");
                    });

                    ui.add_space(20.0);
                    ui.separator();

                    if ui.button("Quitter / Reset").clicked() {
                        self.state = AppState::DimensionInput;
                        self.ants_game_manager = None;
                        self.is_running = false;
                        self.simulation_started = false;
                    }
                });
            });

        // Sync config vers manager
        if !self.simulation_started {
            if let Some(manager) = &mut self.ants_game_manager {
                manager.config = self.config.clone();
                manager.rl_params.alpha = self.config.alpha;
                manager.rl_params.gamma = self.config.gamma;
                manager.rl_params.epsilon = self.config.epsilon;
            }
        }

        // Zone de dessin
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(game_manager) = &self.ants_game_manager {
                self.draw_board(ui, game_manager);
            }
        });
    }

    fn draw_board(&self, ui: &mut egui::Ui, game_manager: &AntsGameManager) {
        let grid = &game_manager.grid;
        let available_size = ui.available_size();

        if available_size.x <= 0.0 || available_size.y <= 0.0 {
            return;
        }

        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::hover());

        let width = grid.get_width() as f32;
        let height = grid.get_height() as f32;
        let cell_size = (available_size.x / width)
            .min(available_size.y / height)
            .min(50.0);

        let offset_x = response.rect.min.x + (available_size.x - width * cell_size) / 2.0;
        let offset_y = response.rect.min.y + (available_size.y - height * cell_size) / 2.0;

        self.draw_grid_base(&painter, grid, offset_x, offset_y, cell_size);

        if self.show_pheromones_food {
            self.draw_pheromones(
                &painter,
                &game_manager.pheromones_food,
                grid,
                offset_x,
                offset_y,
                cell_size,
                egui::Color32::from_rgb(139, 69, 19),
            );
        }
        if self.show_pheromones_nest {
            self.draw_pheromones(
                &painter,
                &game_manager.pheromones_nest,
                grid,
                offset_x,
                offset_y,
                cell_size,
                egui::Color32::from_rgb(255, 105, 180),
            );
        }

        self.draw_grid_objects(&painter, grid, offset_x, offset_y, cell_size);
        self.draw_ants(&painter, game_manager, offset_x, offset_y, cell_size);
    }

    fn draw_grid_base(
        &self,
        painter: &egui::Painter,
        grid: &crate::grid::Grid,
        off_x: f32,
        off_y: f32,
        size: f32,
    ) {
        let grid_w = grid.get_width() as f32 * size;
        let grid_h = grid.get_height() as f32 * size;
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(off_x, off_y), egui::Vec2::new(grid_w, grid_h)),
            0.0,
            egui::Color32::from_gray(30),
        );

        for y in 0..grid.get_height() {
            for x in 0..grid.get_width() {
                let rect = egui::Rect::from_min_size(
                    egui::pos2(off_x + x as f32 * size, off_y + y as f32 * size),
                    egui::Vec2::new(size, size),
                )
                .shrink(1.0);

                if let Some(tile) = grid.get_tile((x, y)) {
                    match tile.tile_type {
                        crate::tile::TileType::Wall => {
                            painter.rect_filled(rect, 2.0, egui::Color32::GRAY);
                        }
                        crate::tile::TileType::DeathZone => {
                            painter.rect_filled(rect, 0.0, egui::Color32::from_rgb(50, 0, 0));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw_pheromones(
        &self,
        painter: &egui::Painter,
        map: &PheromoneMap,
        grid: &crate::grid::Grid,
        off_x: f32,
        off_y: f32,
        size: f32,
        base_color: egui::Color32,
    ) {
        const MAX_EXPECTED_VALUE: f32 = 50.0;

        for y in 0..map.height {
            for x in 0..map.width {
                if !grid.is_walkable(x, y) {
                    continue;
                }

                let max_q = map.get_max_q(x, y, grid).max(0.0);

                if max_q > 0.1 {
                    let ratio = (max_q / MAX_EXPECTED_VALUE).clamp(0.0, 1.0);
                    let visual_intensity = ratio.sqrt();
                    let alpha = (visual_intensity * 200.0) as u8;

                    let rect = egui::Rect::from_min_size(
                        egui::pos2(off_x + x as f32 * size, off_y + y as f32 * size),
                        egui::Vec2::new(size, size),
                    );

                    let color = egui::Color32::from_rgba_unmultiplied(
                        base_color.r(),
                        base_color.g(),
                        base_color.b(),
                        alpha,
                    );

                    painter.rect_filled(rect, 0.0, color);
                }
            }
        }
    }

    fn draw_grid_objects(
        &self,
        painter: &egui::Painter,
        grid: &crate::grid::Grid,
        off_x: f32,
        off_y: f32,
        size: f32,
    ) {
        for y in 0..grid.get_height() {
            for x in 0..grid.get_width() {
                let center = egui::pos2(
                    off_x + x as f32 * size + size / 2.0,
                    off_y + y as f32 * size + size / 2.0,
                );

                if let Some(tile) = grid.get_tile((x, y)) {
                    match tile.tile_type {
                        crate::tile::TileType::Nest { stored_food, .. } => {
                            painter.rect_filled(
                                egui::Rect::from_center_size(center, egui::Vec2::splat(size * 0.5)),
                                2.0,
                                egui::Color32::GOLD,
                            );

                            let text = format!("Nid\n{}", stored_food);
                            painter.text(
                                center,
                                egui::Align2::CENTER_CENTER,
                                text,
                                egui::FontId::monospace(8.0),
                                egui::Color32::BLACK,
                            );
                        }
                        crate::tile::TileType::FoodSource { amount } => {
                            painter.circle_filled(center, size * 0.35, egui::Color32::GREEN);
                            painter.circle_stroke(
                                center,
                                size * 0.35,
                                egui::Stroke::new(2.0, egui::Color32::WHITE),
                            );

                            if amount > 0 {
                                let text = if amount > 999 {
                                    format!("{}k", amount / 1000)
                                } else {
                                    format!("{}", amount)
                                };
                                painter.text(
                                    center,
                                    egui::Align2::CENTER_CENTER,
                                    text,
                                    egui::FontId::monospace(7.0),
                                    egui::Color32::WHITE,
                                );
                            } else {
                                painter.text(
                                    center,
                                    egui::Align2::CENTER_CENTER,
                                    "0",
                                    egui::FontId::monospace(8.0),
                                    egui::Color32::LIGHT_GRAY,
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn draw_ants(
        &self,
        painter: &egui::Painter,
        manager: &AntsGameManager,
        off_x: f32,
        off_y: f32,
        size: f32,
    ) {
        for ant in &manager.ants {
            if let Some((x, y)) = ant.position {
                let center = egui::pos2(
                    off_x + x as f32 * size + size / 2.0,
                    off_y + y as f32 * size + size / 2.0,
                );

                let color = match ant.mode {
                    AntsMode::FINDING => egui::Color32::WHITE,
                    AntsMode::RETURNING => egui::Color32::YELLOW,
                };

                painter.circle_filled(center, size * 0.25, color);
                painter.circle_stroke(
                    center,
                    size * 0.25,
                    egui::Stroke::new(1.0, egui::Color32::BLACK),
                );

                if ant.current_charge > 0 {
                    painter.circle_filled(
                        egui::pos2(center.x + size * 0.15, center.y - size * 0.15),
                        size * 0.1,
                        egui::Color32::GREEN,
                    );
                }
            }
        }
    }
}
