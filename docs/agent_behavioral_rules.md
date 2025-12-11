# Agent Behavioral Rules

This document defines how agents decide what to do. The goal is simple rules that produce complex, human-feeling behavior through interaction.

---

## Design Philosophy

**Agents don't plan.** They react to their current state, relationships, and environment. Long-term "plots" emerge from consistent short-term biases, not from agents modeling the future.

**Personality is just weights.** Two agents in identical situations may act differently because their traits modify action weights. A bold agent and a cautious agent both "consider" confrontation; they just weight it differently.

**Context creates pressure.** The environment (season, faction resources, recent events) shifts what actions are available and attractive. Winter doesn't make agents "decide to betray" — it raises the weight on desperate actions until betrayal becomes thinkable.

---

## Agent State Model

### Needs (Abstracted)

```
food_security: secure | stressed | desperate
social_belonging: integrated | peripheral | isolated
```

These update based on:
- **Food security**: Faction resources ÷ faction size, personal role, season
- **Social belonging**: Ritual attendance, trust from faction members, recent interactions

### Traits (Fixed at Creation)

```yaml
boldness: 0.0 - 1.0        # Willingness to take risks
loyalty_weight: 0.0 - 1.0  # How much faction loyalty factors into decisions
grudge_persistence: 0.0 - 1.0  # How slowly negative memories fade
ambition: 0.0 - 1.0        # Drive toward higher status/power
honesty: 0.0 - 1.0         # Tendency toward truth vs. deception
sociability: 0.0 - 1.0     # Frequency of voluntary interaction
group_preference: 0.0 - 1.0  # Prefers group interaction (1.0) vs 1-on-1 (0.0)
```

### Goals (Dynamic)

Goals emerge from state and events. They're not plans — they're persistent weights on certain outcomes.

```yaml
goals:
  - type: survive_winter
    priority: 0.9
    expires: end_of_winter
    
  - type: revenge
    target: agent_corin_0003
    priority: 0.6
    origin_event: evt_00039102
    
  - type: rise_in_status
    priority: 0.4
    
  - type: protect
    target: agent_daughter_0099
    priority: 0.85
```

Goals modify action weights. An agent with a revenge goal weights actions that harm the target higher.

---

## Action Space

### Movement
| Action | Preconditions | Outcome |
|--------|---------------|---------|
| `travel` | Has destination | Location changes |
| `patrol` | Has assigned area | May detect intruders/events |
| `flee` | Perceives threat | Leaves current location |
| `follow` | Has target | Matches target's location |
| `return_home` | Away from faction HQ | Returns to headquarters |

### Communication
| Action | Preconditions | Outcome |
|--------|---------------|---------|
| `share_memory` | Has memory, has listener | Memory propagates (with fidelity loss) |
| `spread_rumor` | Has memory, low honesty or motivation | Memory propagates (may distort) |
| `lie` | Has motivation, listener present | False memory created in listener |
| `confess` | Has secret, high guilt or trust | Secret revealed |
| `recruit` | Target is vulnerable, different faction or disaffected | Begin defection process |
| `report` | Has information, authority figure accessible | Information reaches leadership |

### Resource
| Action | Preconditions | Outcome |
|--------|---------------|---------|
| `work` | Has work location | Faction resources increase slightly |
| `trade` | Has surplus, trading partner willing | Resources exchange |
| `steal` | Target has resources, low detection risk | Resources transfer, risk of discovery |
| `hoard` | Has access to resources | Personal reserves increase, faction reserves decrease |

### Social
| Action | Preconditions | Outcome |
|--------|---------------|---------|
| `build_trust` | Target present, no hostility | Trust dimensions increase slowly |
| `curry_favor` | Target has higher status | Trust increases, may increase status |
| `gift` | Has resource to give | Trust increases faster than build_trust |
| `ostracize` | Has social standing, target present | Target's belonging decreases |
| `defend_reputation` | Own reputation attacked | May counter rumor effects |

### Archive
| Action | Preconditions | Outcome |
|--------|---------------|---------|
| `write_entry` | At faction HQ, literate or has scribe | Memory becomes archive entry |
| `read_archive` | At faction HQ, literate or reading held | Gain memories from archive |
| `destroy_entry` | At faction HQ, has access | Entry removed from archive |
| `forge_entry` | At faction HQ, has access, low honesty | False entry added to archive |

### Faction
| Action | Preconditions | Outcome |
|--------|---------------|---------|
| `attend_ritual` | Ritual scheduled, at HQ | Belonging increases, memories reinforced |
| `skip_ritual` | Ritual scheduled | Belonging decreases, miss reinforcement |
| `defect` | Has contact in other faction, low loyalty | Faction membership changes |
| `exile_other` | Has authority, target present | Target removed from faction |
| `challenge_leader` | High ambition, support from others | Succession crisis triggered |
| `support_leader` | Leader challenged | Adds weight to leader's position |

### Conflict
| Action | Preconditions | Outcome |
|--------|---------------|---------|
| `argue` | Disagreement exists | Relationship damage, possible resolution |
| `fight` | Hostility high, both present | Physical harm, relationship damage |
| `duel` | Formal grievance, both willing | Decisive outcome, honor satisfied |
| `sabotage` | Has target, has access | Damage to target's resources/reputation |
| `assassinate` | Has target, desperate or high grudge | Target death, high risk of discovery |

---

## Interaction Targeting

When an agent decides to communicate, they must select *who* to interact with. This isn't random — it reflects personality, status awareness, and faction loyalty.

### Target Selection Modes

**Individual targeting**: Agent approaches one specific person
- Deeper trust impact (×1.5 relationship effects)
- Memory shared at full fidelity
- Secrets can be shared (requires 1-on-1)
- Recruitment only works 1-on-1

**Group targeting**: Agent addresses nearby cluster
- Wider spread (all present receive information)
- Shallower impact (×0.5 relationship effects per person)
- Memory fidelity slightly reduced (0.9× — group context is noisier)
- Cannot share secrets
- Cannot recruit
- Builds "faction presence" rather than individual bonds

### Mode Selection

```yaml
select_interaction_mode:
  individual_weight: 1.0 - group_preference
  group_weight: group_preference
  
  modified_by:
    - action_is_secret_sharing: individual only
    - action_is_recruitment: individual only
    - action_is_confession: individual only
    - action_is_public_accusation: group only
    - action_is_rally_support: group only
    - available_group_size < 3: individual only
```

### Individual Target Selection

When targeting an individual, agents score potential targets:

```yaml
target_score:
  base: 1.0
  
  faction_modifier:
    same_faction: ×2.0
    neutral: ×1.0
    enemy_faction: ×0.3
    enemy_faction AND at_neutral_territory: ×0.8
    
  status_modifier:
    target_higher_status: ×(1.0 + 0.5 × status_difference)
    target_same_status: ×1.0
    target_lower_status: ×0.7
    
  relationship_modifier:
    existing_positive_relationship: ×1.3
    existing_negative_relationship: ×0.4
    no_existing_relationship: ×1.0
    
  goal_modifier:
    target_relevant_to_active_goal: ×1.5
    target_is_revenge_target: special handling (see revenge rules)
    
  proximity_modifier:
    same_location: ×1.0
    adjacent_location: ×0.5
    far_location: ×0.1
    
  recency_modifier:
    spoke_this_tick: ×0.1 (avoid repetition)
    spoke_recently: ×0.7
    haven't_spoken_in_long_time: ×1.2
```

### Status Hierarchy

Each faction has implicit status levels:

```yaml
status_levels:
  faction_leader: 5
  reader: 4
  council_member: 4
  specialist (healer, smith, scout_captain): 3
  skilled_worker: 2
  laborer: 1
  newcomer: 1
  exile: 0 (no longer in faction)
```

Status affects:
- Who seeks out whom for conversation
- Whose memories are believed more readily
- Who can write in the archive
- Who attends faction councils
- Weight in succession disputes

### Group Target Selection

When targeting a group, agents prefer:

```yaml
group_preference:
  location_with_most_same_faction: ×1.5
  location_with_high_status_individuals: ×1.3
  location_is_faction_hq: ×1.4
  location_is_public_gathering: ×1.2
  location_has_target_of_goal: ×1.3
```

### Cross-Faction Interaction

Talking to other factions is rare but important:

```yaml
cross_faction_triggers:
  - at_neutral_territory: enables casual cross-faction talk
  - at_trade_location: enables trade negotiation
  - war_time AND enemy_approaches: enables parley
  - has_recruitment_goal: enables recruitment attempts
  - desperate AND other_faction_has_resources: enables begging/dealing
  
cross_faction_effects:
  - own_faction_members_may_witness: can trigger suspicion events
  - trust_builds_slower: ×0.5 trust gain rate
  - information_more_valuable: secrets about own faction worth more
  - relationship_is_risky: discovery damages own faction standing
```

### Social Dynamics That Emerge

**Status-seeking creates information flow upward**: Lower-status agents prefer talking to higher-status agents, who hear a lot but initiate less downward. Leaders become information hubs but may be out of touch with ground-level sentiment.

**Group communicators become broadcast nodes**: High group_preference agents spread information widely but have fewer deep relationships. They're influential but not trusted with secrets.

**Individual communicators become trust anchors**: Low group_preference agents build strong dyadic bonds. They know secrets, can recruit, can betray meaningfully. Their social graph is smaller but denser.

**Cross-faction friendships are fragile**: An agent who talks to outsiders too often becomes peripheral in their own faction. But those relationships may be the only bridges when factions need to negotiate.

---

## Decision Architecture

Each tick, agents:

1. **Perceive** — Update awareness of nearby agents, events, resources
2. **Update state** — Recalculate needs based on conditions
3. **Generate options** — List actions whose preconditions are met
4. **Weight options** — Apply traits, needs, goals, relationships
5. **Select action** — Probabilistic choice weighted by scores
6. **Execute** — Perform action, generate event

### Weighting Formula

```
action_weight = base_weight 
              × trait_modifiers 
              × need_modifiers 
              × goal_modifiers 
              × relationship_modifiers
              × context_modifiers
              + noise
```

The noise term prevents determinism. Even low-weight actions occasionally happen.

---

## Behavioral Rules

Rules are structured as:
```
WHEN [conditions]
THEN [action] has weight [base_weight]
MODIFIED BY [modifiers]
```

### Survival Rules

```yaml
- name: desperate_theft
  when:
    - food_security == desperate
    - nearby_agent has resources
  then: steal
  base_weight: 0.6
  modified_by:
    - boldness: +0.3 at boldness=1.0, +0.0 at boldness=0.0
    - target_is_same_faction: ×0.3
    - target_is_enemy_faction: ×1.5
    - detection_risk: ×(1.0 - risk)

- name: desperate_defection_consideration
  when:
    - food_security == desperate
    - social_belonging == isolated OR social_belonging == peripheral
    - contacted_by_other_faction recently
  then: defect
  base_weight: 0.3
  modified_by:
    - loyalty_weight: ×(1.0 - loyalty_weight)
    - other_faction_food_security: ×1.5 if secure, ×0.5 if also desperate
    - grudge_against_own_faction: +0.2 per significant grudge
```

### Social Rules

```yaml
- name: gossip
  when:
    - nearby_agent exists
    - has_interesting_memory
    - no_hostility with nearby_agent
  then: share_memory
  base_weight: 0.4
  modified_by:
    - sociability: +0.4 at sociability=1.0
    - memory_is_negative_about_third_party: +0.2
    - listener_is_same_faction: +0.2
    - memory_is_secret: ×(1.0 - honesty) if sharing would be betrayal

- name: attend_ritual
  when:
    - ritual_scheduled
    - can_reach_hq
  then: attend_ritual
  base_weight: 0.7
  modified_by:
    - social_belonging: +0.3 if isolated (need connection)
    - loyalty_weight: +0.2 at loyalty_weight=1.0
    - has_secret_that_might_be_read: -0.4
    - food_security == desperate AND work_available: -0.3
```

### Loyalty Rules

```yaml
- name: report_suspicious_activity
  when:
    - witnessed betrayal_indicator
    - authority_figure accessible
  then: report
  base_weight: 0.5
  modified_by:
    - loyalty_weight: +0.4 at loyalty_weight=1.0
    - trust_in_suspect: ×(1.0 - reliability_trust)
    - trust_in_authority: +0.2 if high
    - suspect_is_friend: -0.3

- name: refuse_recruitment
  when:
    - being_recruited by other_faction_agent
  then: reject_and_report
  base_weight: 0.4
  modified_by:
    - loyalty_weight: +0.5 at loyalty_weight=1.0
    - social_belonging == integrated: +0.3
    - food_security == secure: +0.2
    - recruiter_trust_reliability: -0.1 per 0.1 trust
```

### Revenge Rules

```yaml
- name: spread_negative_memory
  when:
    - has_goal type=revenge
    - has_memory damaging to target
    - nearby_agent could be influenced
  then: share_memory (selecting damaging memory)
  base_weight: 0.5
  modified_by:
    - grudge_persistence: +0.3 at grudge_persistence=1.0
    - memory_age: ×0.9 per season elapsed
    - listener_is_close_to_target: +0.3

- name: sabotage_target
  when:
    - has_goal type=revenge
    - target_has_vulnerable_resource_or_reputation
    - detection_risk acceptable
  then: sabotage
  base_weight: 0.2
  modified_by:
    - boldness: +0.3 at boldness=1.0
    - grudge_persistence: +0.2
    - goal_priority: ×priority
    - detection_risk: ×(1.0 - risk)

- name: assassination_consideration
  when:
    - has_goal type=revenge with priority > 0.8
    - food_security == desperate OR social_belonging == isolated
    - opportunity exists (alone with target, target vulnerable)
  then: assassinate
  base_weight: 0.05
  modified_by:
    - boldness: +0.1 at boldness=1.0
    - grudge_persistence: +0.1
    - target_has_harmed_family: +0.2
    - detection_risk: ×(1.0 - risk)²
```

### Archive Rules

```yaml
- name: record_significant_event
  when:
    - witnessed significant_event
    - at_faction_hq
    - has_archive_access
  then: write_entry
  base_weight: 0.3
  modified_by:
    - event_reflects_well_on_self: +0.3
    - event_reflects_poorly_on_rival: +0.2
    - event_reflects_poorly_on_self: -0.4
    - loyalty_weight: +0.2 if event is faction-relevant

- name: destroy_embarrassing_record
  when:
    - at_faction_hq
    - has_archive_access
    - archive contains entry damaging to self
  then: destroy_entry
  base_weight: 0.2
  modified_by:
    - honesty: ×(1.0 - honesty)
    - entry_is_widely_known: -0.3
    - entry_is_forgotten: +0.3
    - detection_risk: ×(1.0 - risk)

- name: reader_selects_entries
  when:
    - is_reader
    - ritual_scheduled
  then: select_entries_to_read
  base_weight: 1.0 (mandatory)
  selection_modified_by:
    - entry_reinforces_faction_loyalty: +0.3
    - entry_embarrasses_current_leader: -0.4 if loyal, +0.2 if disloyal
    - entry_embarrasses_rival: +0.2
    - entry_praises_self: +0.1
    - entry_recently_read: -0.2
```

### Ambition Rules

```yaml
- name: seek_promotion
  when:
    - ambition > 0.5
    - higher_role available or achievable
  then: curry_favor with authority
  base_weight: 0.3
  modified_by:
    - ambition: +0.4 at ambition=1.0
    - social_belonging == integrated: +0.2
    - current_role_is_low_status: +0.2

- name: challenge_leadership
  when:
    - ambition > 0.7
    - leader_is_weakened (lost trust, failed recently)
    - has_support from faction_members
  then: challenge_leader
  base_weight: 0.1
  modified_by:
    - ambition: +0.2 at ambition=1.0
    - leader_trust_from_faction: ×(1.0 - avg_trust)
    - own_trust_from_faction: +0.3 if high
    - boldness: +0.2 at boldness=1.0
```

---

## Context Modifiers

Global conditions that shift weights across many actions:

### Seasonal

```yaml
winter:
  desperate_actions: ×1.5
  travel: ×0.7
  work_output: ×0.6
  ritual_attendance: +0.1 (community need)
  
summer:
  travel: ×1.2
  trade: ×1.3
  surplus_hoarding: +0.2
```

### Faction State

```yaml
faction_at_war:
  defection: ×0.5 (rallying effect)
  report_suspicious: ×1.5
  internal_conflict: ×0.7
  
faction_resources_critical:
  desperate_actions: ×1.5
  hoarding: ×1.5
  loyalty_to_leader: ×0.8 (blame)
  
recent_betrayal_discovered:
  suspicion_of_others: ×1.5
  report_suspicious: ×1.3
  trust_building: ×0.7
```

### Location

```yaml
at_faction_hq:
  archive_actions: enabled
  ritual_actions: enabled
  faction_social_actions: ×1.3
  
neutral_territory:
  cross_faction_interaction: ×1.5
  secret_meetings: enabled
  detection_risk: ×0.5
  
enemy_territory:
  all_actions: ×0.5 (caution)
  detection_risk: ×2.0
  flee_threshold: lowered
```

---

## Memory and Trust Dynamics

### Memory Formation

```yaml
firsthand_memory:
  fidelity: 1.0
  emotional_weight: based on event impact
  decay_rate: slow (0.95 per season)
  
secondhand_memory:
  fidelity: source_fidelity × 0.7
  emotional_weight: source_weight × 0.5
  decay_rate: faster (0.85 per season)
  attribution: "agent_X claims..."
  
archive_memory:
  fidelity: 0.9 (written record)
  emotional_weight: 0.3 (unless reinforced by ritual)
  decay_rate: very slow if repeatedly read
  attribution: "the archive records..."
```

### Trust Updates

```yaml
positive_interaction:
  reliability: +0.05 if promise kept
  alignment: +0.03 if helped toward goal
  capability: +0.02 if demonstrated skill
  
negative_interaction:
  reliability: -0.15 if promise broken (asymmetric!)
  alignment: -0.10 if worked against goal
  capability: -0.05 if failed at task
  
betrayal:
  reliability: -0.5 (catastrophic)
  alignment: -0.4
  creates: grudge memory with high persistence
  
secondhand_negative:
  apply: 30% of direct effect
  modified_by: trust in source
```

---

## Emergence Patterns to Watch For

These aren't programmed — they should emerge from rule interactions:

- **Faction drift**: Agents who miss rituals slowly diverge from faction consensus
- **Reputation cascades**: One agent's rumor spreads, trust erodes, creates confirming behavior
- **Desperate defection waves**: Winter scarcity triggers one defection, which weakens faction, triggering more
- **Reader kingmaking**: Reader who consistently emphasizes certain narratives shifts faction priorities
- **Grudge inheritance**: Agent dies, but their written grievances persist, inherited by readers
- **Reconciliation difficulty**: Even when both parties want peace, secondhand memories keep circulating

---

## Tuning Parameters

Key values to adjust during testing:

| Parameter | Starting Value | What to Watch |
|-----------|----------------|---------------|
| `gossip_base_weight` | 0.4 | Too high = memory spreads too fast |
| `trust_decay_rate` | 0.02/season | Too slow = grudges never heal |
| `betrayal_trust_penalty` | -0.5 | Too low = betrayal has no weight |
| `secondhand_fidelity` | 0.7 | Too high = rumors = facts |
| `desperate_threshold` | resource < 0.2 | Too low = never desperate |
| `noise_factor` | 0.1 | Too low = deterministic |
| `ritual_reinforcement` | +0.1 | Too high = faction groupthink |

---

## Implementation Notes

### Action Selection Pseudocode

```python
def select_action(agent, world):
    options = []
    
    for action in ALL_ACTIONS:
        if action.preconditions_met(agent, world):
            weight = action.base_weight
            weight *= agent.trait_modifier(action)
            weight *= agent.need_modifier(action)
            weight *= agent.goal_modifier(action)
            weight *= agent.relationship_modifier(action, world)
            weight *= world.context_modifier(action)
            weight += random.gauss(0, NOISE_FACTOR)
            
            options.append((action, max(0, weight)))
    
    # Probabilistic selection weighted by scores
    return weighted_random_choice(options)
```

### Event Generation

Every action produces an event. Even "nothing interesting happened" can be a non-event tick for the agent, but significant actions always log:

```python
def execute_action(agent, action, world):
    outcome = action.execute(agent, world)
    
    event = Event(
        type=action.event_type,
        actors=action.get_actors(agent, world),
        context=action.get_context(agent, world),
        outcome=outcome,
        drama_score=calculate_drama(action, outcome),
        drama_tags=action.get_tags(outcome)
    )
    
    world.event_log.append(event)
    world.update_tensions(event)
    
    return outcome
```

---

*Rules version 0.1 — expect significant iteration during testing*
