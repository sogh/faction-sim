# Emergent Medieval Simulation Engine

A terrarium-style simulation where hundreds of agents with simple behavioral rules produce emergent, dramatically compelling narratives.

## What Is This?

A headless simulation engine that generates history organically. Agents navigate loyalty, survival, and ambition within a medieval world of competing factions. The simulation produces structured output for a Director AI to select what's worth visualizing and for LLMs to summarize into dramatic retellings.

## Core Concept

The tension this project explores: mathematically emergent systems often feel alien, while human-feeling simulations typically require complex rule sets. We aim for the sweet spot — simple rules that produce genuinely human drama.

## Documentation

- `docs/emergent_sim_design.md` — High-level vision and design principles
- `docs/simulation_output_schemas.md` — JSON schemas for all simulation output  
- `docs/agent_behavioral_rules.md` — Agent decision-making rules and weighting system
- `docs/implementation_prompts.md` — Phased prompts for building with Claude Code

## Project Structure

```
emergent_sim/
├── README.md
├── claude.md              # Claude Code project context
├── docs/
│   ├── emergent_sim_design.md
│   ├── simulation_output_schemas.md
│   ├── agent_behavioral_rules.md
│   └── implementation_prompts.md
└── src/                   # (to be created)
```

## Key Features

- **Event Sourcing**: Every action produces an immutable event. Full history replay and branching.
- **Multi-Dimensional Trust**: Agents track reliability, alignment, and capability separately.
- **Memory Propagation**: Gossip spreads with attribution and fidelity loss.
- **Faction Archives**: Written records that can be read, destroyed, or forged.
- **Ritual Readings**: Scheduled gatherings that reinforce faction memory selectively.
- **Tension Detection**: Pre-computed drama indicators for the Director AI.
- **Intervention System**: Pause and modify the simulation at any point.

## Getting Started

1. Read `docs/emergent_sim_design.md` to understand the vision
2. Review `docs/agent_behavioral_rules.md` for how agents decide
3. Follow prompts in `docs/implementation_prompts.md` to build incrementally
4. Reference `claude.md` for coding conventions and architecture

## Technical Stack

- **Language**: Rust
- **ECS**: Bevy ECS
- **Output**: JSON/JSONL for human and LLM readability
- **Target**: 200-500 agents, 1000+ ticks/second

## License

[Your choice]

---

*Built through design conversation, December 2024*
