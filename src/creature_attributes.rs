use serde::{Deserialize, Serialize};

/// Defines the dietary preference of a creature.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DietType {
    Herbivore, // Eats plants (not implemented yet)
    Carnivore, // Eats other creatures
    Omnivore,  // Eats both
}

/// Core attributes defining a creature's state and ecological role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureAttributes {
    pub energy: f32,
    pub max_energy: f32,
    pub energy_recovery_rate: f32, // Energy gained per second when resting

    pub satiety: f32,
    pub max_satiety: f32,
    pub metabolic_rate: f32, // Satiety lost per second passively

    pub diet_type: DietType,
    pub size: f32, // General size indicator

    // Tags defining what this creature *can* eat
    pub prey_tags: Vec<String>,
    // Tags defining what this creature is. Used for things like determining which things can eat this creature.
    pub self_tags: Vec<String>,
}

#[allow(dead_code)]
impl CreatureAttributes {
    /// Creates a new set of attributes with default values.
    /// Consider using a builder pattern if this gets complex.
    pub fn new(
        max_energy: f32,
        energy_recovery_rate: f32,
        max_satiety: f32,
        metabolic_rate: f32,
        diet_type: DietType,
        size: f32,
        prey_tags: Vec<String>,
        self_tags: Vec<String>,
    ) -> Self {
        Self {
            energy: max_energy, // Start full
            max_energy,
            energy_recovery_rate,
            satiety: max_satiety, // Start full
            max_satiety,
            metabolic_rate,
            diet_type,
            size,
            prey_tags,
            self_tags,
        }
    }

    // Placeholder methods for future logic
    pub fn update_passive_stats(&mut self, dt: f32, is_resting: bool) {
        // Decrease satiety over time
        self.satiety = (self.satiety - self.metabolic_rate * dt).max(0.0);

        // Passive metabolic energy drain (always occurs)
        self.energy = (self.energy - self.metabolic_rate * dt * 0.5).max(0.0); // Example: energy drains at half the metabolic rate of satiety

        // Recover energy if resting
        if is_resting {
            self.energy = (self.energy + self.energy_recovery_rate * dt).min(self.max_energy);
        }
    }

    pub fn consume_energy(&mut self, amount: f32) {
        self.energy = (self.energy - amount).max(0.0);
    }

    pub fn gain_satiety(&mut self, amount: f32) {
        self.satiety = (self.satiety + amount).min(self.max_satiety);
    }

    pub fn is_hungry(&self) -> bool {
        self.satiety < self.max_satiety * 0.5 // Example threshold
    }

    pub fn is_tired(&self) -> bool {
        self.energy < self.max_energy * 0.2 // Example threshold
    }

    /// Checks if this creature *can* eat another creature based on tags.
    pub fn can_eat(&self, other: &CreatureAttributes) -> bool {
        match self.diet_type {
            DietType::Herbivore => false, // Can't eat creatures
            DietType::Carnivore | DietType::Omnivore => {
                // Must be smaller or similar size (adjust multiplier as needed)
                if other.size > self.size * 1.5 { return false; }
                // Check if any of the other's tags match our prey tags
                self.prey_tags.iter().any(|prey_tag| other.self_tags.contains(prey_tag))
            }
        }
    }

    /// Checks if this creature *can* be eaten by another creature based on tags.
    pub fn can_be_eaten_by(&self, potential_predator: &CreatureAttributes) -> bool {
        potential_predator.can_eat(self)
    }
} 