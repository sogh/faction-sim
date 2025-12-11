# Implementation Prompts

Prompts for building the simulation engine in phases. Feed these to Claude Code sequentially. Each phase builds on the previous.

---

## Phase 1: Project Scaffolding

### Prompt 1.1: Initialize Project

```
Create a new Rust project for an emergent medieval simulation engine. Set up:

1. Cargo.toml with dependencies:
   - bevy_ecs (just the ECS, not full Bevy)
   - serde and serde_json for serialization
   - rand with the "small_rng" feature for reproducible randomness
   - chrono for timestamps

2. The directory structure from claude.md

3. A minimal main.rs that:
   - Accepts command line args for seed, tick count, and snapshot interval
   - Creates a Bevy ECS World and Schedule
   - Runs an empty simulation loop for N ticks
   - Prints tick progress every 100 ticks

Don't implement any systems yet — just the skeleton.
```

### Prompt 1.2: Core Components

```
Define the core ECS components in src/components/. Reference the behavioral rules doc for the exact fields.

Create:

1. agent.rs:
   - Agent marker component
   - Traits component (boldness, loyalty_weight, grudge_persistence, ambition, honesty, sociability, group_preference)
   - Needs component (food_security enum, social_belonging enum)
   - Goals component (vec of Goal structs with type, priority, optional target, optional expiry)

2. social.rs:
   - Relationship component (stores trust dimensions: reliability, alignment, capability)
   - Memory component (event reference, content summary, fidelity, source chain, emotional weight, tick created)
   - RelationshipGraph resource (maps agent pairs to relationships)
   - MemoryBank resource (maps agents to their memories)

3. faction.rs:
   - FactionMembership component (faction_id, role, status_level)
   - Faction resource (id, name, territory, hq_location, resources, leader, reader)
   - ArchiveEntry struct (id, author, subject, content, tick_written, times_read)
   - Archive resource per faction

4. world.rs:
   - Position component (location_id)
   - Location resource (id, type, controlling_faction, properties, resources)
   - Season enum and WorldState resource (current_tick, current_season, day)

Use serde derives on everything for JSON serialization.
```

---

## Phase 2: World Setup

### Prompt 2.1: Location and Faction Setup

```
Implement world initialization in src/setup/.

Create:

1. world.rs:
   - Function to create a medieval world map with ~15-20 locations
   - Location types: village, fields, forest, bridge, crossroads, hall
   - Some locations are faction HQs, some are neutral territory
   - Define adjacency/connectivity between locations

2. factions.rs:
   - Create 4 factions: Thornwood, Ironmere, Saltcliff, Northern Hold
   - Each faction has: territory (list of locations), HQ, starting resources (grain, iron, salt)
   - Each faction starts with empty archive

Output the world setup as JSON to verify the structure matches our schemas.
```

### Prompt 2.2: Agent Spawning

```
Implement agent spawning in src/setup/agents.rs.

Create:

1. A function to spawn N agents for a faction with:
   - Randomized traits (using seeded RNG) with reasonable distributions
   - Names generated from faction-appropriate name lists
   - Initial role assignment (1 leader, 1 reader, rest distributed among workers/specialists)
   - Starting location at faction HQ
   - Initial needs set to Secure/Integrated
   - No memories or relationships yet

2. A function to initialize starter relationships:
   - Everyone in a faction starts with mild positive trust toward faction-mates
   - Leader and Reader start with higher trust from others
   - Cross-faction relationships start at neutral/none

3. Spawn approximately:
   - 50-60 agents per faction (200-250 total)
   - Ensure role distribution makes sense (mostly laborers, few specialists)

Generate initial world snapshot to verify it matches our schema.
```

---

## Phase 3: Event System

### Prompt 3.1: Event Types and Logging

```
Implement the event system in src/events/.

Create:

1. types.rs:
   - EventType enum with all types from the schema (movement, communication, betrayal, loyalty, conflict, cooperation, faction, archive, ritual, resource, death, birth)
   - Subtype enums for each event type
   - Event struct matching the schema exactly:
     - event_id, timestamp, event_type, subtype
     - actors (primary, secondary, affected)
     - context (trigger, preconditions, location_description)
     - outcome (flexible per event type)
     - drama_tags, drama_score, connected_events
   - ActorSnapshot struct for capturing agent state at event time

2. logger.rs:
   - EventLogger resource that wraps a file handle
   - Append-only writes, one JSON event per line
   - Method to log an event (serializes and writes)
   - Flush on snapshot intervals

3. Helper function to create ActorSnapshot from current agent components

Write a test that creates and logs a sample movement event, then reads it back and verifies it parses correctly.
```

### Prompt 3.2: Snapshot Output

```
Implement world snapshot generation in src/output/.

Create:

1. schemas.rs:
   - All the structs needed to serialize a WorldSnapshot matching our schema
   - WorldSnapshot, FactionSnapshot, AgentSnapshot, RelationshipSnapshot, LocationSnapshot
   - ComputedMetrics including social network analysis stubs

2. snapshot.rs:
   - System that runs on snapshot intervals
   - Queries all entities and resources
   - Builds complete WorldSnapshot
   - Calculates computed metrics (faction power balance, defection risk, etc.)
   - Writes to snapshots/snap_{tick}.json

3. current_state.rs:
   - Similar but writes to current_state.json every tick (or every N ticks for performance)

Generate a snapshot of the initial world state and verify it matches the schema.
```

---

## Phase 4: Basic Agent Behavior

### Prompt 4.1: Perception and Need Updates

```
Implement the foundational systems in src/systems/.

Create:

1. perception.rs:
   - System that updates each agent's awareness of nearby agents
   - Query agents at same location
   - Store as a "VisibleAgents" component or resource
   
2. needs.rs:
   - System that updates agent needs based on world state
   - FoodSecurity: based on faction resources / faction size, with role modifier
   - SocialBelonging: based on recent interaction count, ritual attendance, trust received from faction-mates
   - State transitions should be gradual, not instant (use thresholds)

3. Add these systems to the schedule, running every tick.

Test by modifying faction resources and verifying agent food_security updates appropriately.
```

### Prompt 4.2: Movement Actions

```
Implement movement as the first action type.

Create:

1. actions/movement.rs:
   - MoveAction struct with destination
   - Precondition: destination is adjacent or reachable
   - Execution: update Position component, calculate travel time

2. systems/action/generate.rs:
   - Framework for generating valid actions
   - Start with just movement: generate possible destinations for each agent
   
3. systems/action/weight.rs:
   - Framework for weighting actions
   - Movement weights based on:
     - Scheduled patrol (if scout)
     - Return to HQ (if far and belonging low)
     - Move toward food (if desperate)
     - Random wandering (low base weight)

4. systems/action/select.rs:
   - Weighted random selection from action list
   - Include noise factor

5. systems/action/execute.rs:
   - Execute selected action
   - Generate and log movement event

Run the simulation for 100 ticks and verify agents are moving around, events are logging correctly.
```

---

## Phase 5: Communication and Memory

### Prompt 5.1: Memory System

```
Implement the memory system.

Create:

1. systems/memory.rs:
   - Memory decay: reduce fidelity/emotional_weight over time
   - Memory cleanup: remove memories below significance threshold
   - Query memories by subject, by emotional valence, by recency

2. Update social.rs if needed:
   - Methods to add memory with proper attribution
   - Methods to query "what do I know about agent X"
   - Methods to get "interesting memories" for sharing

Test memory decay over 1000 ticks, verify fidelity decreases as expected.
```

### Prompt 5.2: Communication Actions

```
Implement communication actions.

Create:

1. actions/communication.rs:
   - ShareMemoryAction: share a specific memory with target
   - SpreadRumorAction: share with potential distortion
   - LieAction: create false memory in target
   
2. Interaction targeting system:
   - Score potential targets using rules from behavioral doc
   - Factor in: same faction bonus, status preference, existing relationship, group_preference trait
   - Support both individual and group targeting

3. Weight calculations:
   - Gossip weight based on sociability, memory interestingness
   - Target selection based on status, faction, relationship
   - Lies based on honesty trait and motivation

4. Execution:
   - Transfer memory with fidelity loss
   - Update relationship slightly
   - Log communication event

5. Memory propagation effects:
   - When receiving negative info about third party, reduce trust in that party (scaled by trust in source)

Run simulation, verify memories spread through the faction over time.
```

---

## Phase 6: Trust and Relationships

### Prompt 6.1: Trust Dynamics

```
Implement trust update mechanics.

Create:

1. Enhance social.rs:
   - Trust update methods with asymmetric gains/losses
   - Separate updates for each dimension (reliability, alignment, capability)
   
2. systems/trust.rs (or add to existing):
   - Process events and update trust accordingly
   - Positive interactions: small trust gains
   - Broken promises: large reliability hit
   - Betrayal: catastrophic trust collapse + grudge formation

3. Grudge formation:
   - When trust drops below threshold after negative event, create revenge goal
   - Grudge persistence trait affects how long revenge goal stays active

4. Trust queries:
   - "Who do I trust most in my faction?"
   - "Who has betrayed me?"
   - "What's my overall sentiment toward faction X?"

Test by simulating a promise-break event and verifying trust updates correctly, grudge forms.
```

---

## Phase 7: Faction Mechanics

### Prompt 7.1: Archive System

```
Implement the faction archive system.

Create:

1. actions/archive.rs:
   - WriteEntryAction: agent writes memory to faction archive
   - ReadArchiveAction: agent reads entries, gains memories
   - DestroyEntryAction: agent removes an entry
   - ForgeEntryAction: agent creates false entry

2. Weight calculations:
   - Write when witnessed significant event and at HQ
   - Bias toward writing entries that reflect well on self
   - Destroy entries that embarrass self (modified by honesty)
   - Forge requires low honesty and motivation

3. Execution:
   - Entries stored in faction's Archive resource
   - Reading creates memories with "archive" attribution
   - Destruction removes entry
   - Forgery creates entry with false content

Test by having agents witness events, write entries, have others read them.
```

### Prompt 7.2: Ritual Reading System

```
Implement the ritual reading mechanic.

Create:

1. Ritual scheduling:
   - Each faction has a ritual every N ticks (e.g., every 500 ticks = ~1 week)
   - Ritual occurs at faction HQ

2. Reader behavior:
   - Reader selects entries to read using weighted selection
   - Bias toward: faction loyalty reinforcement, away from leader embarrassment
   - Reader's own biases affect selection

3. Attendance:
   - Agents at HQ attend automatically
   - Generate attendance event for each agent
   - Absent agents noted in event

4. Memory reinforcement:
   - Attendees get memory reinforcement for entries read
   - Boost fidelity and emotional weight of matching memories
   - Create new memories for entries they hadn't heard

5. Social belonging impact:
   - Attendance boosts belonging
   - Missing rituals erodes belonging

Run simulation with rituals, verify faction consensus emerges over time on repeatedly-read topics.
```

---

## Phase 8: Tension Detection

### Prompt 8.1: Tension System

```
Implement tension detection for the Director AI.

Create:

1. output/tension.rs:
   - Tension struct matching schema
   - TensionType enum with all types from schema
   - TensionStream resource holding active tensions

2. systems/tension.rs:
   - Run after events are processed
   - Detect patterns that indicate developing drama:
   
   Brewing Betrayal:
   - Agent trust in faction leader below threshold
   - Agent has cross-faction contact
   - Agent food_security or belonging degraded
   
   Resource Conflict:
   - Two factions with critical resources
   - Contested location between them
   
   Revenge Arc:
   - Agent has revenge goal
   - Target is accessible
   
   Faction Fracture:
   - Cluster of agents with shared negative sentiment toward leader
   - Low average belonging in faction

3. Tension lifecycle:
   - Create when pattern detected
   - Update severity as situation evolves
   - Resolve/remove when concluded

4. Write tensions.json on each update

Test by engineering a betrayal scenario, verify tension is detected and tracked.
```

---

## Phase 9: Remaining Actions

### Prompt 9.1: Resource and Social Actions

```
Implement remaining action categories.

Create:

1. actions/resource.rs:
   - WorkAction: increase faction resources slightly
   - TradeAction: exchange resources with other agent
   - StealAction: take resources with detection risk
   - HoardAction: personal reserves vs faction

2. actions/social.rs:
   - BuildTrustAction: slow trust improvement
   - CurryFavorAction: target higher status agents
   - GiftAction: faster trust building, costs resources
   - OstracizeAction: reduce target's belonging

3. Weight calculations following behavioral rules doc
4. Execution and event generation for each

Run simulation, verify resource dynamics and social maneuvering occur.
```

### Prompt 9.2: Faction and Conflict Actions

```
Implement faction and conflict actions.

Create:

1. actions/faction.rs:
   - DefectAction: leave faction, join another
   - ExileAction: remove another agent (requires authority)
   - ChallengeLeaderAction: initiate succession crisis
   - SupportLeaderAction: back current leader

2. actions/conflict.rs:
   - ArgueAction: relationship damage, possible resolution
   - FightAction: physical harm, high relationship damage
   - SabotageAction: damage target's resources/reputation
   - AssassinateAction: kill target (high risk, requires desperate/isolated state)

3. Weight calculations:
   - Defection requires degraded loyalty + external contact
   - Challenge requires ambition + support + weakened leader
   - Assassination requires extreme conditions (rare)

4. Conflict resolution:
   - Simple probability based on capability traits
   - Winner/loser affects trust, status, potentially death

Run simulation for extended period, verify these dramatic actions occur rarely but naturally.
```

---

## Phase 10: Polish and Tuning

### Prompt 10.1: Drama Scoring

```
Implement drama scoring for events.

Create:

1. events/drama.rs:
   - Function to calculate drama_score for any event
   - Factors:
     - Event type base score (betrayal high, movement low)
     - Agents involved (high status = more dramatic)
     - Relationships affected (close relationships = more dramatic)
     - Faction implications (affects whole faction = more dramatic)
     - Rarity (common actions score lower)
   
2. Drama tags assignment:
   - Automatic tags based on event properties
   - "faction_critical", "secret_meeting", "winter_crisis", etc.

3. Connected events tracking:
   - When event references previous events (e.g., memory shared), link them
   - Build event chains for narrative summarization

Test by generating various events and verifying scores match intuition.
```

### Prompt 10.2: Intervention System

```
Implement the intervention system for runtime modification.

Create:

1. interventions/mod.rs:
   - Watch interventions/ directory for JSON files
   - Intervention types:
     - ModifyAgent: change traits, needs, goals
     - ModifyFaction: change resources, leader
     - SpawnAgent: add new agent
     - KillAgent: remove agent
     - ModifyRelationship: change trust values
     - TriggerEvent: force an event to occur
   
2. Apply interventions at start of tick
3. Log interventions as special events in the event stream
4. Delete intervention file after applying

Test by:
- Running simulation
- Dropping an intervention file that makes an agent desperate
- Verifying behavior changes accordingly
```

### Prompt 10.3: Performance and Tuning

```
Optimize and add tuning infrastructure.

Create:

1. Tuning config file (tuning.toml or similar):
   - All the weight values from behavioral rules
   - Easy to modify without recompiling
   - Loaded at startup

2. Performance optimization:
   - Profile the simulation loop
   - Optimize hot paths (action generation, event logging)
   - Consider batch writes for event log

3. Statistics output:
   - Events per tick by type
   - Average action weights
   - Faction health metrics over time
   - Output as separate stats.json for analysis

4. Determinism verification:
   - Test that same seed produces identical event log
   - Add CI test for this

Run extended simulation (100k ticks), verify performance targets met, output useful for analysis.
```

---

## Validation Checklist

After completing all phases, verify:

- [ ] Simulation runs for 100k ticks without crashing
- [ ] Same seed produces identical output
- [ ] Events.jsonl is valid JSONL, each event parses correctly
- [ ] Snapshots match schema exactly
- [ ] Tensions are detected and updated
- [ ] Agents move, communicate, form relationships
- [ ] Memories propagate and decay
- [ ] Rituals occur on schedule
- [ ] Archive entries are written and read
- [ ] Betrayals/defections occur occasionally under right conditions
- [ ] Drama scores correlate with interesting events
- [ ] Interventions work correctly
- [ ] Performance targets met (1000+ ticks/second)
