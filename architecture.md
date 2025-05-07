# Softies Project Architecture

This document outlines the general code architecture of the Softies project, a 2D soft-body creature simulation.

## 1. Crate and Module Structure

The project is organized into several key Rust modules within the `src` directory:

*   **`main.rs`**:
    *   The binary entry point for native execution.
    *   Initializes `eframe` and sets up the native window.
    *   Instantiates and runs `SoftiesApp`.
    *   Handles basic setup like `tracing_subscriber`.

*   **`lib.rs`**:
    *   The library entry point, primarily used for WebAssembly (WASM) builds.
    *   Defines the `#[wasm_bindgen]` function `start()` to bootstrap the application in a web environment.
    *   Declares the main modules of the application (`app`, `creature`, `creatures`, `creature_attributes`).

*   **`app.rs` (`SoftiesApp`)**:
    *   The heart of the application, implementing the `eframe::App` trait.
    *   **Responsibilities**:
        *   Owns and manages the Rapier2D physics world components (`RigidBodySet`, `ColliderSet`, `ImpulseJointSet`, `QueryPipeline`, `PhysicsPipeline`, etc.).
        *   Maintains a list of all active creatures (`Vec<Box<dyn Creature>>`).
        *   Manages view state (e.g., `view_center`, `zoom`), though panning/zooming are not yet implemented.
        *   Handles the main simulation update loop.
        *   Manages UI state and rendering using `egui`.
        *   Spawns initial creatures and environment (walls).

*   **`creature.rs`**:
    *   Defines the core abstractions for all creatures:
        *   `Creature` trait: The common interface that all creature types (e.g., `Snake`, `Plankton`) must implement. It includes methods for accessing physics handles, attributes, updating state and behavior, applying custom forces, and drawing.
        *   `CreatureState` enum: Represents the general behavioral state of a creature (e.g., `Wandering`, `SeekingFood`, `Resting`).
        *   `WorldContext` struct: Passes environmental information (like world dimensions) to creatures.
        *   `CreatureInfo` struct: A lightweight data structure containing essential information about a creature (ID, type, position, velocity, radius). This is used to allow creatures to be aware of others in their vicinity without needing direct access to the `Box<dyn Creature>` objects, simplifying borrowing and data sharing.

*   **`creatures/` (directory)**:
    *   Contains modules for specific creature implementations. Each file (e.g., `plankton.rs`, `snake.rs`) defines a struct that implements the `Creature` trait.
    *   `creatures::mod.rs`: Publicly exports the creature structs from this directory.
    *   **Example (`plankton.rs`, `snake.rs`)**:
        *   Define the creature's specific data (e.g., segment handles, radii, internal timers).
        *   Implement `spawn_rapier` to create its physical representation in the Rapier world.
        *   Implement `update_state_and_behavior` to define its AI, state transitions, and interactions (including boids logic for Plankton).
        *   Implement `apply_custom_forces` for specialized physics like buoyancy or drag.
        *   Implement `draw` for its visual representation.

*   **`creature_attributes.rs`**:
    *   Defines `CreatureAttributes` struct: Holds common attributes like energy, satiety, size, diet type, and tags for ecological interactions (prey/self tags).
    *   Defines `DietType` enum.
    *   Provides methods for managing these attributes (e.g., `update_passive_stats`, `consume_energy`, `can_eat`).

## 2. Core Application Flow (within `SoftiesApp::update`)

The main simulation loop in `SoftiesApp::update` executes roughly in this order:

1.  **Input & Time**: Gets delta time (`dt`) from `egui` context.
2.  **Passive Creature Updates**:
    *   Iterates through creatures, updating their passive attributes (e.g., energy recovery if resting, satiety decrease) via `creature.attributes_mut().update_passive_stats()`.
3.  **Prepare `CreatureInfo`**:
    *   Creates a `Vec<CreatureInfo>` by iterating through all creatures. For each creature, it extracts its ID, type, primary body handle, current position, velocity (from `RigidBodySet`), and radius. This vector provides a snapshot of the world state for sensing.
4.  **Active Creature Updates (Behavior & State)**:
    *   Iterates through creatures, calling `creature.update_state_and_behavior()`. This is where individual creature AI, state transitions, and complex interactions (like boids) are handled. Creatures receive:
        *   Their own ID (`own_id`).
        *   Mutable access to `RigidBodySet` and `ImpulseJointSet` (for acting on themselves).
        *   Immutable access to `ColliderSet` and `QueryPipeline` (for sensing the environment).
        *   The `Vec<CreatureInfo>` (for awareness of other creatures).
        *   `WorldContext`.
5.  **Apply Custom Physics Forces**:
    *   Iterates through creatures, calling `creature.apply_custom_forces()`. This allows creatures to apply specific physical effects not covered by general Rapier forces (e.g., snake's anisotropic drag, plankton's buoyancy).
6.  **Physics Step**:
    *   Advances the Rapier2D physics simulation using `physics_pipeline.step()`. This integrates forces, detects collisions, and updates the positions and velocities of all rigid bodies and colliders.
7.  **Failsafe**: Checks for and resets any creatures that may have escaped the defined world boundaries.
8.  **UI Rendering (`egui`)**:
    *   **Side Panel**: Displays a list of creatures and their current state. Provides hover interactions.
    *   **Central Panel (Simulation View)**:
        *   Obtains a `Painter` from `egui`.
        *   Performs world-to-screen coordinate transformations.
        *   Draws the walls of the aquarium.
        *   Iterates through creatures, calling their `draw()` method to render them.

## 3. Physics Approach (Rapier2D)

The simulation uses the Rapier2D physics engine to manage all physical interactions.

*   **Core Components**:
    *   `RigidBodySet`: Stores all rigid bodies (dynamic, fixed, kinematic).
    *   `ColliderSet`: Stores all colliders attached to rigid bodies, defining their shapes and physical properties (density, restitution, friction).
    *   `ImpulseJointSet`: Manages joints that connect rigid bodies (e.g., `RevoluteJoint` for snake segments or plankton's two-body structure).
    *   `IntegrationParameters`: Controls global physics parameters like gravity, timestep details (though we use `dt` from `egui`), and solver iterations.
    *   `PhysicsPipeline`: The main entry point for stepping the physics simulation.
    *   `QueryPipeline`: Used for spatial queries (e.g., ray casting, shape casting, finding intersections) independent of the physics step. Crucial for creature sensing.
    *   `IslandManager`, `BroadPhaseMultiSap`, `NarrowPhase`, `CCDSolver`: Internal components of Rapier that handle various stages of the physics simulation.

*   **Creature Representation**:
    *   Creatures are typically composed of one or more `RigidBody` instances (e.g., segments of a snake, the two parts of a plankton).
    *   These rigid bodies are often connected by `ImpulseJoints`.
    *   Each rigid body has one or more `Collider` instances defining its physical shape (e.g., `ColliderBuilder::ball()`, `ColliderBuilder::cuboid()`).
    *   The `user_data` field on `Collider`s is used to store the unique `u128` ID of the creature they belong to. This allows linking a physics object back to a creature instance. Walls use `u128::MAX` as their ID.

*   **Movement and Forces**:
    *   **Direct Impulses**: Creatures can apply direct forces or impulses to their rigid bodies (e.g., for random wandering, boids steering).
    *   **Joint Motors**: Joints can have motors (e.g., the snake's wiggle is driven by setting target velocities on its `RevoluteJoint` motors).
    *   **Custom Forces**: The `apply_custom_forces` method in the `Creature` trait allows for bespoke physics, like the buoyancy applied to plankton or the anisotropic drag for snake segments.
    *   **Global Gravity**: A global gravitational force is applied by Rapier (currently `Vector2::new(0.0, -1.0)` for a gentle downward pull).

*   **Sensing and Interaction**:
    *   **`QueryPipeline`**: This is the primary mechanism for creatures to "sense" their environment. `query_pipeline.intersections_with_shape()` is used by plankton to find nearby colliders within their "perception radius" for the boids algorithm.
    *   **`Collider::user_data`**: Used to identify *what* has been sensed (linking back to a creature ID).
    *   **`Vec<CreatureInfo>`**: Once a nearby creature's ID is found via the query pipeline, its detailed information (type, full position, velocity) is looked up in this vector.
    *   **Collision Groups & Filters**: `InteractionGroups` and `QueryFilter` are used with the `QueryPipeline` to selectively sense certain types of objects (e.g., only other creatures on a specific collision group, excluding oneself). Colliders themselves will also need their collision groups set appropriately to control physical interactions and sensor detection.

## 4. Rendering

*   Rendering is handled by `egui`.
*   The `SoftiesApp`'s central panel is used as a canvas.
*   A `world_to_screen` transformation function converts physics coordinates (meters, Y-up) to `egui` screen coordinates (pixels, Y-down, origin top-left of drawing area).
*   Each creature implements a `draw()` method, which takes an `egui::Painter`, the `RigidBodySet` (to get current positions), and the transformation function to draw itself.

## 5. Analogy to Entity Component System (ECS)

While not strictly an ECS architecture, the project exhibits some ECS-like patterns:

*   **Entities**: The `Box<dyn Creature>` instances in `SoftiesApp::creatures` act as entities.
*   **Components**:
    *   Data within each creature struct (e.g., `Plankton::primary_radius`, `Snake::wiggle_timer`).
    *   `CreatureAttributes` associated with each creature.
    *   Rapier2D handles (`RigidBodyHandle`, `ImpulseJointHandle`, `ColliderHandle`) effectively link entities to their physics components managed by Rapier.
*   **Systems**: The loops within `SoftiesApp::update` that iterate over creatures to perform specific actions (update passive stats, update behavior, apply forces, draw) are analogous to systems that operate on entities with specific components. Rapier's `physics_pipeline.step()` is a large, integrated system for all physics-related components.

This architecture aims for a separation of concerns, allowing creature-specific logic to be encapsulated within their respective modules while `app.rs` orchestrates the overall simulation and interaction with the `eframe` and `Rapier2D` libraries. 