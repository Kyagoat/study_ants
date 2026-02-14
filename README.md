# Ant Colony Simulation

An advanced ant colony simulator built in Rust featuring reinforcement learning, pheromone systems, and graphical simulation interface.

## Overview

This project implements a multi-agent simulation system where ants navigate a grid environment, explore for food, and learn optimal paths using Q-Learning. The simulation includes dynamic pheromone trails, environmental obstacles, and configurable learning parameters.

### Key Features

- **Ant Movement & Pathfinding**: Ants explore the grid and build paths toward food sources
- **Pheromone System**: Dynamic pheromone trails that evaporate over time guide ant behavior
- **Dual Q-Learning Tables**: Separate Q-tables for exploration and exploitation strategies
- **Food Collection**: Ants gather food from designated zones and return to the nest
- **Graphical Map Editor**: Built-in visual interface to design custom environments (walls, nest, food zones, danger areas)
- **Dynamic Parameters**: Adjust learning hyperparameters (Alpha, Gamma, Epsilon) during simulation
- **Dual Mode Execution**: Run via graphical interface (GUI) or command-line interface (CLI)
- **Timeline System**: Save and track simulation snapshots for analysis
- **Meta project**: Meta project that automaticaly adjust hyperparameters to optimize the simulation

### Algorithm Complexity

- **Main Simulation Loop**: O(N²) - Grid traversal and pheromone dissipation
- **Pheromone Evaporation**: O(N²) - Full grid update per tick
- **Timeline Snapshots**: O(N²) - Complete grid duplication
- **Meta-Optimization**: O(N) - Parameter combination iteration
- **Q-Learning Updates**: O(1) - Direct table lookup and arithmetic

## Requirements

- **Rust 1.70+** and **Cargo**
- **Visual Studio Build Tools** (Windows) or equivalent C++ build tools
- Compatible with: Windows, Linux (Ubuntu), macOS

## Installation

### Prerequisites

Ensure Rust and Cargo are installed. If not, install from [rustup.rs](https://rustup.rs/)

### Clone the Repository

```bash
git clone <git@github.com:Kyagoat/study_ants.git>
cd fourmis_darona_mehdi-main/ants_project
```

## How to Launch

All commands assume you're in the `ants_project` directory.

### Standard Launch (With GUI)

Opens the graphical interface for configuring simulation parameters:

```bash
cargo run --release
```

The GUI allows you to:
- Set grid dimensions (width, height)
- Specify number of ants
- Configure learning parameters
- Design the environment using the map editor

### Launch with CLI Parameters

Pass parameters directly to pre-configure the simulation:

```bash
cargo run --release -- [OPTIONS]
```

**Example:**

```bash
cargo run --release -- --width 50 --height 50 --explorers 20 --alpha 0.1 --gamma 0.9
```

### CLI Mode (No GUI)

Run simulation in headless mode:

```bash
cargo run --release -- --cli --width 40 --height 40 --explorers 15
```

## Available Arguments

### Display Options
- `--cli`: Disable GUI, run in command-line mode

### Grid Configuration
- `--width <N>`: Grid width (default: 50)
- `--height <N>`: Grid height (default: 50)

### Ant Configuration
- `--explorers <N>`: Number of explorer ants (default: 10)

### Learning Parameters
- `--alpha <F>`: Learning rate (default: 0.1, range: 0.0-1.0)
- `--gamma <F>`: Discount factor (default: 0.9, range: 0.0-1.0)
- `--epsilon <F>`: Exploration rate (default: 0.1, range: 0.0-1.0)

### Example Configurations

Standard exploration setup:
```bash
cargo run --release -- --width 100 --height 100 --explorers 50
```

Aggressive learning:
```bash
cargo run --release -- --alpha 0.5 --gamma 0.95 --epsilon 0.2
```

Conservative learning:
```bash
cargo run --release -- --alpha 0.05 --gamma 0.8 --epsilon 0.05
```

## Building for Release

Optimized build for performance:

```bash
cargo build --release
```

The compiled binary will be in `target/release/`.

## Troubleshooting

### Build Fails on Windows
Ensure Visual Studio Build Tools with C++ support is installed. See the main requirements section.

### Slow GUI Rendering
Reduce grid size or number of ants, or adjust simulation speed in the GUI.

### Ants Not Converging
Verify learning parameters (Alpha, Gamma, Epsilon) are properly set and simulation has run long enough.

## Authors

- Kylian MEHDI
- Lucas DARONA

## Implementation Notes

The simulation uses a discrete time-step model where each frame represents one logical time unit. Pheromones dissipate according to a configurable model, and ant decisions use ε-greedy Q-Learning with separate Q-tables for different behavioral strategies.
