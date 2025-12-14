# Simulation Output Schemas

This document defines the data structures the simulation engine produces. These outputs serve three consumers:
1. **Director AI** — to select what's worth visualizing
2. **Visualization layer** — to render the world
3. **Human/LLM analysis** — to iterate on behavioral rules

All formats are JSON for readability and tooling compatibility.

---

## Event Log

The primary historical record. Append-only, immutable once written. Each event must be interpretable without access to external state.

### Event Schema

```json
{
  "event_id": "evt_00042371",
  "timestamp": {
    "tick": 84729,
    "date": "year_3.winter.day_12"
  },
  "event_type": "betrayal",
  "subtype": "secret_shared_with_enemy",
  
  "actors": {
    "primary": {
      "agent_id": "agent_mira_0042",
      "name": "Mira of Thornwood",
      "faction": "thornwood",
      "role": "scout",
      "location": "eastern_bridge"
    },
    "secondary": {
      "agent_id": "agent_voss_0017",
      "name": "Voss the Quiet",
      "faction": "ironmere",
      "role": "spymaster",
      "location": "eastern_bridge"
    },
    "affected": [
      {
        "agent_id": "agent_corin_0003",
        "name": "Corin Thornwood",
        "faction": "thornwood",
        "role": "faction_leader",
        "relationship_to_primary": "trusts_highly"
      }
    ]
  },

  "context": {
    "trigger": "mira_reliability_trust_in_corin_below_threshold",
    "preconditions": [
      "mira witnessed corin_broke_promise_to_mira at tick 81204",
      "mira has been approached by voss 3 times",
      "thornwood grain reserves critical"
    ],
    "location_description": "neutral territory, no witnesses"
  },

  "outcome": {
    "information_transferred": "thornwood_winter_grain_cache_location",
    "relationship_changes": [
      {
        "from": "agent_mira_0042",
        "to": "agent_voss_0017",
        "dimension": "alignment",
        "old_value": 0.3,
        "new_value": 0.5
      }
    ],
    "state_changes": [
      "ironmere now knows thornwood_winter_grain_cache_location"
    ]
  },

  "drama_tags": ["betrayal", "faction_critical", "secret_meeting", "winter_crisis"],
  "drama_score": 0.87,
  "connected_events": ["evt_00039102", "evt_00041888"]
}
```

### Event Types

| Type | Subtypes | Description |
|------|----------|-------------|
| `movement` | `travel`, `flee`, `pursue`, `patrol` | Agent changes location |
| `communication` | `share_memory`, `spread_rumor`, `lie`, `confess` | Information transfer between agents |
| `betrayal` | `secret_shared_with_enemy`, `sabotage`, `defection`, `false_testimony` | Trust violation |
| `loyalty` | `defend_ally`, `sacrifice_for_faction`, `refuse_bribe` | Trust-affirming action |
| `conflict` | `argument`, `fight`, `duel`, `raid` | Direct antagonism |
| `cooperation` | `trade`, `alliance_formed`, `gift`, `favor` | Positive interaction |
| `faction` | `join`, `leave`, `exile`, `promotion`, `demotion` | Membership changes |
| `archive` | `write_entry`, `read_entry`, `destroy_entry`, `forge_entry` | Institutional memory actions |
| `ritual` | `reading_held`, `reading_disrupted`, `reading_attended`, `reading_missed` | Faction rituals |
| `resource` | `acquire`, `lose`, `trade`, `steal`, `hoard` | Economic actions |
| `death` | `natural`, `killed`, `executed`, `sacrifice` | Agent death |
| `birth` | `born`, `arrived`, `created` | New agent enters simulation |

### Event Examples

#### Simple Movement
```json
{
  "event_id": "evt_00042370",
  "timestamp": {"tick": 84728, "date": "year_3.winter.day_12"},
  "event_type": "movement",
  "subtype": "travel",
  "actors": {
    "primary": {
      "agent_id": "agent_mira_0042",
      "name": "Mira of Thornwood",
      "faction": "thornwood",
      "role": "scout",
      "location": "thornwood_village"
    }
  },
  "context": {
    "trigger": "scheduled_patrol",
    "preconditions": []
  },
  "outcome": {
    "new_location": "eastern_bridge",
    "travel_duration_ticks": 45
  },
  "drama_tags": [],
  "drama_score": 0.05,
  "connected_events": []
}
```

#### Ritual Reading
```json
{
  "event_id": "evt_00042400",
  "timestamp": {"tick": 84900, "date": "year_3.winter.day_14"},
  "event_type": "ritual",
  "subtype": "reading_held",
  "actors": {
    "primary": {
      "agent_id": "agent_elder_wen_0008",
      "name": "Elder Wen",
      "faction": "thornwood",
      "role": "reader",
      "location": "thornwood_hall"
    },
    "affected": [
      {"agent_id": "agent_corin_0003", "attended": true},
      {"agent_id": "agent_mira_0042", "attended": false, "reason": "absent_from_village"},
      {"agent_id": "agent_tom_0044", "attended": true}
    ]
  },
  "context": {
    "trigger": "scheduled_weekly_ritual",
    "preconditions": ["no_active_raid", "reader_alive"]
  },
  "outcome": {
    "entries_read": [
      {"entry_id": "archive_tw_0012", "subject": "ironmere_broke_salt_treaty_year_1"},
      {"entry_id": "archive_tw_0089", "subject": "mira_heroic_winter_scout_year_2"}
    ],
    "entries_skipped": [
      {"entry_id": "archive_tw_0067", "subject": "corin_failed_harvest_negotiation"}
    ],
    "memory_reinforcement": [
      {"memory": "ironmere_untrustworthy", "agents_reinforced": 12, "strength_delta": 0.1},
      {"memory": "mira_is_hero", "agents_reinforced": 12, "strength_delta": 0.08}
    ]
  },
  "drama_tags": ["selective_history", "absent_agent", "leader_embarrassment_hidden"],
  "drama_score": 0.45,
  "connected_events": ["evt_00042371"]
}
```

#### Memory Propagation
```json
{
  "event_id": "evt_00042450",
  "timestamp": {"tick": 85000, "date": "year_3.winter.day_15"},
  "event_type": "communication",
  "subtype": "share_memory",
  "actors": {
    "primary": {
      "agent_id": "agent_tom_0044",
      "name": "Tom the Farmhand",
      "faction": "thornwood",
      "role": "laborer",
      "location": "thornwood_fields"
    },
    "secondary": {
      "agent_id": "agent_bess_0051",
      "name": "Bess of Thornwood",
      "faction": "thornwood",
      "role": "laborer",
      "location": "thornwood_fields"
    }
  },
  "context": {
    "trigger": "casual_conversation",
    "preconditions": ["same_location", "no_hostility"]
  },
  "outcome": {
    "memory_shared": {
      "original_event": "evt_00039102",
      "content": "corin_broke_promise_to_mira",
      "source_chain": ["mira told tom", "tom tells bess"],
      "fidelity": 0.7
    },
    "recipient_state_change": {
      "new_memory_added": true,
      "trust_impact": {
        "toward": "agent_corin_0003",
        "dimension": "reliability",
        "delta": -0.05,
        "reason": "secondhand_negative_testimony"
      }
    }
  },
  "drama_tags": ["rumor_spreading", "leader_reputation_erosion"],
  "drama_score": 0.35,
  "connected_events": ["evt_00039102", "evt_00042371"]
}
```

---

## World Snapshot

Periodic capture of full simulation state. Taken at regular intervals and on significant events. Used for visualization, analysis, and save/restore.

### Snapshot Schema

```json
{
  "snapshot_id": "snap_003847",
  "timestamp": {
    "tick": 85000,
    "date": "year_3.winter.day_15"
  },
  "triggered_by": "periodic_1000_ticks",

  "world": {
    "season": "winter",
    "global_resources": {
      "total_grain": 4500,
      "total_iron": 1200,
      "total_salt": 300
    },
    "active_threats": ["harsh_winter", "bandit_activity_north"]
  },

  "factions": [
    {
      "faction_id": "thornwood",
      "territory": ["thornwood_village", "thornwood_fields", "eastern_forest"],
      "headquarters": "thornwood_hall",
      "resources": {
        "grain": 800,
        "iron": 150,
        "salt": 40
      },
      "member_count": 47,
      "leader": "agent_corin_0003",
      "reader": "agent_elder_wen_0008",
      "archive_entry_count": 94,
      "cohesion_score": 0.72,
      "external_reputation": {
        "ironmere": -0.4,
        "saltcliff": 0.2,
        "northern_hold": 0.0
      }
    }
  ],

  "agents": [
    {
      "agent_id": "agent_mira_0042",
      "name": "Mira of Thornwood",
      "alive": true,
      "faction": "thornwood",
      "role": "scout",
      "location": "eastern_bridge",
      "traits": {
        "boldness": 0.7,
        "loyalty_weight": 0.4,
        "memory_strength": 0.8,
        "grudge_persistence": 0.9,
        "group_preference": 0.3
      },
      "status": {
        "level": 3,
        "role_title": "scout_captain",
        "influence_score": 0.65,
        "social_reach": 24,
        "trusted_by_count": 18,
        "trusts_count": 12
      },
      "physical_state": {
        "health": 0.85,
        "hunger": 0.3,
        "exhaustion": 0.2
      },
      "needs": {
        "food_security": "stressed",
        "social_belonging": "integrated"
      },
      "goals": [
        {"goal": "survive_winter", "priority": 0.9},
        {"goal": "revenge_on_corin", "priority": 0.6}
      ],
      "inventory": ["scout_cloak", "dagger", "stolen_ironmere_map"],
      "visual_markers": ["scar_left_cheek", "ironmere_cloak_stolen"]
    }
  ],

  "relationships": {
    "agent_mira_0042": {
      "agent_corin_0003": {
        "reliability": 0.2,
        "alignment": 0.3,
        "capability": 0.7,
        "last_interaction_tick": 81204,
        "memory_count": 12,
        "significant_memories": ["corin_broke_promise", "corin_promoted_rival"]
      },
      "agent_voss_0017": {
        "reliability": 0.4,
        "alignment": 0.5,
        "capability": 0.8,
        "last_interaction_tick": 84729,
        "memory_count": 4,
        "significant_memories": ["voss_offered_deal", "voss_kept_secret"]
      }
    }
  },

  "locations": [
    {
      "location_id": "eastern_bridge",
      "type": "neutral_territory",
      "controlling_faction": null,
      "agents_present": ["agent_mira_0042", "agent_voss_0017"],
      "resources": {},
      "properties": ["hidden_meeting_spot", "trade_route"]
    }
  ],

  "computed_metrics": {
    "faction_power_balance": {
      "thornwood": 0.28,
      "ironmere": 0.35,
      "saltcliff": 0.22,
      "northern_hold": 0.15
    },
    "war_probability_30_days": {
      "thornwood_vs_ironmere": 0.65,
      "thornwood_vs_saltcliff": 0.1
    },
    "agents_at_defection_risk": ["agent_mira_0042", "agent_karl_0088"],
    "factions_at_collapse_risk": [],
    "social_network": {
      "hubs": [
        {
          "agent_id": "agent_corin_0003",
          "faction": "thornwood",
          "influence_score": 0.89,
          "role": "faction_leader",
          "connections": 42
        },
        {
          "agent_id": "agent_elder_wen_0008",
          "faction": "thornwood",
          "influence_score": 0.78,
          "role": "reader",
          "connections": 38
        }
      ],
      "bridges": [
        {
          "agent_id": "agent_mira_0042",
          "connects": ["thornwood", "ironmere"],
          "bridge_strength": 0.4,
          "known_to_faction": false
        }
      ],
      "isolates": [
        {
          "agent_id": "agent_old_tom_0077",
          "faction": "thornwood",
          "connections": 2,
          "belonging": "isolated",
          "risk": "death_unnoticed"
        }
      ],
      "cliques": [
        {
          "members": ["agent_tom_0044", "agent_bess_0051", "agent_will_0052"],
          "faction": "thornwood",
          "shared_memory": "corin_broke_promise",
          "sentiment_toward_leader": -0.3
        }
      ]
    }
  }
}
```

---

## Tension Stream

Real-time feed of developing situations worth attention. Higher-level than raw events. The Director AI's primary input for deciding what to show.

### Tension Schema

```json
{
  "tension_id": "tens_00284",
  "detected_at_tick": 84000,
  "last_updated_tick": 85000,
  "status": "escalating",

  "tension_type": "brewing_betrayal",
  "severity": 0.82,
  "confidence": 0.75,

  "summary": "Mira of Thornwood losing faith in faction leader, has established contact with Ironmere spymaster",

  "key_agents": [
    {
      "agent_id": "agent_mira_0042",
      "role_in_tension": "potential_betrayer",
      "trajectory": "toward_defection"
    },
    {
      "agent_id": "agent_corin_0003",
      "role_in_tension": "unaware_target",
      "trajectory": "stable"
    },
    {
      "agent_id": "agent_voss_0017",
      "role_in_tension": "external_catalyst",
      "trajectory": "actively_recruiting"
    }
  ],

  "key_locations": ["eastern_bridge", "thornwood_hall", "ironmere_camp"],

  "trigger_events": ["evt_00039102", "evt_00041888", "evt_00042371"],

  "predicted_outcomes": [
    {
      "outcome": "mira_defects_to_ironmere",
      "probability": 0.45,
      "impact": "high",
      "estimated_ticks_until": 2000
    },
    {
      "outcome": "mira_discovered_and_exiled",
      "probability": 0.25,
      "impact": "high",
      "estimated_ticks_until": 1500
    },
    {
      "outcome": "mira_reconciles_with_corin",
      "probability": 0.15,
      "impact": "medium",
      "estimated_ticks_until": 3000
    },
    {
      "outcome": "mira_assassinates_corin",
      "probability": 0.10,
      "impact": "critical",
      "estimated_ticks_until": 4000
    }
  ],

  "narrative_hooks": [
    "Mira was once Corin's most trusted scout",
    "The stolen Ironmere map in Mira's possession is not yet discovered",
    "Elder Wen continues to read praise of Mira at rituals, unaware of her meetings"
  ],

  "recommended_camera_focus": {
    "primary": "agent_mira_0042",
    "secondary": ["agent_voss_0017", "agent_corin_0003"],
    "locations_of_interest": ["eastern_bridge"]
  },

  "connected_tensions": ["tens_00279", "tens_00281"]
}
```

### Tension Types

| Type | Description | Typical Agents |
|------|-------------|----------------|
| `brewing_betrayal` | Trust eroding toward defection/sabotage | betrayer, target, catalyst |
| `succession_crisis` | Leadership contested or uncertain | candidates, kingmakers, current_leader |
| `resource_conflict` | Scarcity driving competition | competitors, holders, desperate |
| `forbidden_alliance` | Cross-faction relationship forming | allies, faction_loyalists, discoverers |
| `revenge_arc` | Agent pursuing payback | avenger, target, witnesses |
| `rising_power` | Agent gaining influence rapidly | rising_agent, threatened_agents |
| `faction_fracture` | Internal faction divisions deepening | faction_camps, mediators |
| `external_threat` | Outside pressure forcing response | threatened, responders, opportunists |
| `secret_exposed` | Hidden information about to surface | secret_holder, affected_parties, discoverer |
| `ritual_disruption` | Upcoming ritual at risk | reader, disruptors, attendees |

### Tension Example: Resource Conflict

```json
{
  "tension_id": "tens_00279",
  "detected_at_tick": 82000,
  "last_updated_tick": 85000,
  "status": "critical",
  "tension_type": "resource_conflict",
  "severity": 0.91,
  "confidence": 0.90,
  "summary": "Winter grain shortage forcing Thornwood and Ironmere toward confrontation over eastern granary",
  "key_agents": [
    {"agent_id": "agent_corin_0003", "role_in_tension": "faction_leader_desperate"},
    {"agent_id": "agent_iron_chief_0001", "role_in_tension": "faction_leader_aggressive"}
  ],
  "key_locations": ["eastern_granary", "thornwood_village", "ironmere_camp"],
  "predicted_outcomes": [
    {"outcome": "armed_raid_on_granary", "probability": 0.55, "impact": "critical"},
    {"outcome": "desperate_negotiation", "probability": 0.30, "impact": "medium"},
    {"outcome": "third_faction_intervenes", "probability": 0.15, "impact": "high"}
  ],
  "connected_tensions": ["tens_00284"]
}
```

---

## Output Files

The simulation produces these files:

| File | Format | Write Pattern | Size Expectation |
|------|--------|---------------|------------------|
| `events.jsonl` | JSON Lines | Append-only | Grows continuously |
| `snapshots/snap_{id}.json` | JSON | Periodic write | ~1-5MB per snapshot |
| `tensions.json` | JSON | Overwrite on change | ~100KB |
| `current_state.json` | JSON | Overwrite each tick | ~2-10MB |

### JSON Lines for Events

Events are stored one per line for efficient streaming and append:

```
{"event_id":"evt_00000001","timestamp":{"tick":1,"date":"year_1.spring.day_1"},...}
{"event_id":"evt_00000002","timestamp":{"tick":2,"date":"year_1.spring.day_1"},...}
{"event_id":"evt_00000003","timestamp":{"tick":5,"date":"year_1.spring.day_1"},...}
```

This allows:
- Tailing the file to watch live events
- Efficient grep/search for specific event types
- Line-by-line LLM processing without loading entire history

---

## Usage Examples

### Human Analysis: Find All Betrayals
```bash
grep '"event_type": "betrayal"' events.jsonl | jq '.actors.primary.name, .outcome'
```

### LLM Prompt: Summarize Recent History
```
Here are the last 50 events from the simulation:
{events}

And here are the current active tensions:
{tensions}

Write a dramatic summary of what's happening in this world right now,
focusing on the most compelling storylines.
```

### Director AI: Select Focus
```
Given these active tensions ranked by severity:
{tensions}

And the current camera position at {location},

Which tension should we visualize next? Consider:
- Dramatic payoff (is something about to happen?)
- Visual interest (are agents in interesting locations?)
- Narrative continuity (does this connect to what we just showed?)
```

---

## Design Notes

### Why Self-Contained Events?
Each event includes actor state at the moment of the event (not just IDs) because:
1. Historical queries don't require loading snapshots
2. LLMs can understand events without additional context
3. Humans can read the log directly
4. The Director doesn't need random access to past state

### Why Computed Tension Scores?
Raw events are too granular for drama detection. Pre-computing tension indicators means:
1. Director AI operates on higher-level abstractions
2. Drama detection logic is centralized and tunable
3. Multiple consumers (Director, dashboards, alerts) share the same analysis

### Why Both Snapshots and Current State?
- `current_state.json`: Always reflects right now, for visualization
- `snapshots/`: Historical checkpoints, for replay and branching
- Different access patterns, different files

---

*Schema version 0.1 — subject to iteration as behavioral rules develop*
