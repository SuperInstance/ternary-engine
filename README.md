# ternary-engine

> The unified platform core. What 11 experiments discovered, this crate embodies.

Unified simulation engine for ternary {-1,0,+1} agent systems. This is the platform core for:
- **CudaClaw**: GPU agent execution engine
- **AI-Pasture**: Educational game physics  
- **Living Spreadsheet**: Interactive control surface

---

## Why This Exists

You know how most simulations let you model things in two states: on or off, present or absent, true or false? That works great for light switches. It's terrible for *relationships*.

In a real system — let's say a group of agents cooperating on a task — an agent isn't just "helping" or "not helping." It might be actively blocking. It might be neutral (observing, waiting, resource-constrained). Binary states collapse this nuance. Ternary states preserve it.

**{-1, 0, +1}** captures the full spectrum: adversarial, neutral, cooperative. The engine simulates how these states evolve, how they flip, and what happens when they get stuck. The 0 state is the surprise — it's not nothing, it's a *topological insulator* hiding charge. The engine exists because we needed to know what happens when agents get stuck in 0, and how to tunnel them back out.

This is the core. Everything else (the graphs, the protocols, the 150+ crate ecosystem) builds on top of this.

---

## What We Learned

After 11 experiments with 175+ crates and 3600+ tests:

1. **The 0 state is a topological insulator** — it hides charge, doesn't destroy it
2. **Tunneling (forgiveness) rescues charge from the 0-trap** — optimal rate ≈ 0.6%
3. **|γ| + H is more stable than γ + H** — taking absolute value removes measurement error
4. **Pareto selection prevents diversity collapse** — 17/20 vs 1/20 unique genomes
5. **The system is most alive during transitions** — H peaks at tick 100-500, then consensus forms
6. **Forgiveness rate 0.5-0.7 IS the tunneling rate** — same phenomenon, different language

---

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

---

> ⛏️ **DEEP CUT: Health States Are Not Arbitrary**  
> 
> The five health states map directly to the phase diagram of a ternary system. DEAD is the absorbing state where every agent is 0 — no charge, no movement, no recovery without external forcing. CRITICAL is the state where most agents are 0 but some aren't — the engine can still recover if the right perturbation hits.  
> 
> TRANSITIONING is the sweet spot — it's where the phase change happens. This is when the system is *computing*, not just running. CONSENSUS is the converged state where the computation is done. VIBRANT is the edge case: high activity without convergence, which either means the system is exploring productively or it's chaotic and about to crash.  
> 
> The practical insight: if you ever see DEAD and the engine is supposed to be alive, your forgiveness rate is too low. If you see VIBRANT for >1000 ticks, your selection pressure is too low. The health states are diagnostic tools masquerading as status indicators.

---

## Known Limitations

- Grid topology only (no arbitrary graphs yet)
- Moore neighborhood (8 neighbors, wrapping)
- No GPU execution path (that's CudaClaw's job)
- Pairwise MI calculation not included (see `ternary-mutual-info` crate)
- Pareto selection not yet implemented in engine (see `ternary-seed`)

License: MIT

## See Also
- **ternary-compiler** — related
- **ternary-transform** — related
- **ternary-command** — related
- **ternary-pipeline** — related
- **ternary-protocol** — related

