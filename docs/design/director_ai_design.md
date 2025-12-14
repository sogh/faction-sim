# Director AI: Design & Implementation Plan

## Purpose

The Director AI sits between the simulation and visualization layers. It watches the stream of raw events and active tensions, then decides **what's worth showing** and **how to show it**. Think of it as an invisible documentary filmmaker—choosing when to cut, where to point the camera, and what story threads to follow.

---

## Design Principles

### Start Dumb, Get Smart
The initial implementation uses deterministic rules and templates. LLM integration comes later, once we understand what decisions actually need intelligence vs. what can be handled by heuristics.

### Separation of Concerns
- **What happened**: Simulation's job (events, state changes)
- **What matters**: Director's job (drama detection, narrative threading)
- **What it looks like**: Visualization's job (rendering, animation)

### Graceful Degradation
The Director should produce useful output even when:
- Running without LLM access
- Processing faster than real-time (batch mode)
- Tensions are sparse or overwhelming

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      DIRECTOR AI                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │   Ingestion  │───▶│   Analysis   │───▶│   Output     │  │
│  │              │    │              │    │   Generation │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│         │                   │                   │           │
│         ▼                   ▼                   ▼           │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │ Event Buffer │    │  Tension     │    │  Camera      │  │
│  │              │    │  Tracker     │    │  Script      │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│                             │                   │           │
│                             ▼                   ▼           │
│                      ┌──────────────┐    ┌──────────────┐  │
│                      │  Narrative   │    │  Commentary  │  │
│                      │  Threads     │    │  Queue       │  │
│                      └──────────────┘    └──────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘

Inputs:                              Outputs:
- events.jsonl (streaming)           - camera_script.json
- tensions.json                      - commentary.json  
- current_state.json                 - highlights.json
```

---

## Core Data Structures

### Camera Instruction

What the Director tells the visualization to do:

```rust
pub struct CameraInstruction {
    pub instruction_id: String,
    pub timestamp: SimTimestamp,
    pub valid_until: Option<SimTimestamp>,
    
    pub camera_mode: CameraMode,
    pub focus: CameraFocus,
    pub pacing: PacingHint,
    
    pub reason: String,  // For debugging/logging
    pub tension_id: Option<String>,  // What tension spawned this
}

pub enum CameraMode {
    FollowAgent { agent_id: String, zoom: ZoomLevel },
    FrameLocation { location_id: String, zoom: ZoomLevel },
    FrameMultiple { agent_ids: Vec<String>, auto_zoom: bool },
    Cinematic { path: Vec<CameraWaypoint>, duration_ticks: u32 },
    Overview { region: Option<String> },
}

pub enum CameraFocus {
    Primary(String),           // Single agent/location
    Conversation(String, String),  // Two agents
    Group(Vec<String>),        // Multiple agents
    Location(String),          // Place itself
}

pub enum PacingHint {
    Slow,       // Linger, something subtle is happening
    Normal,     // Standard pacing
    Urgent,     // Quick cuts, tension building
    Climactic,  // Key moment, hold steady
}

pub enum ZoomLevel {
    Extreme,    // Face details
    Close,      // Single agent + immediate surroundings
    Medium,     // Small group, single building
    Wide,       // Village/camp scale
    Regional,   // Multiple locations visible
}
```

### Commentary Item

Optional text overlays or narrative beats:

```rust
pub struct CommentaryItem {
    pub item_id: String,
    pub timestamp: SimTimestamp,
    pub display_duration_ticks: u32,
    
    pub commentary_type: CommentaryType,
    pub content: String,
    pub priority: f32,
    
    pub related_agents: Vec<String>,
    pub related_tension: Option<String>,
}

pub enum CommentaryType {
    EventCaption,       // "Mira arrives at the eastern bridge"
    DramaticIrony,      // "Corin doesn't know his scout met with the enemy last night"
    ContextReminder,    // "Three months ago, Corin broke his promise to Mira"
    TensionTeaser,      // "Winter stores are running low across the valley"
    NarratorVoice,      // LLM-generated dramatic prose (Phase 2+)
}
```

### Narrative Thread

Tracks ongoing storylines for continuity:

```rust
pub struct NarrativeThread {
    pub thread_id: String,
    pub created_at_tick: u64,
    pub last_updated_tick: u64,
    pub status: ThreadStatus,
    
    pub tension_ids: Vec<String>,
    pub key_agents: Vec<String>,
    pub key_events: Vec<String>,
    
    pub thread_type: String,  // "betrayal_arc", "succession", etc.
    pub summary: String,
    pub hook: String,  // One-line teaser
    
    pub screen_time_ticks: u64,  // How long we've focused on this
    pub last_shown_tick: Option<u64>,
}

pub enum ThreadStatus {
    Developing,   // Building toward something
    Climaxing,    // Key moment happening now
    Resolving,    // Aftermath playing out
    Dormant,      // Nothing happening, might reactivate
    Concluded,    // Story finished
}
```

---

## Phase 1: Template-Based Director

### Goal
Get something working that makes reasonable camera decisions without any ML/LLM dependency. This establishes the interface contracts and lets us iterate on what "good directing" means.

### Implementation

#### 1.1 Event Prioritization

Simple scoring based on event properties:

```rust
pub struct EventScorer {
    weights: EventWeights,
}

pub struct EventWeights {
    pub base_scores: HashMap<EventType, f32>,
    pub subtype_modifiers: HashMap<String, f32>,
    pub drama_tag_scores: HashMap<String, f32>,
    pub recency_decay: f32,
}

impl EventScorer {
    pub fn score(&self, event: &Event, context: &DirectorContext) -> f32 {
        let mut score = self.weights.base_scores
            .get(&event.event_type)
            .copied()
            .unwrap_or(0.1);
        
        // Subtype modifier
        if let Some(modifier) = self.weights.subtype_modifiers.get(&event.subtype) {
            score *= modifier;
        }
        
        // Drama tags additive
        for tag in &event.drama_tags {
            if let Some(tag_score) = self.weights.drama_tag_scores.get(tag) {
                score += tag_score;
            }
        }
        
        // Boost if involves agents we're already tracking
        if context.tracked_agents.contains(&event.actors.primary.agent_id) {
            score *= 1.5;
        }
        
        // Boost if connected to active tension
        if context.active_tension_events.contains(&event.event_id) {
            score *= 2.0;
        }
        
        score
    }
}
```

Default weights (tunable via config):

```toml
[event_weights.base_scores]
betrayal = 0.9
death = 0.85
conflict = 0.7
faction = 0.6
ritual = 0.5
cooperation = 0.4
communication = 0.3
resource = 0.25
movement = 0.1

[event_weights.drama_tag_scores]
faction_critical = 0.3
secret_meeting = 0.25
leader_involved = 0.2
cross_faction = 0.15
winter_crisis = 0.1
```

#### 1.2 Tension-Based Focus Selection

```rust
pub struct FocusSelector {
    min_tension_severity: f32,
    max_concurrent_threads: usize,
    thread_fatigue_threshold_ticks: u64,
}

impl FocusSelector {
    pub fn select_focus(
        &self,
        tensions: &[Tension],
        active_threads: &[NarrativeThread],
        current_focus: Option<&CameraFocus>,
    ) -> CameraInstruction {
        // Filter to actionable tensions
        let viable: Vec<_> = tensions.iter()
            .filter(|t| t.severity >= self.min_tension_severity)
            .filter(|t| t.status != TensionStatus::Resolved)
            .collect();
        
        if viable.is_empty() {
            return self.default_wandering_camera();
        }
        
        // Prefer continuing current thread if still active
        if let Some(current) = current_focus {
            if let Some(tension) = self.find_tension_for_focus(current, &viable) {
                if !self.is_fatigued(tension, active_threads) {
                    return self.continue_focus(tension);
                }
            }
        }
        
        // Otherwise pick highest severity we haven't over-covered
        let selected = viable.iter()
            .filter(|t| !self.is_fatigued(t, active_threads))
            .max_by(|a, b| a.severity.partial_cmp(&b.severity).unwrap());
        
        match selected {
            Some(tension) => self.focus_on_tension(tension),
            None => self.default_wandering_camera(),
        }
    }
    
    fn is_fatigued(&self, tension: &Tension, threads: &[NarrativeThread]) -> bool {
        threads.iter()
            .find(|t| t.tension_ids.contains(&tension.tension_id))
            .map(|t| t.screen_time_ticks > self.thread_fatigue_threshold_ticks)
            .unwrap_or(false)
    }
}
```

#### 1.3 Template-Based Commentary

Commentary generated from templates with slot-filling:

```rust
pub struct CommentaryGenerator {
    templates: CommentaryTemplates,
}

pub struct CommentaryTemplates {
    pub event_captions: HashMap<(EventType, String), Vec<String>>,
    pub dramatic_irony: Vec<IronyTemplate>,
    pub context_reminders: Vec<ReminderTemplate>,
}

pub struct IronyTemplate {
    pub pattern: String,  // "agent_unaware_of_betrayal"
    pub templates: Vec<String>,
    pub required_context: Vec<String>,
}

impl CommentaryGenerator {
    pub fn caption_event(&self, event: &Event) -> Option<CommentaryItem> {
        let key = (event.event_type.clone(), event.subtype.clone());
        let templates = self.templates.event_captions.get(&key)?;
        
        let template = templates.choose(&mut rand::thread_rng())?;
        let content = self.fill_template(template, event);
        
        Some(CommentaryItem {
            item_id: format!("cap_{}", event.event_id),
            timestamp: event.timestamp.clone(),
            display_duration_ticks: 100,
            commentary_type: CommentaryType::EventCaption,
            content,
            priority: event.drama_score,
            related_agents: event.all_agent_ids(),
            related_tension: None,
        })
    }
    
    fn fill_template(&self, template: &str, event: &Event) -> String {
        template
            .replace("{primary_name}", &event.actors.primary.name)
            .replace("{primary_faction}", &event.actors.primary.faction)
            .replace("{location}", &event.actors.primary.location)
            .replace("{secondary_name}", 
                event.actors.secondary.as_ref()
                    .map(|s| s.name.as_str())
                    .unwrap_or("someone"))
    }
}
```

Example templates:

```toml
[event_captions.betrayal.secret_shared_with_enemy]
templates = [
    "{primary_name} shares faction secrets with {secondary_name}",
    "At {location}, {primary_name} crosses a line that cannot be uncrossed",
    "{primary_name} tells {secondary_name} what no outsider should know",
]

[event_captions.ritual.reading_held]
templates = [
    "The weekly reading begins at {location}",
    "{primary_name} opens the book of {primary_faction}",
    "The faithful gather to hear their history",
]

[dramatic_irony.agent_unaware_of_betrayal]
pattern = "leader_doesnt_know_subordinate_betrayed"
templates = [
    "{unaware_agent} still trusts {betrayer}—for now",
    "If only {unaware_agent} knew what {betrayer} did at {betrayal_location}",
]
required_context = ["betrayal_event", "unaware_leader", "trust_still_high"]
```

#### 1.4 Output Generation

```rust
pub struct DirectorOutput {
    pub generated_at_tick: u64,
    pub camera_script: Vec<CameraInstruction>,
    pub commentary_queue: Vec<CommentaryItem>,
    pub active_threads: Vec<NarrativeThread>,
    pub highlights: Vec<HighlightMarker>,
}

pub struct HighlightMarker {
    pub event_id: String,
    pub highlight_type: String,  // "key_moment", "turning_point", "climax"
    pub suggested_clip_start: u64,
    pub suggested_clip_end: u64,
}

impl Director {
    pub fn process_tick(
        &mut self,
        events: &[Event],
        tensions: &[Tension],
        state: &WorldSnapshot,
    ) -> DirectorOutput {
        // Score and filter events
        let scored_events = self.score_events(events);
        let notable_events: Vec<_> = scored_events.iter()
            .filter(|(_, score)| *score > self.notability_threshold)
            .collect();
        
        // Update narrative threads
        self.update_threads(&notable_events, tensions);
        
        // Generate camera instructions
        let camera_script = self.generate_camera_script(tensions, &notable_events);
        
        // Generate commentary
        let commentary = self.generate_commentary(&notable_events, tensions, state);
        
        // Mark highlights for later summarization
        let highlights = self.mark_highlights(&notable_events);
        
        DirectorOutput {
            generated_at_tick: state.timestamp.tick,
            camera_script,
            commentary_queue: commentary,
            active_threads: self.threads.clone(),
            highlights,
        }
    }
}
```

### Phase 1 Deliverables

1. **Event scorer** with configurable weights
2. **Focus selector** using tension severity + thread fatigue
3. **Template-based captions** for all event types
4. **Basic dramatic irony** detection (betrayer + unaware target)
5. **Camera script output** format consumed by visualization
6. **Config file** for all tunable parameters
7. **Logging/debugging** output showing decision rationale

### Phase 1 Milestones

| Milestone | Description | Estimate |
|-----------|-------------|----------|
| 1.1 | Event scoring pipeline, config loading | 3-4 days |
| 1.2 | Tension tracking, thread management | 3-4 days |
| 1.3 | Camera instruction generation | 2-3 days |
| 1.4 | Template system, commentary output | 3-4 days |
| 1.5 | Integration testing with sample data | 2-3 days |

---

## Phase 2: Enhanced Pattern Detection

### Goal
Move beyond single-event scoring to multi-event pattern recognition. Detect narrative arcs, not just moments.

### New Capabilities

#### 2.1 Sequence Pattern Matching

```rust
pub struct PatternMatcher {
    patterns: Vec<NarrativePattern>,
}

pub struct NarrativePattern {
    pub pattern_id: String,
    pub name: String,
    pub event_sequence: Vec<PatternStep>,
    pub max_gap_ticks: u64,
    pub drama_multiplier: f32,
}

pub struct PatternStep {
    pub event_type: EventType,
    pub subtype: Option<String>,
    pub agent_role: String,  // "same_primary", "same_secondary", "any"
    pub required_tags: Vec<String>,
}

// Example: "Trust Erosion" pattern
// 1. Agent A has high trust in Agent B
// 2. Agent B breaks promise to A
// 3. Agent A witnesses or hears about it
// 4. Agent A's trust drops below threshold
// 5. Agent A takes retaliatory action OR contacts enemy faction

impl PatternMatcher {
    pub fn detect_patterns(
        &self,
        recent_events: &[Event],
        relationships: &RelationshipMap,
    ) -> Vec<DetectedPattern> {
        let mut detected = Vec::new();
        
        for pattern in &self.patterns {
            for potential_match in self.find_candidates(recent_events, pattern) {
                if self.validate_sequence(&potential_match, pattern) {
                    detected.push(DetectedPattern {
                        pattern_id: pattern.pattern_id.clone(),
                        events: potential_match,
                        confidence: self.calculate_confidence(&potential_match, pattern),
                    });
                }
            }
        }
        
        detected
    }
}
```

#### 2.2 Predictive Hooks

Use detected patterns to anticipate what's coming:

```rust
pub struct PredictiveHook {
    pub hook_id: String,
    pub pattern_in_progress: String,
    pub current_step: usize,
    pub predicted_next: Vec<PredictedEvent>,
    pub agents_to_watch: Vec<String>,
    pub locations_to_watch: Vec<String>,
}

pub struct PredictedEvent {
    pub event_type: EventType,
    pub probability: f32,
    pub estimated_ticks: u64,
    pub would_complete_pattern: bool,
}
```

This enables the dramatic irony the design doc mentions—Director knows betrayal is coming and can start following the betrayer before it happens.

#### 2.3 Relationship Graph Analysis

```rust
pub struct SocialAnalyzer {
    pub fn find_triangles(&self, relationships: &RelationshipMap) -> Vec<RelationshipTriangle> {
        // A trusts B, B trusts C, but A doesn't trust C
        // These are unstable and interesting
    }
    
    pub fn find_bridges(&self, relationships: &RelationshipMap) -> Vec<BridgeAgent> {
        // Agents with significant ties to multiple factions
    }
    
    pub fn find_cliques(&self, relationships: &RelationshipMap) -> Vec<Clique> {
        // Tight clusters that might act as a unit
    }
    
    pub fn find_isolated(&self, relationships: &RelationshipMap) -> Vec<String> {
        // Agents with few connections—vulnerable, unpredictable
    }
}
```

### Phase 2 Deliverables

1. **Pattern definition format** (config/code hybrid)
2. **Sequence matcher** with gap tolerance
3. **Predictive hook system** 
4. **Social graph analyzer**
5. **Enhanced thread tracking** using patterns
6. **Improved camera anticipation** (pre-position for predicted events)

### Phase 2 Milestones

| Milestone | Description | Estimate |
|-----------|-------------|----------|
| 2.1 | Pattern definition format, basic matcher | 4-5 days |
| 2.2 | Predictive hooks, anticipation logic | 3-4 days |
| 2.3 | Social graph analysis integration | 3-4 days |
| 2.4 | Thread tracker upgrade | 2-3 days |
| 2.5 | Testing with longer simulation runs | 3-4 days |

---

## Phase 3: LLM Integration

### Goal
Augment (not replace) the template system with LLM-generated content for richer narrative output.

### Design Principles for LLM Use

1. **LLM as author, not decision-maker**: Core directing logic stays deterministic. LLM generates prose, not camera instructions.

2. **Graceful fallback**: If LLM is unavailable/slow, templates fill in. The show must go on.

3. **Bounded context**: Send focused prompts with just the relevant events/agents, not the whole world state.

4. **Cache aggressively**: Similar situations can reuse generated content.

### LLM-Powered Features

#### 3.1 Narrative Summarization

```rust
pub struct NarrativeSummarizer {
    llm_client: LlmClient,
    template_fallback: CommentaryGenerator,
    cache: SummaryCache,
}

impl NarrativeSummarizer {
    pub async fn summarize_thread(
        &self,
        thread: &NarrativeThread,
        events: &[Event],
        style: NarrativeStyle,
    ) -> String {
        let cache_key = self.compute_cache_key(thread, events);
        
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached;
        }
        
        let prompt = self.build_summary_prompt(thread, events, style);
        
        match self.llm_client.complete(&prompt, self.timeout()).await {
            Ok(response) => {
                self.cache.insert(cache_key, response.clone());
                response
            }
            Err(_) => {
                // Fallback to template
                self.template_fallback.summarize_thread_basic(thread, events)
            }
        }
    }
    
    fn build_summary_prompt(
        &self,
        thread: &NarrativeThread,
        events: &[Event],
        style: NarrativeStyle,
    ) -> String {
        format!(r#"
You are a narrator for a medieval political drama. Summarize this storyline in 2-3 sentences.

Style: {style}

Key agents:
{agents}

Recent events in this storyline:
{events}

Current situation: {summary}

Write a dramatic summary that captures the tension and stakes. Do not use modern language.
"#,
            style = style.description(),
            agents = self.format_agents(thread),
            events = self.format_events(events),
            summary = thread.summary,
        )
    }
}

pub enum NarrativeStyle {
    Chronicle,      // Dry, historical
    Dramatic,       // Heightened, emotional  
    Whispered,      // Conspiratorial, secrets
    Epic,           // Grand, consequential
}
```

#### 3.2 Dynamic Commentary

```rust
pub struct DynamicCommentary {
    llm_client: LlmClient,
    templates: CommentaryTemplates,
}

impl DynamicCommentary {
    pub async fn generate_irony_comment(
        &self,
        situation: &IronySituation,
    ) -> CommentaryItem {
        let prompt = format!(r#"
Write a single sentence of dramatic irony for a medieval simulation.

The situation: {unaware_agent} currently trusts {betrayer} and doesn't know that {betrayer} recently {betrayal_action}.

Write one short, evocative sentence that hints at the dramatic irony without being too on-the-nose. Medieval tone.
"#,
            unaware_agent = situation.unaware_agent.name,
            betrayer = situation.betrayer.name,
            betrayal_action = situation.betrayal_summary,
        );
        
        // ... with fallback to templates
    }
    
    pub async fn generate_tension_teaser(
        &self,
        tension: &Tension,
    ) -> CommentaryItem {
        let prompt = format!(r#"
Write a single ominous sentence teasing upcoming conflict.

The situation: {summary}

Predicted outcomes: {outcomes}

Write one short sentence that builds anticipation without spoiling. Medieval tone, foreboding.
"#,
            summary = tension.summary,
            outcomes = self.format_outcomes(&tension.predicted_outcomes),
        );
        
        // ...
    }
}
```

#### 3.3 Historical Chronicle Generation

For end-of-session or periodic "what happened" summaries:

```rust
pub struct ChronicleGenerator {
    llm_client: LlmClient,
}

impl ChronicleGenerator {
    pub async fn generate_chronicle(
        &self,
        time_range: (SimTimestamp, SimTimestamp),
        events: &[Event],
        tensions_resolved: &[Tension],
        significant_deaths: &[Event],
        faction_changes: &[FactionChange],
    ) -> Chronicle {
        let prompt = format!(r#"
Write a historical chronicle entry for this period in a medieval simulation.

Time period: {start} to {end}

Major events:
{events}

Resolved storylines:
{tensions}

Deaths of note:
{deaths}

Faction changes:
{factions}

Write 3-5 paragraphs in the style of a medieval chronicle. Focus on causation—how one event led to another. Name specific people and places. Be dramatic but grounded.
"#,
            // ... parameters
        );
        
        let content = self.llm_client.complete(&prompt, Duration::from_secs(30)).await?;
        
        Chronicle {
            time_range,
            content,
            key_figures: self.extract_key_figures(events),
            turning_points: self.identify_turning_points(events, tensions_resolved),
        }
    }
}
```

#### 3.4 Agent Voice (Experimental)

Generate what agents might say in key moments:

```rust
pub struct AgentVoice {
    llm_client: LlmClient,
}

impl AgentVoice {
    pub async fn generate_reaction(
        &self,
        agent: &Agent,
        event: &Event,
        relationship_context: &[Relationship],
    ) -> Option<String> {
        // Only for high-drama moments
        if event.drama_score < 0.7 {
            return None;
        }
        
        let prompt = format!(r#"
Write a single line of dialogue for a medieval character reacting to an event.

Character: {name}
Traits: {traits}
Their role in this event: {role}
What happened: {event_summary}

Write one short line they might say or think. Match their personality. No modern language.
"#,
            name = agent.name,
            traits = self.format_traits(&agent.traits),
            role = self.determine_role(agent, event),
            event_summary = self.summarize_event(event),
        );
        
        // ...
    }
}
```

### LLM Integration Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    LLM Integration Layer                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │   Request    │───▶│    LLM       │───▶│   Response   │  │
│  │   Builder    │    │   Client     │    │   Parser     │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│         │                   │                   │           │
│         │                   ▼                   │           │
│         │            ┌──────────────┐           │           │
│         │            │    Cache     │           │           │
│         │            │   (Redis/    │           │           │
│         │            │   In-Memory) │           │           │
│         │            └──────────────┘           │           │
│         │                                       │           │
│         ▼                                       ▼           │
│  ┌──────────────┐                        ┌──────────────┐  │
│  │   Template   │◀───── Fallback ───────│   Timeout/   │  │
│  │   System     │                        │   Error      │  │
│  └──────────────┘                        └──────────────┘  │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Configuration

```toml
[llm]
provider = "anthropic"  # or "openai", "local"
model = "claude-sonnet-4-20250514"
timeout_ms = 5000
max_retries = 2

[llm.rate_limits]
requests_per_minute = 20
tokens_per_minute = 40000

[llm.cache]
enabled = true
ttl_seconds = 3600
max_entries = 10000

[llm.features]
narrative_summaries = true
dynamic_commentary = true
chronicle_generation = true
agent_voice = false  # Experimental, off by default

[llm.fallback]
always_have_template = true
log_fallbacks = true
```

### Phase 3 Deliverables

1. **LLM client abstraction** (provider-agnostic)
2. **Prompt templates** for each use case
3. **Caching layer** with TTL and similarity matching
4. **Fallback system** to templates
5. **Rate limiting** and cost tracking
6. **Narrative summarizer**
7. **Dynamic commentary generator**
8. **Chronicle generator**
9. **Agent voice system** (experimental)

### Phase 3 Milestones

| Milestone | Description | Estimate |
|-----------|-------------|----------|
| 3.1 | LLM client, caching, rate limiting | 4-5 days |
| 3.2 | Narrative summarization | 3-4 days |
| 3.3 | Dynamic commentary | 3-4 days |
| 3.4 | Chronicle generation | 3-4 days |
| 3.5 | Agent voice (experimental) | 2-3 days |
| 3.6 | Integration, tuning, fallback testing | 4-5 days |

---

## Testing Strategy

### Unit Tests
- Event scoring produces expected rankings
- Pattern matcher finds known sequences
- Template filling handles edge cases
- Camera instruction generation is valid

### Integration Tests
- Process sample event streams end-to-end
- Verify output format compatibility with viz layer
- Test fallback behavior when LLM unavailable

### Simulation Tests
- Run against actual simulation output
- Human evaluation of narrative quality
- A/B testing different configurations

### Test Data
- Golden set of events with expected Director outputs
- Edge cases: empty tensions, overwhelming events, single-agent worlds
- Long-running sequences for thread tracking

---

## Open Questions

1. **How much look-ahead?** Director runs ahead of visualization—but how far? More look-ahead = better anticipation but higher memory and potential for stale decisions if interventions occur.

2. **Thread lifecycle**: When does a thread truly end? Some stories have long tails (consequences of a betrayal playing out over years).

3. **Multi-camera**: Should we support parallel storyline visualization (split screen, picture-in-picture) or always single focus?

4. **User override integration**: If user manually moves camera, how does Director adapt? Pause recommendations? Gently suggest returning to the action?

5. **LLM consistency**: How do we maintain consistent voice/tone across LLM calls? System prompts? Few-shot examples? Fine-tuning?

---

*Document version 0.1 — December 2024*
