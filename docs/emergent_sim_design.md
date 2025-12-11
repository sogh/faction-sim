# Emergent Medieval Simulation Engine

## Vision

A terrarium-style simulation where hundreds of agents with simple behavioral rules produce unpredictable, dramatically compelling emergent narratives. The system generates history organically, which can be observed in real-time or summarized into dramatic retellings.

The core tension this project explores: mathematically emergent systems often feel alien, while human-feeling simulations typically require complex rule sets. We aim for the sweet spot — simple rules that produce genuinely human drama.

---

## World Setting

A medieval-era world with 4-5 factions controlling different territories and resources. Agents navigate loyalty, survival, and ambition within this constrained political landscape.

---

## Core Design Principles

### Emergence Through Constraint
Rather than scripting behavior, drama emerges from incompatibilities:
- Agents cannot belong to more than one faction simultaneously
- Resources are scarce and seasonally variable
- Loyalty to multiple people can become mutually exclusive
- Forced choices create dramatic tension

### Environmental Pressure as Plot Engine
The world itself forces characters to change and conflict:
- Seasonal scarcity (winter reduces food, increases desperation)
- External threats requiring cross-faction cooperation
- Resources that deplete and shift in value over time
- Succession crises when faction leaders die

### Multi-Scale Feedback
- Individual actions aggregate into group mood
- Group mood modifies individual action thresholds
- Creates oscillations, tipping points, and unexpected cascades

---

## Agent System

### Scale
- Hundreds of agents total
- Each faction has at least a dozen members
- Agents are unique individuals with distinct traits

### Trust Model
Trust is multi-dimensional rather than a single value:

| Dimension | Description |
|-----------|-------------|
| **Reliability** | Do they do what they say they will? |
| **Alignment** | Do they want what I want? |
| **Capability** | Can they actually deliver on promises? |

These dimensions can diverge: an agent might trust someone's intentions but not their competence, or trust their skills but suspect their loyalty.

### Memory System
- Agents remember events firsthand with full fidelity
- Agents remember betrayal specifically and persistently
- Memories can be communicated to other agents

#### Memory Propagation Rules
When Agent A tells Agent B about Agent C:
- **Degraded fidelity**: Secondhand information carries less weight than direct experience
- **Source attribution**: B remembers "A claims C is a traitor" — if A is later proven unreliable, B can re-evaluate
- **Faction filtering**: Information from in-group sources is believed more readily; propaganda emerges naturally

---

## Faction System

### Structure
- 4-5 factions with distinct territories and resource bases
- Exclusive membership (agents cannot dual-enroll)
- Internal hierarchy and roles

### The Archive: Physical Institutional Memory
Each faction maintains a written archive at their headquarters:
- Agents can write entries recording events, betrayals, alliances
- Other agents can read these records
- Records can be erased, destroyed, stolen, or forged

#### What This Creates
- **Politics of the archive**: Who gets to write? Who gets to read?
- **Selective recording**: Leadership embarrassments may go unrecorded; official history diverges from living memory
- **Destruction as drama**: Burning archives causes faction-wide amnesia; coups might prioritize the record room
- **Forgery**: Agents can plant false records, backdate alliances, frame rivals
- **Intelligence operations**: Raiding enemy HQ yields their records, grudges, and internal fractures

### The Ritual Reading
Each faction holds scheduled gatherings where a specialist (the Reader) recites from the faction's book aloud.

#### Mechanical Implications
- **Selective reinforcement**: The Reader chooses what to recite; repeated readings strengthen faction-wide memory of certain events while neglected entries fade
- **The Reader as power broker**: This role shapes what the faction believes about itself; politically significant enough to assassinate, bribe, or replace
- **Attendance matters**: Agents present get reinforcement; absent agents drift out of sync with faction consensus
- **Rhythm and vulnerability**: Scheduled gatherings are predictable — can be disrupted, attacked, infiltrated
- **Espionage vector**: Spies attending enemy readings learn what that faction is reinforcing

---

## Simulation Architecture

### Event Sourcing
Every significant action logs as an immutable event:
- Timestamp
- Actors involved
- Location
- Context and outcome

Benefits:
- Simulation state at any moment is rebuildable from event stream
- History recording comes for free
- Save/restore/branch becomes trivial

### Intervention System
The simulation can be paused at any moment to inject or modify:
- Agent behavior parameters
- Resource levels
- New agents or agent removal
- Environmental conditions

Interventions are logged as special events in the stream. The simulation doesn't distinguish between organic events and "divine intervention."

### Director AI (Simulation Ahead of Camera)
The simulation runs ahead of what's visualized. A director system watches raw events and decides what's worth showing.

#### What This Enables
- **Dramatic irony**: Director knows Agent Mira will betray her leader; can start following her earlier, showing suspicious meetings, building tension
- **Parallel storylines**: Interleave multiple developing threads; crosscut between them for pacing
- **Pacing control**: Fast-forward through uneventful periods; slow down when pressure builds
- **Focus heuristics**: System identifies "where's the interesting stuff?" — conflict brewing, trust networks fracturing, unusual cross-faction contact

### Narrative Summarization Pipeline

```
[raw events] 
    → pattern detection (narrative units)
    → arc identification ("betrayal sequence", "alliance forming", "faction decline")
    → dramatic summarization (LLM-assisted)
    → historical narrative output
```

Example:
- Raw: [event, event, event, event]
- Pattern: "Agent Mira gathered evidence against Faction Leader Corin over 3 months"
- Narrative: "The seeds of the Thornwood Rebellion were planted quietly..."

---

## Visualization

### Style
- Map-based view of the world
- Paper Mario-style sprites (2D characters with personality, more than dots)
- Agents move around the map in real-time

### Agent Sprites
With hundreds of agents, hand-crafting isn't feasible. Procedural generation with meaningful variation:
- Body shape and coloring
- Clothing reflecting faction and role
- Accessories that accumulate over time (scars from fights, stolen cloaks, insignia of rank)
- The sprite becomes a visual history of the agent

### Camera Behavior
- Follows the director AI's recommendations
- Can zoom to scenes of high drama
- Pulls back for faction-scale movements
- User can override to explore freely

---

## Open Questions

### Memory Weight
When an agent reads a written record, is it:
- Equivalent to being told by another agent (with source attribution: "the archive claims...")?
- More authoritative than oral testimony?
- Something else?

### Faction Beliefs
Do factions develop collective beliefs/values beyond individual member beliefs? If so, how do they form and change?

### Death and Succession
- What triggers leadership succession?
- Do factions have formal succession rules, or does it emerge from power dynamics?
- What happens to an agent's relationships when they die?

### Scope of Rituals
- How often do ritual readings occur?
- What's the selection pressure on what gets read?
- Can agents request specific entries be read?

### Inter-Faction Relations
- Can factions have formal treaties, or only individual relationships?
- How do factions declare war or peace?

---

## Technical Considerations

### Likely Stack
- **Simulation core**: Rust with Bevy ECS for performance and clean entity management
- **Event storage**: Append-only event log (could be simple file, SQLite, or dedicated event store)
- **Director AI**: Pattern matching over event streams; possibly ML-assisted drama detection
- **Narrative generation**: LLM integration for summarization and dramatic retelling
- **Visualization**: Bevy rendering with 2D sprite system

### Performance Targets
- Hundreds of agents simulating concurrently
- Real-time visualization at comfortable frame rate
- Simulation can run faster than real-time when visualization is paused or fast-forwarding

---

## Next Steps

1. Prototype core agent behavior loop with minimal rules
2. Implement trust and memory systems
3. Test emergence with small agent populations
4. Build event sourcing infrastructure
5. Develop drama detection heuristics
6. Create visualization layer
7. Integrate director AI
8. Add narrative summarization

---

*Document generated from design session, December 2024*
