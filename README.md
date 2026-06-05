# ternary-engine

> The unified platform core. What 11 experiments discovered, this crate embodies.

Unified simulation engine for ternary {-1,0,+1} agent systems. This is the platform core for:
- **CudaClaw**: GPU agent execution engine
- **AI-Pasture**: Educational game physics  
- **Living Spreadsheet**: Interactive control surface

## What We Learned

After 11 experiments with 175+ crates and 3600+ tests:

1. **The 0 state is a topological insulator** — it hides charge, doesn't destroy it
2. **Tunneling (forgiveness) rescues charge from the 0-trap** — optimal rate ≈ 0.6%
3. **|γ| + H is more stable than γ + H** — taking absolute value removes measurement error
4. **Pareto selection prevents diversity collapse** — 17/20 vs 1/20 unique genomes
5. **The system is most alive during transitions** — H peaks at tick 100-500, then consensus forms
6. **Forgiveness rate 0.5-0.7 IS the tunneling rate** — same phenomenon, different language

## Usage

```rust
use ternary_engine::{TernaryEngine, EngineConfig};

let config = EngineConfig::default();
let mut engine = TernaryEngine::new(config);

for _ in 0..1000 {
    let metrics = engine.step();
    println!("Tick {}: {} alive={:.1}%", engine.tick, metrics.health(), metrics.frac_alive * 100.0);
}
```

## Engine Health States

| State | Meaning |
|-------|---------|
| 💀 DEAD | All agents trapped in 0, engine seized |
| ⚠️ CRITICAL | Most agents trapped |
| 🔄 TRANSITIONING | In flux — the interesting phase |
| 🌿 VIBRANT | High diversity + high activity |
| ✅ CONSENSUS | Active but converged |

## Known Limitations

- Grid topology only (no arbitrary graphs yet)
- Moore neighborhood (8 neighbors, wrapping)
- No GPU execution path (that's CudaClaw's job)
- Pairwise MI calculation not included (see `ternary-mutual-info` crate)
- Pareto selection not yet implemented in engine (see `ternary-seed`)

License: MIT
