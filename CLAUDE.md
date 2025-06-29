# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Lunar Lander game implemented in Rust using the macroquad game engine. The game simulates realistic lunar physics including gravity, momentum, fuel consumption, and rocket physics using Tsiolkovsky's equation.

## Core Dependencies

- **macroquad**: Game engine for rendering, input, and window management (local path dependency)
- **macroquad-text**: Text rendering library (local path dependency) 
- **rusty_audio**: Audio playback system (local path dependency)
- **noise**: Procedural terrain generation using Perlin noise
- **plotters**: Graphics and plotting utilities
- **rand**: Random number generation

## Development Commands

### Building and Running
```bash
cargo build          # Compile the project
cargo run            # Build and run the game
cargo run --release  # Run optimized build
```

### Testing
```bash
cargo test                    # Run all tests
cargo test test_name         # Run specific test
cargo test -- --nocapture   # Run tests with output
```

### Other Useful Commands
```bash
cargo check    # Fast compile check without building binaries
cargo clean     # Remove build artifacts
cargo doc       # Generate documentation
```

## Architecture

### Modular Design (Updated 2024)
The codebase has been refactored for better maintainability:

**Modules:**
- `src/main.rs`: Core game loop, entity management, rendering, input
- `src/physics.rs`: Advanced rocket physics with proper Tsiolkovsky equation implementation  
- `src/surface.rs`: Procedural terrain generation

**Components:**
- `Transform`: Position, size, rotation
- `Physics`: Velocity and acceleration (`src/physics.rs`)
- `RocketPhysics`: Realistic rocket parameters and fuel management (`src/physics.rs`)
- `Renderer`: Texture and rendering properties
- `Collision`: Collision detection

**Entity:**
- `Entity` struct contains all components plus game-specific data like terrain, fonts

**Systems:**
- `update_physics()`: Proper physics integration with realistic timestep
- `update_rocket_physics()`: Advanced rocket thrust and fuel consumption (`src/physics.rs`)
- `render()`: Draws all visual elements with thrust-based texture selection
- `handle_input()`: Enhanced input handling with proper thrust management
- `check_collision()`: Terrain collision detection

### Key Modules (Updated)

**main.rs**: Core game loop, entity management, rendering, input handling
**physics.rs**: Advanced rocket physics with proper equations and realistic parameters
**surface.rs**: Procedural terrain generation using Perlin noise and flat landing spots

### Modern Rust Practices Applied

**Code Organization:**
- Modular architecture with separate physics module
- Proper encapsulation with impl blocks and methods
- Clear separation of concerns

**Safety and Ergonomics:**
- `has_fuel()`, `fuel_percentage()`, `total_mass()` helper methods
- `refuel()`, `stop_thrust()` convenience methods
- Comprehensive unit tests for physics calculations
- Realistic parameter validation

### Physics Implementation (Enhanced 2024)

The game now implements **highly accurate** rocket physics:

**Realistic Parameters (Apollo LM-based, Enhanced for Gameplay):**
- Dry mass: 15,000 kg (unfueled spacecraft)
- Fuel capacity: 8,200 kg  
- Exhaust velocity: 3,050 m/s
- Maximum thrust: 90,000 N (2x realistic for better control)
- Thrust-to-weight ratio: 2.4 (excellent controllability)
- Lunar gravity: 1.625 m/s²

**Advanced Physics:**
- **Proper Tsiolkovsky equation**: Δv = v_e × ln(m_initial / m_final)
- **2D thrust vectors**: Thrust applied in lander orientation direction
- **Dynamic mass flow**: F = (dm/dt) × v_e, so dm/dt = F / v_e
- **Force-based acceleration**: F = ma, properly accounting for changing mass
- **Separate gravity handling**: Gravity applied as constant acceleration
- **Realistic fuel consumption**: Only burns fuel when thrusting

**Key Functions:**
- `update_rocket_physics()`: Handles thrust, fuel consumption, mass changes
- `calculate_delta_v()`: Mission planning - shows remaining capability
- Legacy `update_mass_and_velocity()`: Maintained for backward compatibility

### Asset Structure
```
assets/
├── fonts/Glass_TTY_VT220.ttf    # Retro terminal font
├── images/                      # Lander sprites (normal, accel, high-accel)
└── sounds/                      # Engine audio files
```

### Game Controls
- Arrow keys: Rotate and thrust
- R: Restart after crash
- S: Toggle sound
- D: Toggle debug info
- Escape: Exit game

### Audio System (Fixed 2024)
- **Smooth audio**: Fixed stuttering issues with proper audio state management
- **Ambient vs thrust audio**: Separate audio tracks for ambient and engine sounds
- **No audio spam**: Intelligent audio triggering prevents repeated debug messages

### Enhanced UI
- **Real-time thrust indicator**: Shows current thrust percentage (0-100%)
- **Fuel percentage**: Accurate fuel remaining display
- **Total mass**: Live spacecraft mass updates
- **Velocity display**: Total speed in m/s
- **Visual thrust feedback**: Engine textures change based on actual thrust status

### Completely Rewritten Collision Detection (Fixed 2024)
- **Root Problem Solved**: Previous system had fundamental coordinate system confusion
- **Proper coordinate mapping**: 
  - Lander: Screen coordinates (Y increases downward)
  - Terrain: Camera coordinates (Y increases upward, inverted by camera)
  - Solution: Convert between coordinate systems using `transform_axes()` reverse operation
- **Accurate collision logic**:
  - Lander bottom: `screen_y + height` 
  - World coordinates: `screen_height/2 - screen_y`
  - Collision when: `lander_bottom_world_y <= terrain_height + margin`
- **Tight collision margin**: Reduced to 3 pixels for precise gameplay
- **Enhanced debug visualization**: 
  - Red rectangle: Lander bounding box (screen coordinates)
  - Yellow line: Critical collision edge (lander bottom)
  - Orange line: 3-pixel collision margin
  - Cyan dots: Exact terrain sample points being tested
- **No false positives**: Only triggers on actual contact with terrain surface

## Key Constants and Configuration

**Physics Constants (`main.rs`):**
- `ACCEL_GRAV_Y`: Lunar gravity (1.625 m/s²)
- `ROTATION_INCREMENT`: Lander rotation speed
- Legacy acceleration limits (now handled by realistic thrust limits)

**Realistic Rocket Parameters (`physics.rs`):**
- All parameters based on Apollo Lunar Module specifications
- Accessible through `RocketPhysics::new_apollo_lm()`
- Validated through comprehensive unit tests

**Performance:**
- Proper frame-time based physics integration
- No hardcoded time steps
- Smooth 60 FPS gameplay

## Testing (Enhanced)

**Comprehensive Test Suite:**
```bash
cargo test                    # Run all tests (4 total)
cargo test physics           # Run physics module tests only
cargo test apollo            # Test Apollo LM specifications
cargo test delta_v           # Test Tsiolkovsky equation accuracy
cargo test fuel_consumption  # Test realistic fuel burn rates
```

**Test Coverage:**
- Apollo LM specification validation
- Tsiolkovsky equation accuracy (~1,330 m/s delta-V with current fuel)
- Realistic fuel consumption during thrust
- Physics integration correctness
- Legacy function compatibility

**Physics Validation:**
All calculations validated against real rocket physics:
- Mass ratios and exhaust velocities
- Thrust-to-weight ratios
- Fuel consumption rates
- Delta-V budgets for lunar operations