# Learning System Implementation Prompts

This document contains sequential prompts for Claude Code CLI to implement the agent learning and cultural belief system. Execute these prompts in order, as each builds on the previous.

---

## Overview

The learning system adds three layers of discoverable knowledge:
1. **Physical World** - Hidden probabilities about resources, routes, locations
2. **Social Patterns** - Learnable patterns about agent/faction behavior  
3. **Cultural Beliefs** - Faction-specific norms that shape behavior

Key mechanisms:
- Agents experience outcomes and update personal beliefs
- Surprising discoveries can be written to faction books
- Ritual readings synchronize faction members toward archetype beliefs
- Heresy emerges when personal experience contradicts orthodoxy

---

## Phase 1: Foundation Types (sim-events)

### Prompt 1.1: Knowledge Claim Types

```
In the sim-events crate, create a new file `src/knowledge.rs` that defines the core knowledge claim types.

Requirements:
1. Define `KnowledgeClaim` struct with:
   - claim_id: String
   - topic: String (what this claim is about)
   - claim_type: KnowledgeClaimType enum
   - suggested_utility: f32 (the claimed value/danger)
   - author_id: String
   - author_faction: String
   - significance_score: f32 (how surprising was this)
   - timestamp_tick: u64
   - contradicts_archetype: bool
   - supporting_evidence: Vec<String> (event_ids that support this)

2. Define `KnowledgeClaimType` enum with variants:
   - ResourceDiscovery { location: String, resource_type: String }
   - RouteCondition { from: String, to: String, condition: String }
   - SocialPattern { about_faction: Option<String>, about_role: Option<String>, pattern: String }
   - Warning { threat_type: String, location: Option<String> }
   - Technique { activity: String, improvement: String }

3. Define `BeliefContent` enum for cultural beliefs:
   - ResponseNorm { trigger: String, required_response: String, failure_consequence: String }
   - ConditionalObligation { condition: String, obligation: String, enforced_by: EnforcementType }
   - CategoryJudgment { category: String, judgment: String, permanence: f32 }
   - SuccessionRule { rule: String, exceptions: Vec<String> }
   - TrustHeuristic { when: String, trust_delta: f32, applies_to: String }

4. Define `EnforcementType` enum:
   - SocialShaming
   - ViolentRetribution  
   - Exile
   - StatusLoss
   - NoEnforcement

5. Define `BeliefCategory` enum:
   - Honor
   - Reciprocity
   - Authority
   - Purity
   - Loyalty
   - Fairness

All types should derive Serialize, Deserialize, Clone, Debug, and PartialEq.

Export from lib.rs with `pub mod knowledge;`
```

### Prompt 1.2: Experience and Outcome Types

```
In sim-events, create `src/outcomes.rs` defining experience and action outcome types.

Requirements:
1. Define `Experience` struct:
   - experience_id: String
   - agent_id: String
   - tick: u64
   - action_taken: ActionType
   - context: ExperienceContext
   - prior_expectation: Option<PriorBelief>
   - actual_outcome: ActionOutcome
   - surprise_score: f32 (calculated from prior vs actual)

2. Define `ActionType` enum covering learnable actions:
   - Forage { location: String, target_resource: Option<String> }
   - Travel { from: String, to: String, route: Option<String> }
   - Trade { with_agent: String, offered: String, requested: String }
   - Negotiate { with_agent: String, negotiation_type: String }
   - Interact { with_agent: String, interaction_type: SocialInteractionType }
   - Craft { item: String, using: Vec<String> }
   - Rest { location: String }

3. Define `SocialInteractionType` enum:
   - TradeProposal
   - ThreatIssuance
   - AllianceOffer
   - InformationRequest
   - FavorRequest
   - Apology
   - Challenge
   - Bribe
   - Intimidation
   - Persuasion
   - Deception

4. Define `ExperienceContext` struct:
   - location: String
   - season: Season
   - time_of_day: TimeOfDay
   - weather: Option<WeatherType>
   - witnesses: Vec<String>
   - relevant_relationships: Vec<(String, f32)> // (agent_id, trust_level)

5. Define `PriorBelief` struct:
   - topic: String
   - expected_utility: f32
   - confidence: f32

6. Define `ActionOutcome` struct:
   - success: bool
   - utility_gained: f32
   - resources_changed: Vec<ResourceChange>
   - relationships_changed: Vec<RelationshipChange>
   - injuries: Option<InjuryOutcome>
   - discoveries: Vec<String>
   - side_effects: Vec<SideEffect>

7. Define supporting types: Season, TimeOfDay, WeatherType, ResourceChange, RelationshipChange, InjuryOutcome, SideEffect

Export from lib.rs.
```

### Prompt 1.3: Learning Event Types

```
In sim-events, extend `src/events.rs` (or create if needed) to add learning-related event types.

Requirements:
1. Add to EventType enum (or create it):
   - Learning with subtypes:
     - SurprisingDiscovery
     - BeliefReinforced
     - BeliefContradicted
     - KnowledgeShared
     - ClaimWrittenToBook
   - CulturalConflict with subtypes:
     - BeliefViolationWitnessed
     - HeresyDetected
     - DoctrinalDispute
     - NormEnforced

2. Define `LearningEventData` struct:
   - experience: Experience
   - belief_update: Option<BeliefUpdate>
   - knowledge_claim: Option<KnowledgeClaim>
   - shared_with: Vec<String>

3. Define `BeliefUpdate` struct:
   - topic: String
   - old_expected_utility: f32
   - new_expected_utility: f32
   - old_confidence: f32
   - new_confidence: f32
   - update_reason: BeliefUpdateReason

4. Define `BeliefUpdateReason` enum:
   - PersonalExperience
   - ToldByOther { source: String, credibility: f32 }
   - RitualReinforcement
   - ReasonedConclusion
   - ArchetypeDrift

5. Define `CulturalConflictEventData` struct:
   - belief_at_stake: String
   - violation_type: ViolationType
   - violator: String
   - witnesses: Vec<WitnessReaction>
   - consequences: Vec<ConflictConsequence>

6. Define `ViolationType` enum:
   - NormViolation { norm: String, action: String }
   - HereticalClaim { orthodox: String, heretical: String }
   - StatusViolation { expected_deference: String, actual_behavior: String }

7. Define `WitnessReaction` struct:
   - agent_id: String
   - judgment: String
   - trust_change: f32
   - will_spread: bool

8. Define `ConflictConsequence` enum:
   - StatusChange { agent: String, delta: f32 }
   - RelationshipChange { from: String, to: String, delta: f32 }
   - FactionTensionCreated { tension_type: String }
   - ViolenceTriggered { aggressor: String, target: String }

Ensure all types serialize cleanly to JSON matching the schema patterns in the design docs.
```

---

## Phase 2: Physical World Knowledge (sim-core)

### Prompt 2.1: World Property System

```
In sim-core, create `src/world/physical.rs` defining the hidden probability system for the physical world.

Requirements:
1. Define `PhysicalWorldKnowledge` struct containing:
   - resource_nodes: HashMap<String, ResourceNode>
   - routes: HashMap<(String, String), RouteProperties>
   - location_properties: HashMap<String, LocationProperties>
   - seasonal_effects: SeasonalEffectTable
   - time_effects: TimeOfDayEffectTable

2. Define `ResourceNode` struct:
   - node_id: String
   - location: String
   - resource_type: ResourceType enum (Food, Material, Medicinal, Luxury, Tool)
   - base_yield: YieldDistribution
   - base_danger: f32
   - depletion_rate: f32
   - regeneration_rate: f32
   - current_abundance: f32
   - last_harvested_tick: u64
   - modifiers: Vec<ResourceModifier>

3. Define `YieldDistribution` struct for probabilistic yields:
   - min: f32
   - max: f32  
   - mode: f32 (most likely value)
   - variance: f32
   
   Implement a `sample(&mut rng) -> f32` method that samples from this distribution (use triangular or beta distribution approximation)

4. Define `ResourceModifier` struct:
   - condition: WorldCondition
   - yield_multiplier: f32
   - danger_delta: f32
   - discovery_chance_delta: f32
   - discoverable: bool (can agents learn this pattern?)
   - discovery_difficulty: f32 (0.0-1.0, how many samples needed)

5. Define `WorldCondition` enum:
   - Season(Season)
   - TimeOfDay(TimeOfDay)
   - Weather(WeatherType)
   - AgentHasTrait { trait_name: String, comparison: Comparison, threshold: f32 }
   - AgentHasSkill { skill: String, min_level: f32 }
   - GroupSize { min: usize, max: Option<usize> }
   - ToolEquipped(String)
   - RecentEvent { event_type: String, within_ticks: u64 }
   - FactionControls { faction: String }
   - NearbyHostileFaction

6. Define `Comparison` enum: GreaterThan, LessThan, Equals, NotEquals

7. Implement `ResourceNode::resolve_harvest()` method:
   - Takes agent reference, current conditions, rng
   - Evaluates all modifiers whose conditions match
   - Calculates final yield multiplier and danger
   - Samples from yield distribution
   - Returns HarvestResult { yield_amount, danger_encountered, discoveries }

8. Create a few example resource nodes as test data:
   - Forest berry patches (seasonal, herbalism helps)
   - Iron deposits (tool required, dangerous)
   - River fishing spots (weather dependent)
```

### Prompt 2.2: Route Properties System

```
In sim-core, create or extend `src/world/routes.rs` with learnable route properties.

Requirements:
1. Define `RouteProperties` struct:
   - route_id: String
   - from_location: String
   - to_location: String
   - base_travel_ticks: u32
   - base_ambush_chance: f32
   - base_discovery_chance: f32 (chance of finding something interesting)
   - base_exposure_risk: f32 (weather/terrain danger)
   - terrain_type: TerrainType
   - requires_skill: Option<(String, f32)>
   - modifiers: Vec<RouteModifier>

2. Define `TerrainType` enum:
   - Road
   - Trail
   - Forest
   - Mountain
   - River
   - Marsh
   - Open

3. Define `RouteModifier` struct:
   - condition: WorldCondition (reuse from physical.rs)
   - travel_time_multiplier: f32
   - ambush_chance_delta: f32
   - visibility_delta: f32 (how visible is traveler)
   - exposure_delta: f32
   - discoverable: bool
   - discovery_difficulty: f32

4. Implement `RouteProperties::resolve_travel()` method:
   - Takes agent, current conditions, rng
   - Evaluates matching modifiers
   - Calculates total travel time
   - Rolls for ambush, discovery, exposure events
   - Returns TravelResult { 
       ticks_taken,
       ambush_occurred: Option<AmbushDetails>,
       discovered: Vec<String>,
       exposure_damage: f32,
       visibility_events: Vec<String> // who saw the traveler
     }

5. Create example routes:
   - Safe road between allied villages (fast, visible)
   - Mountain pass (slow in winter, ambush risk)
   - Forest path (hidden but slow, discovery chances)
   - River crossing (weather dependent danger)
```

### Prompt 2.3: Social Interaction Outcomes

```
In sim-core, create `src/world/social_patterns.rs` defining probabilistic social interaction outcomes.

Requirements:
1. Define `SocialInteractionDefinition` struct:
   - interaction_type: SocialInteractionType
   - base_success_chance: f32
   - base_outcomes: OutcomeDistribution
   - modifiers: Vec<SocialModifier>
   - trust_visibility: TrustDimension (which trust dimension most affects this)
   - status_factor: f32 (how much status difference matters)

2. Define `OutcomeDistribution` struct:
   - outcomes: Vec<(InteractionOutcome, f32)> // outcome, probability
   
   Implement `sample(&mut rng) -> InteractionOutcome`

3. Define `InteractionOutcome` enum:
   - Success { degree: f32 }
   - Failure { reason: String }
   - PartialSuccess { achieved: String, failed: String }
   - CounterOffer { terms: String }
   - Rejection { hostility: f32 }
   - Betrayal { action: String }
   - Escalation { to: String }

4. Define `SocialModifier` struct:
   - condition: SocialCondition
   - success_delta: f32
   - outcome_weight_changes: Vec<(String, f32)> // shift certain outcomes
   - trust_impact_multiplier: f32

5. Define `SocialCondition` enum:
   - RelationshipAbove { dimension: TrustDimension, threshold: f32 }
   - RelationshipBelow { dimension: TrustDimension, threshold: f32 }
   - SameFaction
   - AlliedFactions
   - HostileFactions
   - InitiatorTrait { trait_name: String, comparison: Comparison, threshold: f32 }
   - TargetTrait { trait_name: String, comparison: Comparison, threshold: f32 }
   - WitnessesPresent { min_count: usize }
   - LocationType { location_type: String }
   - StatusDifference { initiator_higher: bool, min_gap: f32 }
   - RecentInteraction { interaction_type: String, was_positive: bool, within_ticks: u64 }
   - TargetInNeed { need_type: String }

6. Define `TrustDimension` enum: Reliability, Alignment, Capability

7. Implement `SocialInteractionDefinition::resolve()` method:
   - Takes initiator, target, context, rng
   - Evaluates relationship states
   - Applies matching modifiers
   - Samples from modified outcome distribution
   - Returns SocialInteractionResult {
       outcome,
       trust_changes: Vec<TrustChange>,
       status_changes: Vec<StatusChange>,
       witnesses_reactions: Vec<WitnessReaction>,
       information_revealed: Vec<String>
     }

8. Create definitions for key interactions:
   - Bribery (affected by target loyalty, witnesses, desperation)
   - Trade proposal (affected by fairness reputation, past deals)
   - Challenge to duel (affected by honor beliefs, status)
   - Information request (affected by trust, faction relations)
   - Intimidation (affected by capability trust, status)
```

---

## Phase 3: Agent Belief System (sim-core)

### Prompt 3.1: Personal Belief Structure

```
In sim-core, create `src/agent/belief_system.rs` defining individual agent beliefs.

Requirements:
1. Define `AgentBeliefSystem` struct:
   - personal_beliefs: HashMap<String, PersonalBelief>
   - cultural_beliefs: HashMap<String, CulturalBeliefStance>
   - faction_alignment: f32 (0.0-1.0, how closely they follow archetype)
   - heretical_tendencies: Vec<HereticalBelief>
   - belief_sources: HashMap<String, BeliefSource>

2. Define `PersonalBelief` struct:
   - topic: String
   - expected_utility: f32
   - confidence: f32 (0.0-1.0)
   - sample_count: u32 (how many experiences inform this)
   - last_updated_tick: u64
   - contradicting_experiences: u32
   - confirming_experiences: u32

3. Define `CulturalBeliefStance` struct:
   - belief_id: String
   - conviction: f32 (0.0-1.0, how strongly held)
   - source: BeliefSource
   - personal_interpretation: Option<String>
   - violations_witnessed: u32
   - times_enforced: u32

4. Define `BeliefSource` enum:
   - RaisedWith { faction: String }
   - RitualReinforcement { count: u32, last_tick: u64 }
   - PersonalExperience { event_ids: Vec<String> }
   - TaughtBy { agent_id: String, credibility: f32 }
   - ReasonedConclusion { from_beliefs: Vec<String> }
   - Converted { by_agent: String, tick: u64 }

5. Define `HereticalBelief` struct:
   - orthodox_belief_id: String
   - personal_variant: String
   - divergence_reason: String
   - supporting_experiences: Vec<String>
   - hidden: bool (do they conceal this?)
   - conviction: f32

6. Implement core methods:
   
   `fn get_expected_utility(&self, topic: &str) -> Option<(f32, f32)>`
   - Returns (expected_utility, confidence) if belief exists
   
   `fn update_from_experience(&mut self, experience: &Experience) -> BeliefUpdate`
   - Calculate surprise score
   - Update or create personal belief
   - Track contradictions/confirmations
   - Return what changed
   
   `fn get_cultural_stance(&self, belief_id: &str) -> Option<&CulturalBeliefStance>`
   
   `fn check_action_against_beliefs(&self, action: &ActionType) -> Vec<BeliefConflict>`
   - Returns any cultural beliefs this action would violate
   
   `fn apply_ritual_reinforcement(&mut self, reinforced_beliefs: &[String], archetype: &FactionArchetype)`
   - Pull personal beliefs toward archetype values
   - Strengthen conviction on reinforced cultural beliefs
   - Handle conflict between heretical beliefs and reinforcement

7. Implement belief update math:
   - Bayesian-inspired update: new_belief = old_belief + learning_rate * (actual - expected)
   - Confidence increases with consistent experiences, decreases with variance
   - Learning rate affected by agent traits (memory_strength, stubbornness)
```

### Prompt 3.2: Belief-Based Decision Making

```
In sim-core, create or extend `src/agent/decision.rs` to integrate beliefs into decisions.

Requirements:
1. Define `BeliefBasedDecision` trait that agents implement:
   
   ```rust
   pub trait BeliefBasedDecision {
       fn evaluate_action(&self, action: &ActionType, context: &DecisionContext) -> ActionEvaluation;
       fn get_belief_conflicts(&self, action: &ActionType) -> Vec<BeliefConflict>;
       fn would_share_experience(&self, experience: &Experience) -> ShareDecision;
   }
   ```

2. Define `ActionEvaluation` struct:
   - expected_utility: f32
   - confidence: f32
   - belief_conflicts: Vec<BeliefConflict>
   - social_risk: f32 (risk of reputation damage)
   - physical_risk: f32
   - information_value: f32 (value of learning from this action)

3. Define `BeliefConflict` struct:
   - belief_id: String
   - conflict_type: ConflictType
   - severity: f32
   - predicted_consequence: String

4. Define `ConflictType` enum:
   - ViolatesNorm { norm: String }
   - ContradictsFactionBelief { belief: String }
   - BetraysRelationship { with: String }
   - RisksStatus
   - ViolatesPersonalCode

5. Define `ShareDecision` struct:
   - should_share: bool
   - share_with_book: bool
   - share_with_agents: Vec<String>
   - reason: String

6. Implement decision logic:
   
   `fn evaluate_action()`:
   - Get expected utility from beliefs
   - Adjust for confidence (low confidence = explore more)
   - Check for cultural belief conflicts
   - Factor in personality traits:
     - Bold agents discount risk
     - Loyal agents weight faction beliefs higher
     - High memory_strength agents trust their experiences more

   `fn would_share_experience()`:
   - Calculate surprise score
   - If below threshold, don't share
   - Check if agent has standing to write to book (role-based)
   - Consider who would benefit from knowing
   - Consider if sharing contradicts faction beliefs (risky)

7. Implement exploration vs exploitation logic:
   - Agents with high confidence exploit known-good options
   - Agents with low confidence or "boldness" trait explore
   - Information value calculation: how much would we learn?
```

### Prompt 3.3: Heresy Detection and Management

```
In sim-core, add heresy-related logic to the belief system.

Requirements:
1. Add to `AgentBeliefSystem`:
   
   `fn consider_heresy(&mut self, belief_id: &str, experience: &Experience, traits: &AgentTraits) -> Option<HereticalBelief>`
   - Called when experience contradicts archetype belief
   - Probability based on:
     - Number of contradictions vs confirmations
     - Agent's loyalty_weight trait (high = resist heresy)
     - Agent's boldness (high = more likely to form independent beliefs)
     - Severity of contradiction
   - If heresy forms, decide if hidden based on boldness

   `fn heresy_strength(&self, belief_id: &str) -> f32`
   - Returns 0.0 if orthodox, up to 1.0 for strong heresy
   - Based on conviction in heretical belief vs cultural stance

   `fn would_express_heresy(&self, context: &SocialContext) -> bool`
   - Would agent voice heretical belief in this context?
   - Affected by: witnesses present, status of listeners, hidden flag
   - Bold agents more likely to speak up
   - Desperate agents might reveal heresy seeking allies

2. Define `HeresyEvent` for when heresy is expressed:
   - heretic: String
   - belief_id: String
   - heretical_content: String
   - context: SocialContext
   - witnesses: Vec<String>

3. Add witness reaction logic:
   
   `fn react_to_heresy(&self, heresy: &HeresyEvent, own_beliefs: &AgentBeliefSystem) -> WitnessReaction`
   - If witness also holds similar heresy (hidden or not): sympathetic
   - If witness is strong orthodox believer: hostile
   - If witness is uncertain (low conviction): intrigued
   - Reaction affects trust toward heretic

4. Implement heresy spread mechanics:
   - Heretics may seek each other out (recognize fellow travelers)
   - Expressed heresy can convert others (especially low-conviction)
   - Creates "heresy networks" within factions

5. Add heresy to tension detection:
   - Track agents with significant heretical beliefs
   - Detect when heretical beliefs cluster (potential schism)
   - Detect when heretic is approaching threshold to express publicly
```

---

## Phase 4: Faction Archetype and Book System (sim-core)

### Prompt 4.1: Faction Archetype

```
In sim-core, create `src/faction/archetype.rs` defining the faction's canonical beliefs.

Requirements:
1. Define `FactionArchetype` struct:
   - faction_id: String
   - canonical_beliefs: HashMap<String, CanonicalBelief>
   - utility_weights: HashMap<String, f32> // faction's learned knowledge about world
   - cultural_norms: Vec<CulturalNorm>
   - founding_mythology: Vec<String> // core beliefs that rarely change
   - challenge_threshold: f32 // how much contradiction needed to update

2. Define `CanonicalBelief` struct:
   - topic: String
   - official_utility: f32 // faction's official stance on utility
   - confidence: f32
   - authority_source: CanonicalSource
   - last_updated_tick: u64
   - challenge_count: u32 // how often has this been contradicted
   - reinforcement_count: u32

3. Define `CanonicalSource` enum:
   - FounderTradition
   - RitualCanonization { tick: u64, claim_id: String }
   - LeaderDecree { leader_id: String, tick: u64 }
   - ConsensusEmergence { supporting_claims: Vec<String> }

4. Define `CulturalNorm` struct:
   - norm_id: String
   - category: BeliefCategory
   - content: BeliefContent
   - origin: NormOrigin
   - enforcement_strength: f32
   - violations_recorded: u32
   - last_enforced_tick: Option<u64>

5. Define `NormOrigin` enum:
   - Founding
   - LeaderEstablished { leader: String }
   - EmergentConsensus
   - ImportedFrom { faction: String }
   - ReactiveToEvent { event_id: String }

6. Implement archetype methods:

   `fn get_official_stance(&self, topic: &str) -> Option<&CanonicalBelief>`
   
   `fn challenge_belief(&mut self, topic: &str, contradicting_claim: &KnowledgeClaim) -> ChallengeResult`
   - Increment challenge count
   - If challenges exceed threshold relative to reinforcements, update belief
   - Return whether belief changed
   
   `fn reinforce_belief(&mut self, topic: &str)`
   - Increment reinforcement count
   - Used during ritual readings
   
   `fn add_new_belief(&mut self, claim: &KnowledgeClaim, source: CanonicalSource)`
   - Add new canonical belief from knowledge claim
   - Set initial confidence based on claim significance
   
   `fn check_norm_violation(&self, action: &ActionType, actor: &Agent) -> Option<NormViolation>`
   - Check if action violates any cultural norm
   - Return details of violation if found

7. Create initial archetypes for test factions:
   - Thornwood: hospitality, promise-keeping, agricultural knowledge
   - Ironmere: martial honor, challenge culture, strength knowledge
   - Saltcliff: contract law, trade knowledge, profit virtue
```

### Prompt 4.2: Faction Book System

```
In sim-core, create `src/faction/book.rs` implementing the knowledge claim book.

Requirements:
1. Define `FactionBook` struct:
   - faction_id: String
   - claims: Vec<StoredClaim>
   - pending_claims: Vec<KnowledgeClaim> // awaiting next ritual
   - max_claims: usize // book has limited space
   - write_permissions: WritePermissions

2. Define `StoredClaim` struct:
   - claim: KnowledgeClaim
   - status: ClaimStatus
   - times_read: u32
   - last_read_tick: Option<u64>
   - added_tick: u64
   - disputed_by: Vec<String> // agent_ids who dispute this

3. Define `ClaimStatus` enum:
   - Pending // not yet canonized
   - Canonized // officially part of faction knowledge
   - Disputed // multiple contradicting claims exist  
   - Superseded { by: String } // replaced by newer claim
   - Expunged { by: String, reason: String } // deliberately removed

4. Define `WritePermissions` struct:
   - roles_can_write: Vec<String>
   - traits_can_write: Vec<(String, f32)> // trait above threshold grants permission
   - leader_can_grant: bool
   - minimum_standing: f32 // status threshold to write

5. Implement book methods:

   `fn can_write(&self, agent: &Agent) -> bool`
   - Check role and traits against permissions
   
   `fn add_claim(&mut self, claim: KnowledgeClaim) -> Result<(), BookError>`
   - Validate agent can write
   - Check for contradicting existing claims
   - Add to pending (will be considered at ritual)
   - If book full, return error or queue for replacement
   
   `fn get_claims_for_reading(&self, count: usize, reader: &Agent) -> Vec<&StoredClaim>`
   - Reader selects which claims to read at ritual
   - Selection influenced by reader's personal beliefs and faction politics
   - Return most significant unread, plus reinforcement of key beliefs
   
   `fn mark_read(&mut self, claim_id: &str, tick: u64)`
   
   `fn dispute_claim(&mut self, claim_id: &str, disputer: &str, counter_evidence: Option<String>)`
   
   `fn canonize_pending(&mut self, claim_id: &str, tick: u64)`
   - Move from pending to canonized
   - Called during ritual when claim is formally accepted
   
   `fn expunge_claim(&mut self, claim_id: &str, by: &str, reason: &str)`
   - Mark claim as expunged (historical record kept but no longer read)

6. Implement book politics:
   - Track who wrote what
   - Allow disputes to be recorded
   - Reader selection algorithm should be tunable
```

### Prompt 4.3: Ritual Reading System

```
In sim-core, create `src/faction/ritual.rs` implementing the ritual reading mechanics.

Requirements:
1. Define `RitualReading` struct:
   - ritual_id: String
   - faction_id: String
   - reader_id: String
   - location: String
   - tick: u64
   - attendees: Vec<String>
   - absent_members: Vec<(String, AbsenceReason)>
   - claims_read: Vec<String> // claim_ids
   - claims_skipped: Vec<String>
   - pending_canonized: Vec<String>
   - disputes_raised: Vec<DisputeRecord>

2. Define `AbsenceReason` enum:
   - TooFarAway
   - Hostile { with: String }
   - Injured
   - OnMission
   - Deliberately_Avoided // suspicious
   - Dead

3. Define `DisputeRecord` struct:
   - disputer_id: String
   - claim_id: String
   - counter_claim: Option<KnowledgeClaim>
   - resolution: Option<DisputeResolution>

4. Define `DisputeResolution` enum:
   - DisputerSilenced
   - ClaimAmended
   - ClaimExpunged
   - DisputeTabled
   - SchismFormed // faction splits over this

5. Implement `RitualReadingSystem`:

   `fn can_hold_ritual(&self, faction: &Faction, tick: u64) -> RitualPrerequisites`
   - Check reader alive
   - Check no active raid
   - Check minimum attendees
   - Return what's blocking if can't hold
   
   `fn begin_ritual(&mut self, faction: &Faction, tick: u64) -> RitualReading`
   - Identify reader
   - Gather attendees (based on location, relationships)
   - Reader selects claims to read
   
   `fn process_reading(&mut self, ritual: &mut RitualReading, book: &mut FactionBook, archetype: &mut FactionArchetype)`
   - For each claim read:
     - Reinforce in archetype
     - Mark read in book
     - Canonize if pending
   - For skipped claims:
     - Do not reinforce (organic forgetting)
   
   `fn apply_to_attendees(&self, ritual: &RitualReading, agents: &mut [Agent], archetype: &FactionArchetype) -> Vec<AgentRitualEffect>`
   - Don't update immediately - tag agents with NeedsRitualSync
   - Return list of agents tagged
   
   `fn process_disputes(&mut self, ritual: &mut RitualReading, book: &mut FactionBook)`
   - Allow attendees to raise disputes
   - Resolve based on relative status, evidence
   
   `fn emit_ritual_event(&self, ritual: &RitualReading) -> Event`
   - Create event for logging

6. Implement the deferred sync system (from Gemini's suggestion):

   Define `NeedsRitualSync` component:
   - faction_id: String
   - reinforced_beliefs: Vec<String>
   - ritual_tick: u64
   
   `fn tick_ritual_sync(agents_needing_sync: Query<With<NeedsRitualSync>>, archetype: &FactionArchetype)`
   - Process N agents per tick (configurable, default 5-10)
   - For each agent: pull beliefs toward archetype
   - Remove component when done
   - This spreads CPU load across many ticks
```

---

## Phase 5: Learning Systems (sim-core)

### Prompt 5.1: Experience Processing System

```
In sim-core, create `src/systems/learning.rs` implementing the core learning loop.

Requirements:
1. Define `LearningSystem` struct:
   - significance_threshold: f32 // minimum surprise to be notable
   - sharing_cooldown_ticks: u64 // don't spam shares
   - book_write_cooldown_ticks: u64
   - max_beliefs_per_agent: usize

2. Implement the main learning loop:

   `fn process_action_outcome(&mut self, agent: &mut Agent, action: &ActionType, outcome: &ActionOutcome, context: &ExperienceContext) -> LearningResult`
   
   Steps:
   a. Get agent's prior belief about this action/topic
   b. Calculate surprise score: |actual_utility - expected_utility| / confidence
   c. Create Experience record
   d. Update agent's personal beliefs via belief_system.update_from_experience()
   e. If surprise > threshold:
      - Consider writing to book
      - Consider sharing with nearby agents
   f. Check for heresy formation if contradicts faction archetype
   g. Return LearningResult with all changes

3. Define `LearningResult` struct:
   - experience: Experience
   - belief_updated: bool
   - new_belief_formed: bool
   - surprise_score: f32
   - written_to_book: Option<KnowledgeClaim>
   - shared_with: Vec<String>
   - heresy_formed: Option<HereticalBelief>
   - events_to_emit: Vec<Event>

4. Implement surprise calculation:

   `fn calculate_surprise(&self, prior: Option<&PersonalBelief>, actual_utility: f32) -> f32`
   - If no prior belief, return 1.0 (maximum surprise)
   - Otherwise: |actual - expected| * (1.0 / confidence)
   - Clamp to 0.0-1.0

5. Implement sharing decisions:

   `fn should_write_to_book(&self, agent: &Agent, experience: &Experience, surprise: f32, book: &FactionBook) -> bool`
   - Check surprise threshold
   - Check agent has write permission
   - Check cooldown
   - Check if contradicts archetype (risky to write)
   - Bold agents more likely to write controversial claims
   
   `fn select_agents_to_tell(&self, agent: &Agent, experience: &Experience, nearby_agents: &[&Agent]) -> Vec<String>`
   - Filter to agents the agent trusts enough to share with
   - Prioritize same-faction
   - Consider if information is sensitive
   - Return agent_ids to share with

6. Implement the experience-to-claim conversion:

   `fn create_knowledge_claim(&self, agent: &Agent, experience: &Experience, surprise: f32) -> KnowledgeClaim`
   - Extract topic from action type
   - Set suggested utility from actual outcome
   - Calculate significance from surprise
   - Check if contradicts archetype
```

### Prompt 5.2: Knowledge Propagation System

```
In sim-core, create `src/systems/knowledge_spread.rs` for inter-agent knowledge transfer.

Requirements:
1. Define `KnowledgePropagationSystem` struct:
   - fidelity_decay_per_hop: f32 // secondhand info less reliable
   - max_chain_length: usize // after N hops, info too degraded
   - faction_trust_bonus: f32 // same faction = more credible

2. Define `SecondhandKnowledge` struct:
   - original_topic: String
   - original_event_id: Option<String>
   - content: String
   - suggested_utility: f32
   - source_chain: Vec<SourceHop>
   - current_fidelity: f32

3. Define `SourceHop` struct:
   - agent_id: String
   - agent_name: String
   - tick: u64
   - relationship_to_next: f32 // how much next agent trusted this one

4. Implement propagation:

   `fn share_experience(&mut self, from: &Agent, to: &mut Agent, experience: &Experience) -> ShareResult`
   - Create SecondhandKnowledge from experience
   - Calculate initial fidelity (firsthand = 1.0)
   - Apply to recipient

   `fn share_secondhand(&mut self, from: &Agent, to: &mut Agent, knowledge: &SecondhandKnowledge) -> ShareResult`
   - Add hop to source chain
   - Decay fidelity
   - If fidelity below threshold, recipient gets vague memory
   - Apply to recipient

5. Implement recipient belief update:

   `fn apply_shared_knowledge(&self, recipient: &mut Agent, knowledge: &SecondhandKnowledge, source: &Agent) -> BeliefUpdateResult`
   - Calculate credibility:
     - Base = fidelity
     - Multiplied by trust in immediate source
     - Bonus if same faction
     - Penalty if source is known to be unreliable
   - If credibility > threshold:
     - Update or create belief with reduced confidence
     - Tag source in belief for later re-evaluation
   - Return what changed

6. Implement rumor mechanics:

   `fn check_for_distortion(&self, knowledge: &SecondhandKnowledge, sharer: &Agent, rng: &mut impl Rng) -> SecondhandKnowledge`
   - Small chance of accidental distortion (misremembering)
   - Larger chance if sharer has low memory_strength trait
   - Deliberate distortion if sharer has motive (grudge, benefit)
   
   `fn detect_deliberate_lie(&self, recipient: &Agent, sharer: &Agent, knowledge: &SecondhandKnowledge) -> bool`
   - Based on recipient's trust in sharer
   - Based on plausibility of claim
   - If detected, damages trust significantly

7. Create event emission:

   `fn emit_share_event(&self, from: &Agent, to: &Agent, knowledge: &SecondhandKnowledge, accepted: bool) -> Event`
```

### Prompt 5.3: Belief Drift System

```
In sim-core, create `src/systems/belief_drift.rs` handling gradual belief changes.

Requirements:
1. Define `BeliefDriftSystem` struct:
   - drift_rate_base: f32 // how fast beliefs change without reinforcement
   - archetype_pull_strength: f32 // how strongly rituals pull toward orthodoxy
   - isolation_acceleration: f32 // lonely agents drift faster
   - social_conformity_factor: f32 // tendency to match nearby agents

2. Implement periodic drift:

   `fn process_drift_tick(&mut self, agent: &mut Agent, faction_archetype: &FactionArchetype, social_context: &SocialContext)`
   
   For each personal belief:
   - If not reinforced recently, confidence decays slightly
   - Very old beliefs with low confidence may be forgotten
   
   For faction alignment:
   - If attending rituals regularly, pull toward archetype
   - If absent from rituals, drift away
   - If holding heresies, alignment decreases

3. Implement social conformity:

   `fn apply_social_pressure(&mut self, agent: &mut Agent, nearby_agents: &[&Agent])`
   - Agents adjust beliefs slightly toward those they trust
   - Stronger effect in same faction
   - Can reinforce both orthodox and heretical beliefs
   - Creates belief clusters naturally

4. Implement isolation effects:

   `fn check_isolation(&self, agent: &Agent, social_graph: &SocialGraph) -> IsolationLevel`
   - Count recent social interactions
   - Check trust network density
   - Return isolation level
   
   `fn apply_isolation_effects(&mut self, agent: &mut Agent, isolation: IsolationLevel)`
   - Isolated agents:
     - Drift away from archetype faster
     - May develop unique beliefs
     - More susceptible to conversion if contacted
     - May become depressed (affects other behaviors)

5. Implement heresy progression:

   `fn progress_heresies(&mut self, agent: &mut Agent, tick: u64)`
   - Hidden heresies that keep getting confirmed grow stronger
   - Heresies that contradict experiences weaken
   - Strong heresies may become unhidden (agent starts expressing them)
   - Very strong heresies may trigger defection consideration

6. Implement belief death:

   `fn cull_weak_beliefs(&mut self, agent: &mut Agent)`
   - Remove beliefs with confidence below threshold
   - Remove beliefs not accessed in long time
   - Keep max_beliefs limit respected
   - Preference to keep faction-relevant beliefs
```

---

## Phase 6: Director Integration

### Prompt 6.1: Doctrinal Tension Detection

```
In the director crate, create `src/tensions/doctrinal.rs` for detecting belief-based tensions.

Requirements:
1. Define `DoctrinalTensionDetector` struct:
   - heresy_threshold: f32 // when is heresy worth tracking
   - schism_threshold: usize // how many heretics before schism tension
   - orthodoxy_challenge_threshold: u32 // challenges to archetype

2. Define new tension types to add to existing TensionType enum:
   - HeresyRising { heretic: String, belief: String }
   - DoctrinalDispute { belief: String, factions_involved: Vec<String> }
   - OrthodoxyChallenged { faction: String, belief: String, challenges: u32 }
   - SchismBrewing { faction: String, orthodox_camp: Vec<String>, heretic_camp: Vec<String> }
   - ProphetRising { agent: String, new_doctrine: String }

3. Implement detection:

   `fn detect_heresy_tensions(&self, faction: &Faction, agents: &[&Agent]) -> Vec<Tension>`
   - Find agents with significant heresies
   - Check if heresies are spreading (shared by multiple agents)
   - Check if heretics are gaining status (dangerous to orthodoxy)
   - Create tensions for significant threats

   `fn detect_schism_potential(&self, faction: &Faction, agents: &[&Agent]) -> Option<Tension>`
   - Cluster agents by belief similarity
   - If two distinct clusters emerge, potential schism
   - Calculate likelihood based on cluster sizes, conviction levels

   `fn detect_orthodoxy_erosion(&self, archetype: &FactionArchetype) -> Option<Tension>`
   - Check beliefs with high challenge counts
   - If challenges > reinforcements * threshold, orthodoxy eroding
   - Create tension suggesting faction belief may change

4. Implement narrative hooks for belief tensions:

   `fn generate_heresy_hooks(&self, tension: &HeresyTension) -> Vec<String>`
   Examples:
   - "Once the faction's most devout, {heretic} now questions the old ways"
   - "The Reader doesn't know that three attendees no longer believe"
   - "If {heretic} speaks at the next ritual, everything could change"

5. Implement camera recommendations:

   `fn recommend_camera_for_belief_tension(&self, tension: &Tension) -> CameraRecommendation`
   - For heresy: follow heretic, watch for confession moments
   - For schism: frame both camps, especially at rituals
   - For orthodoxy erosion: focus on challenges being raised
```

### Prompt 6.2: Learning Narrative Patterns

```
In the director crate, create `src/patterns/learning_arcs.rs` for learning-related narrative patterns.

Requirements:
1. Define `LearningPatternMatcher` struct:
   - patterns: Vec<LearningPattern>

2. Define `LearningPattern` struct:
   - pattern_id: String
   - name: String
   - description: String
   - event_sequence: Vec<LearningPatternStep>
   - max_gap_ticks: u64
   - drama_multiplier: f32
   - narrative_template: String

3. Define narrative patterns:

   Pattern: "Discovery to Heresy"
   - Agent has surprising discovery
   - Discovery contradicts faction belief
   - Agent writes to book OR shares with others
   - Faction/Reader rejects claim
   - Agent becomes heretical
   - Heresy spreads OR agent is silenced
   
   Pattern: "False Prophet"
   - Agent writes false/mistaken claim to book
   - Claim gets canonized at ritual
   - Faction acts on false belief
   - Negative consequences occur
   - Original author blamed or belief corrected
   
   Pattern: "Knowledge Asymmetry"
   - One faction learns valuable information
   - Other factions don't know
   - Knowing faction exploits advantage
   - Eventually others discover or suffer
   
   Pattern: "Cultural Collision"
   - Agent from Faction A witnesses Faction B norm
   - Norm conflicts with A's beliefs
   - Agent must navigate or conflict occurs
   - Resolution shapes future relations
   
   Pattern: "The Convert"
   - Agent exposed to new beliefs (captured, traveled)
   - New beliefs resonate more than original
   - Agent returns to faction changed
   - Tension between old identity and new beliefs

4. Implement pattern detection:

   `fn detect_active_patterns(&self, events: &[Event], agents: &[&Agent]) -> Vec<ActivePattern>`
   - Scan recent events for pattern starts
   - Track in-progress patterns
   - Predict likely completions

5. Implement drama scoring for learning events:

   `fn score_learning_event(&self, event: &LearningEvent, context: &NarrativeContext) -> f32`
   - High score if contradicts viewer-known faction beliefs
   - High score if viewer knows consequences others don't
   - High score if part of active pattern
   - Bonus for involving tracked characters
```

---

## Phase 7: Testing and Data

### Prompt 7.1: Test World Data

```
In sim-core, create `src/test_data/mod.rs` with comprehensive test world setup.

Requirements:
1. Create function `create_test_world() -> PhysicalWorldKnowledge`:
   
   Resource nodes:
   - Thornwood Forest berries (seasonal, herbalism bonus)
   - Eastern Iron deposits (tool required, danger)
   - River fishing (weather dependent)
   - Mountain herbs (rare, high value, dangerous)
   - Marsh reeds (craft material, seasonal)
   
   Routes:
   - Main road (safe, fast, visible)
   - Forest path (hidden, discovery chance)
   - Mountain pass (seasonal danger)
   - River crossing (weather dependent)
   - Smuggler's trail (hidden, ambush risk)

2. Create function `create_test_factions() -> Vec<FactionArchetype>`:

   Thornwood (agricultural):
   - Canonical beliefs: hospitality sacred, promises binding, patience virtue
   - Norms: guest-right, oath-keeping, seasonal festivals
   - Known knowledge: farming, local flora, weather signs
   
   Ironmere (martial):
   - Canonical beliefs: strength legitimacy, challenges must answer, death before dishonor
   - Norms: duel culture, victory celebrations, weapon care
   - Known knowledge: combat, metalwork, territorial threats
   
   Saltcliff (mercantile):
   - Canonical beliefs: contracts absolute, profit virtue, information power
   - Norms: witnessed agreements, debt repayment, trade courtesy
   - Known knowledge: trade routes, goods values, negotiation
   
   Northern Hold (survivalist):
   - Canonical beliefs: self-reliance, winter preparation, suspicion of outsiders
   - Norms: resource sharing within faction, isolation preference
   - Known knowledge: harsh climate survival, preservation, hunting

3. Create function `create_test_agents(factions: &[FactionArchetype]) -> Vec<Agent>`:
   - 10-15 agents per faction
   - Varied traits (some loyal, some bold, some potential heretics)
   - Initial beliefs matching faction archetype with small variance
   - Some agents start with unique experiences (seeds for stories)

4. Create function `create_test_social_interactions() -> HashMap<SocialInteractionType, SocialInteractionDefinition>`:
   - All interaction types defined
   - Reasonable base probabilities
   - Faction-appropriate modifiers
```

### Prompt 7.2: Learning System Integration Tests

```
In sim-core, create `src/tests/learning_integration.rs` with comprehensive tests.

Requirements:
1. Test: Basic belief formation
   - Agent with no prior belief takes action
   - Outcome creates new belief
   - Subsequent same action uses belief for prediction

2. Test: Belief update from experience
   - Agent has prior belief
   - Experience contradicts
   - Belief updates toward actual
   - Confidence changes appropriately

3. Test: Surprise threshold filtering
   - Low-surprise experience doesn't trigger sharing
   - High-surprise experience does trigger sharing
   - Threshold is tunable

4. Test: Book writing permissions
   - Agent with wrong role can't write
   - Agent with right role can write
   - Bold agent with laborer role might write (trait override)

5. Test: Ritual synchronization
   - Create faction with archetype
   - Create agents with drifted beliefs
   - Run ritual
   - Verify agents tagged for sync
   - Process sync over multiple ticks
   - Verify beliefs pulled toward archetype

6. Test: Heresy formation
   - Agent experiences repeated contradiction of archetype
   - Eventually forms heretical belief
   - Heresy is hidden based on traits
   - Bold agent might express heresy

7. Test: Knowledge propagation fidelity
   - Agent A experiences event firsthand (fidelity 1.0)
   - A shares with B (fidelity decays)
   - B shares with C (fidelity decays more)
   - C's belief has lower confidence

8. Test: Schism detection
   - Create faction with orthodox and heretic clusters
   - Run schism detection
   - Verify tension created when threshold crossed

9. Test: Full learning loop
   - Agent takes action with uncertain outcome
   - World resolves with hidden probabilities
   - Agent updates beliefs
   - Agent decides to share
   - Book updated
   - Ritual occurs
   - Faction knowledge updated
   - All events properly emitted
```

---

## Implementation Order

Execute prompts in this order for dependency management:

1. **Phase 1** (sim-events): Foundation types that everything depends on
   - 1.1, 1.2, 1.3

2. **Phase 2** (sim-core/world): Physical world that agents interact with
   - 2.1, 2.2, 2.3

3. **Phase 3** (sim-core/agent): Agent belief systems
   - 3.1, 3.2, 3.3

4. **Phase 4** (sim-core/faction): Faction-level knowledge
   - 4.1, 4.2, 4.3

5. **Phase 5** (sim-core/systems): Core simulation systems
   - 5.1, 5.2, 5.3

6. **Phase 6** (director): Narrative integration
   - 6.1, 6.2

7. **Phase 7**: Testing and validation
   - 7.1, 7.2

---

## Notes for Claude Code

- Each prompt is self-contained but references types from previous prompts
- Use `// TODO: implement` comments for complex logic you need to stub
- Run `cargo check` after each prompt to verify compilation
- The test data (Phase 7) can be implemented earlier if needed for manual testing
- Consider creating `CLAUDE.md` files in each new module directory with module-specific context
