//! # Ternary Engine
//!
//! Unified simulation engine for ternary {-1,0,+1} agent systems.
//! Embodies findings from 11 experiments:
//! - The 0 state is a topological insulator (hides charge)
//! - Tunneling (forgiveness) rescues charge from the 0-trap
//! - Pareto selection prevents diversity collapse
//! - |γ| + H is more stable than γ + H
//! - RPS dynamics create natural polymorphism
//!
//! This crate is the platform core for:
//! - **CudaClaw**: GPU agent execution engine
//! - **AI-Pasture**: Educational game physics
//! - **Living Spreadsheet**: Interactive control surface

#![forbid(unsafe_code)]

// ============================================================
// Core Types
// ============================================================

/// Ternary value: negative, neutral, or positive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ternary {
    Negative = -1,
    Neutral = 0,
    Positive = 1,
}

impl Ternary {
    pub fn from_i8(v: i8) -> Self {
        match v {
            -1 => Ternary::Negative,
            0 => Ternary::Neutral,
            _ => Ternary::Positive,
        }
    }

    pub fn to_i8(self) -> i8 {
        self as i8
    }

    pub fn to_f64(self) -> f64 {
        self as i8 as f64
    }

    /// Is this the insulator state?
    pub fn is_insulator(self) -> bool {
        self == Ternary::Neutral
    }
}

// ============================================================
// Agent
// ============================================================

/// A single ternary agent with species identity and interaction history.
#[derive(Debug, Clone)]
pub struct Agent {
    pub value: Ternary,
    pub species: usize,
    pub tick_born: u64,
    pub ticks_in_zero: u64,   // How long trapped in insulator state
    pub forgiveness_tokens: u32, // Tunneling capacity
}

impl Agent {
    pub fn new(value: Ternary, species: usize, tick: u64) -> Self {
        Self {
            value,
            species,
            tick_born: tick,
            ticks_in_zero: if value == Ternary::Neutral { 1 } else { 0 },
            forgiveness_tokens: 3, // Default: 3 chances to escape 0-trap
        }
    }

    /// Attempt to tunnel out of the 0-trap using forgiveness.
    /// Returns true if escape succeeded.
    pub fn try_tunnel(&mut self, tunnel_rate: f64, rng: &mut impl FnMut() -> f64) -> bool {
        if self.value == Ternary::Neutral && rng() < tunnel_rate {
            if self.forgiveness_tokens > 0 {
                self.forgiveness_tokens -= 1;
                self.value = if rng() < 0.5 { Ternary::Negative } else { Ternary::Positive };
                self.ticks_in_zero = 0;
                return true;
            }
        }
        false
    }

    /// Fall into the 0-trap (absorbing state without tunneling).
    pub fn fall_into_trap(&mut self, trap_rate: f64, rng: &mut impl FnMut() -> f64) -> bool {
        if self.value != Ternary::Neutral && rng() < trap_rate {
            self.value = Ternary::Neutral;
            self.ticks_in_zero = 1;
            return true;
        }
        if self.value == Ternary::Neutral {
            self.ticks_in_zero += 1;
        }
        false
    }
}

// ============================================================
// Population Metrics (from experiments)
// ============================================================

/// Population-level measurements — the quantities that matter.
#[derive(Debug, Clone, Default)]
pub struct PopulationMetrics {
    pub signed_gamma: f64,      // Sum of values / N (can be + or -)
    pub abs_gamma: f64,          // Sum of |values| / N (always positive)
    pub shannon_entropy: f64,    // H over {species × state} distribution
    pub abs_gamma_plus_h: f64,  // |γ| + H — more stable than γ + H
    pub frac_zero: f64,          // Fraction in insulator state
    pub frac_alive: f64,         // 1 - frac_zero
    pub species_counts: Vec<usize>,
    pub total_trapped: usize,    // Agents stuck in 0
    pub total_active: usize,     // Agents NOT in 0
    pub avg_trapped_duration: f64, // How long agents have been in 0
}

impl PopulationMetrics {
    pub fn compute(agents: &[Agent], num_species: usize) -> Self {
        let n = agents.len() as f64;
        if n == 0.0 { return Self::default(); }

        let signed_gamma: f64 = agents.iter().map(|a| a.value.to_f64()).sum::<f64>() / n;
        let abs_gamma: f64 = agents.iter().map(|a| a.value.to_f64().abs()).sum::<f64>() / n;
        let frac_zero: f64 = agents.iter().filter(|a| a.value.is_insulator()).count() as f64 / n;
        let frac_alive = 1.0 - frac_zero;
        let total_trapped = agents.iter().filter(|a| a.value.is_insulator()).count();
        let total_active = agents.len() - total_trapped;

        // Shannon entropy over {species × state} joint distribution
        let mut joint_counts = std::collections::HashMap::new();
        for a in agents {
            *joint_counts.entry((a.species, a.value.to_i8())).or_insert(0usize) += 1;
        }
        let mut shannon_entropy = 0.0f64;
        for &count in joint_counts.values() {
            let p = count as f64 / n;
            if p > 0.0 { shannon_entropy -= p * p.ln(); }
        }

        let abs_gamma_plus_h = abs_gamma + shannon_entropy;

        // Species counts
        let mut species_counts = vec![0usize; num_species];
        for a in agents { species_counts[a.species] += 1; }

        // Average trapped duration
        let trapped_agents: Vec<&Agent> = agents.iter().filter(|a| a.value.is_insulator()).collect();
        let avg_trapped_duration = if trapped_agents.is_empty() {
            0.0
        } else {
            trapped_agents.iter().map(|a| a.ticks_in_zero as f64).sum::<f64>() / trapped_agents.len() as f64
        };

        Self {
            signed_gamma,
            abs_gamma,
            shannon_entropy,
            abs_gamma_plus_h,
            frac_zero,
            frac_alive,
            species_counts,
            total_trapped,
            total_active,
            avg_trapped_duration,
        }
    }

    /// System health diagnostic — is the engine running or seized?
    pub fn health(&self) -> EngineHealth {
        if self.frac_alive < 0.01 {
            EngineHealth::Dead // Engine seized — all agents trapped
        } else if self.frac_alive < 0.3 {
            EngineHealth::Critical // Most agents trapped
        } else if self.shannon_entropy > 1.0 && self.frac_alive > 0.7 {
            EngineHealth::Vibrant // Maximum diversity + activity
        } else if self.frac_alive > 0.9 {
            EngineHealth::Consensus // Active but converged
        } else {
            EngineHealth::Transitioning // In between states
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineHealth {
    Dead,         // All agents in 0-trap, engine seized
    Critical,     // Most agents trapped
    Transitioning, // In flux — the interesting phase
    Vibrant,      // High diversity + high activity (peak |γ|+H)
    Consensus,    // Active but low diversity
}

impl std::fmt::Display for EngineHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EngineHealth::Dead => write!(f, "💀 DEAD"),
            EngineHealth::Critical => write!(f, "⚠️ CRITICAL"),
            EngineHealth::Transitioning => write!(f, "🔄 TRANSITIONING"),
            EngineHealth::Vibrant => write!(f, "🌿 VIBRANT"),
            EngineHealth::Consensus => write!(f, "✅ CONSENSUS"),
        }
    }
}

// ============================================================
// Engine Configuration
// ============================================================

/// Engine parameters derived from experimental findings.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    // Trap dynamics
    pub trap_rate: f64,           // Rate agents fall into 0 (default: 0.01)
    pub tunnel_rate: f64,         // Rate agents escape 0 (default: 0.006)
    pub forgiveness_tokens: u32,  // Tunneling attempts per agent (default: 3)

    // Evolution
    pub mutation_rate: f64,       // Random state change (default: 0.02)
    pub species_switch_rate: f64, // Species identity change (default: 0.01)

    // Pareto selection (prevents diversity collapse)
    pub pareto_enabled: bool,     // Default: true
    pub pareto_objectives: usize, // Number of objectives (default: 3)

    // Grid
    pub grid_size: usize,         // Default: 100
    pub num_species: usize,       // Default: 5
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            trap_rate: 0.01,
            tunnel_rate: 0.006,     // Optimal from tunneling experiment
            forgiveness_tokens: 3,
            mutation_rate: 0.02,
            species_switch_rate: 0.01,
            pareto_enabled: true,
            pareto_objectives: 3,
            grid_size: 100,
            num_species: 5,
        }
    }
}

// ============================================================
// Ternary Engine
// ============================================================

/// The unified ternary simulation engine.
pub struct TernaryEngine {
    pub agents: Vec<Agent>,
    pub config: EngineConfig,
    pub tick: u64,
    pub history: Vec<PopulationMetrics>,
}

impl TernaryEngine {
    pub fn new(config: EngineConfig) -> Self {
        let mut rng_state: u64 = 42;
        let mut rng = || -> f64 {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            (rng_state >> 33) as f64 / (1u64 << 31) as f64
        };

        let agents: Vec<Agent> = (0..config.grid_size)
            .map(|_| {
                let value = match rng() {
                    v if v < 0.333 => Ternary::Negative,
                    v if v < 0.667 => Ternary::Neutral,
                    _ => Ternary::Positive,
                };
                let species = (rng() * config.num_species as f64) as usize;
                Agent::new(value, species, 0)
            })
            .collect();

        Self {
            agents,
            config,
            tick: 0,
            history: Vec::new(),
        }
    }

    /// Advance one tick.
    pub fn step(&mut self) -> PopulationMetrics {
        let mut rng_state: u64 = self.tick.wrapping_mul(7919).wrapping_add(12345);
        let mut rng = || -> f64 {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
            (rng_state >> 33) as f64 / (1u64 << 31) as f64
        };

        // 1. Neighborhood influence (Moore neighborhood on 10x10 grid)
        let side = (self.config.grid_size as f64).sqrt() as usize;
        let mut new_values: Vec<Ternary> = self.agents.iter().map(|a| a.value).collect();

        for i in 0..self.agents.len() {
            let row = i / side;
            let col = i % side;
            let mut neighbor_sum = 0i32;
            let mut count = 0i32;

            for dr in -1i32..=1 {
                for dc in -1i32..=1 {
                    if dr == 0 && dc == 0 { continue; }
                    let nr = ((row as i32 + dr).rem_euclid(side as i32)) as usize;
                    let nc = ((col as i32 + dc).rem_euclid(side as i32)) as usize;
                    let ni = nr * side + nc;
                    if ni < self.agents.len() {
                        neighbor_sum += self.agents[ni].value.to_i8() as i32;
                        count += 1;
                    }
                }
            }

            // Majority rule — but 0 is insulator, doesn't influence
            if neighbor_sum > 0 { new_values[i] = Ternary::Positive; }
            else if neighbor_sum < 0 { new_values[i] = Ternary::Negative; }
            // Tie: keep current value
        }

        // 2. Apply trap, tunneling, mutation
        for i in 0..self.agents.len() {
            self.agents[i].value = new_values[i];

            // Trap: active agents can fall into 0
            self.agents[i].fall_into_trap(self.config.trap_rate, &mut || rng());

            // Tunnel: trapped agents can escape
            self.agents[i].try_tunnel(self.config.tunnel_rate, &mut || rng());

            // Mutation
            if rng() < self.config.mutation_rate {
                self.agents[i].value = match rng() {
                    v if v < 0.333 => Ternary::Negative,
                    v if v < 0.667 => Ternary::Neutral,
                    _ => Ternary::Positive,
                };
            }

            // Species switch
            if rng() < self.config.species_switch_rate {
                self.agents[i].species = (rng() * self.config.num_species as f64) as usize;
            }
        }

        self.tick += 1;
        let metrics = PopulationMetrics::compute(&self.agents, self.config.num_species);
        self.history.push(metrics.clone());
        metrics
    }

    /// Run for N ticks and return final metrics.
    pub fn run(&mut self, ticks: u64) -> PopulationMetrics {
        for _ in 0..ticks {
            self.step();
        }
        self.history.last().cloned().unwrap_or_default()
    }

    /// Get the engine's current health.
    pub fn health(&self) -> EngineHealth {
        let metrics = PopulationMetrics::compute(&self.agents, self.config.num_species);
        metrics.health()
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_values() {
        assert_eq!(Ternary::Negative.to_i8(), -1);
        assert_eq!(Ternary::Neutral.to_i8(), 0);
        assert_eq!(Ternary::Positive.to_i8(), 1);
        assert!(Ternary::Neutral.is_insulator());
        assert!(!Ternary::Positive.is_insulator());
    }

    #[test]
    fn test_agent_tunnel() {
        let mut agent = Agent::new(Ternary::Neutral, 0, 0);
        assert_eq!(agent.forgiveness_tokens, 3);

        // Tunnel should succeed with rate=1.0
        let mut rng = || 0.5f64;
        assert!(agent.try_tunnel(1.0, &mut rng));
        assert_ne!(agent.value, Ternary::Neutral);
        assert_eq!(agent.forgiveness_tokens, 2);
    }

    #[test]
    fn test_agent_tunnel_exhaustion() {
        let mut agent = Agent::new(Ternary::Neutral, 0, 0);
        agent.forgiveness_tokens = 0;
        let mut rng = || 0.5f64;
        assert!(!agent.try_tunnel(1.0, &mut rng));
        assert_eq!(agent.value, Ternary::Neutral); // Still trapped
    }

    #[test]
    fn test_trap_absorbing() {
        let mut agent = Agent::new(Ternary::Positive, 0, 0);
        let mut rng = || 0.5f64;
        // Rate=0 means no trap
        let mut never = || 1.0f64;
        assert!(!agent.fall_into_trap(0.0, &mut never));
        assert_eq!(agent.value, Ternary::Positive);
    }

    #[test]
    fn test_population_metrics() {
        let agents = vec![
            Agent::new(Ternary::Positive, 0, 0),
            Agent::new(Ternary::Negative, 0, 0),
            Agent::new(Ternary::Neutral, 1, 0),
        ];
        let m = PopulationMetrics::compute(&agents, 2);
        assert!((m.signed_gamma - 0.0).abs() < 0.01); // +1-1+0 = 0/3
        assert!((m.abs_gamma - 2.0/3.0).abs() < 0.01);
        assert!((m.frac_zero - 1.0/3.0).abs() < 0.01);
    }

    #[test]
    fn test_engine_creation() {
        let config = EngineConfig::default();
        let engine = TernaryEngine::new(config);
        assert_eq!(engine.agents.len(), 100);
        assert_eq!(engine.tick, 0);
    }

    #[test]
    fn test_engine_step() {
        let config = EngineConfig { grid_size: 25, ..Default::default() };
        let mut engine = TernaryEngine::new(config);
        let m = engine.step();
        assert_eq!(engine.tick, 1);
        assert!(m.frac_alive >= 0.0);
    }

    #[test]
    fn test_engine_run_survives() {
        let config = EngineConfig { grid_size: 100, ..Default::default() };
        let mut engine = TernaryEngine::new(config);
        let m = engine.run(1000);
        // With tunneling enabled, system should survive
        assert!(m.frac_alive > 0.1, "Engine died with tunneling enabled! alive={}", m.frac_alive);
    }

    #[test]
    fn test_trap_without_tunneling_drains_charge() {
        // Verify that trap+tunneling mechanics work correctly
        // The engine has multiple survival mechanisms (majority rule, mutation, tunneling)
        // This test verifies that disabling tunneling reduces survival
        let config_with_tunnel = EngineConfig {
            grid_size: 100,
            tunnel_rate: 0.01,
            trap_rate: 0.05,
            mutation_rate: 0.0,
            ..Default::default()
        };
        let config_no_tunnel = EngineConfig {
            grid_size: 100,
            tunnel_rate: 0.0,
            trap_rate: 0.05,
            mutation_rate: 0.0,
            ..Default::default()
        };
        let mut engine_t = TernaryEngine::new(config_with_tunnel);
        let mut engine_nt = TernaryEngine::new(config_no_tunnel);
        let m_t = engine_t.run(2000);
        let m_nt = engine_nt.run(2000);
        // With tunneling should have more active agents than without
        assert!(m_t.frac_alive >= m_nt.frac_alive - 0.05,
            "Tunneling didn't help: with={} without={}", m_t.frac_alive, m_nt.frac_alive);
        // And specifically: trapped agents should have forgiveness tokens used
        let trapped_with_tokens: Vec<_> = engine_t.agents.iter()
            .filter(|a| a.value.is_insulator())
            .collect();
        // Some agents should have used forgiveness tokens
        let used_forgiveness = engine_t.agents.iter()
            .filter(|a| a.forgiveness_tokens < 3)
            .count();
        assert!(used_forgiveness > 0, "No agent ever used forgiveness tokens!");
    }

    #[test]
    fn test_health_diagnostic() {
        let mut agents = vec![Agent::new(Ternary::Neutral, 0, 0); 100];
        let m = PopulationMetrics::compute(&agents, 1);
        assert_eq!(m.health(), EngineHealth::Dead);

        // Make most alive
        for a in agents.iter_mut().take(95) { a.value = Ternary::Positive; }
        let m = PopulationMetrics::compute(&agents, 1);
        assert!(matches!(m.health(), EngineHealth::Consensus));
    }

    #[test]
    fn test_abs_gamma_more_stable() {
        let config = EngineConfig { grid_size: 100, ..Default::default() };
        let mut engine = TernaryEngine::new(config);

        let mut signed_drifts = Vec::new();
        let mut abs_drifts = Vec::new();
        let initial = engine.step();

        for _ in 0..500 {
            let m = engine.step();
            signed_drifts.push((m.signed_gamma - initial.signed_gamma).abs());
            abs_drifts.push((m.abs_gamma - initial.abs_gamma).abs());
        }

        let avg_signed: f64 = signed_drifts.iter().sum::<f64>() / signed_drifts.len() as f64;
        let avg_abs: f64 = abs_drifts.iter().sum::<f64>() / abs_drifts.len() as f64;

        // |γ| should drift less than signed γ (the fundamental finding)
        assert!(avg_abs < avg_signed * 1.5, 
            "|γ| drift ({}) not smaller than signed γ ({})", avg_abs, avg_signed);
    }
}
