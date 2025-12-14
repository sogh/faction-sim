//! Template-based commentary generation.
//!
//! Generates text overlays and captions for the visualization based on
//! events, tensions, and dramatic irony situations.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use sim_events::{Event, EventSubtype, EventType, Tension, WorldSnapshot};

use crate::config::CommentaryConfig;
use crate::output::{generate_commentary_id, CommentaryItem, CommentaryType};

/// Templates for generating commentary text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommentaryTemplates {
    /// Event captions keyed by "event_type.subtype" (e.g., "betrayal.secret_shared_with_enemy")
    #[serde(default)]
    pub event_captions: HashMap<String, Vec<String>>,

    /// Dramatic irony templates
    #[serde(default)]
    pub dramatic_irony: Vec<IronyTemplate>,

    /// Context reminder templates
    #[serde(default)]
    pub context_reminders: Vec<ReminderTemplate>,

    /// Tension teaser templates
    #[serde(default)]
    pub tension_teasers: Vec<TeaserTemplate>,
}

impl CommentaryTemplates {
    /// Loads templates from a TOML file.
    pub fn from_file(path: &Path) -> Result<Self, TemplateError> {
        let content = std::fs::read_to_string(path).map_err(TemplateError::IoError)?;
        Self::from_str(&content)
    }

    /// Parses templates from a TOML string.
    pub fn from_str(content: &str) -> Result<Self, TemplateError> {
        toml::from_str(content).map_err(TemplateError::TomlError)
    }

    /// Gets templates for a specific event type and subtype.
    pub fn get_event_templates(&self, event_type: &str, subtype: &str) -> Option<&Vec<String>> {
        let key = format!("{}.{}", event_type, subtype);
        self.event_captions.get(&key)
    }

    /// Gets templates for an event type (without subtype).
    pub fn get_type_templates(&self, event_type: &str) -> Option<&Vec<String>> {
        self.event_captions.get(event_type)
    }
}

/// Template for dramatic irony situations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IronyTemplate {
    /// Pattern identifier (e.g., "unaware_of_betrayal")
    pub pattern: String,
    /// Template strings with placeholders
    pub templates: Vec<String>,
    /// Required context keys for this template
    #[serde(default)]
    pub required_context: Vec<String>,
}

/// Template for context reminders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderTemplate {
    /// Context type (e.g., "past_betrayal", "old_alliance")
    pub context_type: String,
    /// Template strings
    pub templates: Vec<String>,
    /// Minimum ticks ago for this reminder to apply
    #[serde(default)]
    pub min_ticks_ago: u64,
}

/// Template for tension teasers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeaserTemplate {
    /// Tension type this applies to
    pub tension_type: String,
    /// Template strings
    pub templates: Vec<String>,
    /// Minimum severity for this teaser
    #[serde(default)]
    pub min_severity: f32,
}

/// Situation where dramatic irony exists.
#[derive(Debug, Clone)]
pub struct IronySituation {
    /// Type of irony situation
    pub situation_type: String,
    /// Agent who is unaware
    pub unaware_agent_name: String,
    /// ID of the unaware agent
    pub unaware_agent_id: String,
    /// Agent who did the betrayal (if applicable)
    pub betrayer_name: Option<String>,
    /// ID of the betrayer
    pub betrayer_id: Option<String>,
    /// The secret information
    pub secret_info: String,
    /// Location where betrayal occurred (if applicable)
    pub betrayal_location: Option<String>,
    /// Related event ID
    pub betrayal_event_id: Option<String>,
}

impl IronySituation {
    /// Creates a new irony situation for an unaware betrayal.
    pub fn unaware_of_betrayal(
        unaware_agent_id: impl Into<String>,
        unaware_agent_name: impl Into<String>,
        betrayer_id: impl Into<String>,
        betrayer_name: impl Into<String>,
        betrayal_event_id: impl Into<String>,
        betrayal_location: Option<String>,
    ) -> Self {
        Self {
            situation_type: "unaware_of_betrayal".to_string(),
            unaware_agent_name: unaware_agent_name.into(),
            unaware_agent_id: unaware_agent_id.into(),
            betrayer_name: Some(betrayer_name.into()),
            betrayer_id: Some(betrayer_id.into()),
            secret_info: "betrayal".to_string(),
            betrayal_location,
            betrayal_event_id: Some(betrayal_event_id.into()),
        }
    }
}

/// Record of a betrayal event for irony tracking.
#[derive(Debug, Clone)]
pub struct BetrayalRecord {
    /// The event ID of the betrayal
    pub event_id: String,
    /// ID of the agent who betrayed
    pub betrayer_id: String,
    /// Name of the betrayer
    pub betrayer_name: String,
    /// IDs of agents affected by the betrayal
    pub affected_ids: Vec<String>,
    /// Tick when the betrayal occurred
    pub tick: u64,
    /// Location where the betrayal occurred
    pub location: Option<String>,
    /// Agents who have discovered this betrayal
    pub discovered_by: HashSet<String>,
}

impl BetrayalRecord {
    /// Creates a new betrayal record from an event.
    pub fn from_event(event: &Event) -> Option<Self> {
        // Only process betrayal events
        if event.event_type != EventType::Betrayal {
            return None;
        }

        let betrayer_id = event.actors.primary.agent_id.clone();
        let betrayer_name = event.actors.primary.name.clone();
        let location = Some(event.actors.primary.location.clone());

        // Affected agents are:
        // 1. The secondary actor (if any)
        // 2. All explicitly affected actors
        // 3. Potentially all members of the betrayer's faction (simplified: just use affected list)
        let affected_ids: Vec<String> = event
            .actors
            .affected
            .iter()
            .map(|a| a.agent_id.clone())
            .collect();

        // If there's a secondary actor who isn't the betrayer, they're not "affected" in the same way
        // The secondary is typically the one receiving the secret, not someone being betrayed
        // So we don't add them to affected_ids

        // If no explicit affected agents, this betrayal doesn't have trackable victims
        // (e.g., defection might affect the whole faction, but we'd need more context)
        if affected_ids.is_empty() {
            // Fall back: anyone in the betrayer's faction who isn't the betrayer is affected
            // For now, we'll just return None if there are no explicitly affected agents
            // In a real implementation, we'd look up faction members
            return None;
        }

        Some(Self {
            event_id: event.event_id.clone(),
            betrayer_id,
            betrayer_name,
            affected_ids,
            tick: event.timestamp.tick,
            location,
            discovered_by: HashSet::new(),
        })
    }

    /// Checks if a specific agent has discovered this betrayal.
    pub fn is_discovered_by(&self, agent_id: &str) -> bool {
        self.discovered_by.contains(agent_id)
    }

    /// Checks if all affected agents have discovered this betrayal.
    pub fn is_fully_discovered(&self) -> bool {
        self.affected_ids.iter().all(|id| self.discovered_by.contains(id))
    }
}

/// Detects dramatic irony situations based on betrayals and trust relationships.
#[derive(Debug, Clone, Default)]
pub struct IronyDetector {
    /// Recent betrayals that may create irony situations
    recent_betrayals: Vec<BetrayalRecord>,
    /// Trust threshold below which an agent is considered to have "discovered" betrayal
    trust_threshold: f32,
}

impl IronyDetector {
    /// Creates a new irony detector with default settings.
    pub fn new() -> Self {
        Self {
            recent_betrayals: Vec::new(),
            trust_threshold: 0.5,
        }
    }

    /// Creates a new irony detector with a custom trust threshold.
    pub fn with_trust_threshold(trust_threshold: f32) -> Self {
        Self {
            recent_betrayals: Vec::new(),
            trust_threshold,
        }
    }

    /// Records a betrayal event for tracking.
    ///
    /// Only betrayal-type events will be recorded.
    pub fn record_betrayal(&mut self, event: &Event) {
        if let Some(record) = BetrayalRecord::from_event(event) {
            self.recent_betrayals.push(record);
        }
    }

    /// Marks a betrayal as discovered by an agent.
    ///
    /// This should be called when an agent learns about a betrayal through
    /// communication, observation, or trust erosion.
    pub fn mark_discovered(&mut self, betrayal_event_id: &str, discoverer_id: &str) {
        for record in &mut self.recent_betrayals {
            if record.event_id == betrayal_event_id {
                record.discovered_by.insert(discoverer_id.to_string());
                break;
            }
        }
    }

    /// Detects irony situations based on current world state.
    ///
    /// Detection logic for "unaware_of_betrayal":
    /// 1. For each recorded betrayal not yet discovered by affected parties
    /// 2. Check if any affected agent still has high trust in the betrayer
    /// 3. If reliability trust > threshold, they're still unaware = irony opportunity
    pub fn detect_irony(&self, state: &WorldSnapshot) -> Vec<IronySituation> {
        let mut situations = Vec::new();

        for record in &self.recent_betrayals {
            // Skip fully discovered betrayals
            if record.is_fully_discovered() {
                continue;
            }

            // Check each affected agent
            for affected_id in &record.affected_ids {
                // Skip if this agent already discovered the betrayal
                if record.is_discovered_by(affected_id) {
                    continue;
                }

                // Check the trust relationship from affected -> betrayer
                if let Some(relationship) = state.get_relationship(affected_id, &record.betrayer_id) {
                    // If they still trust the betrayer, there's irony
                    if relationship.reliability > self.trust_threshold {
                        // Get the agent's name for the situation
                        let agent_name = state
                            .find_agent(affected_id)
                            .map(|a| a.name.clone())
                            .unwrap_or_else(|| affected_id.clone());

                        situations.push(IronySituation::unaware_of_betrayal(
                            affected_id,
                            agent_name,
                            &record.betrayer_id,
                            &record.betrayer_name,
                            &record.event_id,
                            record.location.clone(),
                        ));
                    }
                }
            }
        }

        situations
    }

    /// Cleans up old betrayal records.
    ///
    /// Removes fully discovered betrayals and those older than max_age_ticks.
    pub fn cleanup(&mut self, current_tick: u64, max_age_ticks: u64) {
        self.recent_betrayals.retain(|record| {
            !record.is_fully_discovered() && current_tick - record.tick < max_age_ticks
        });
    }

    /// Returns the number of tracked betrayals.
    pub fn betrayal_count(&self) -> usize {
        self.recent_betrayals.len()
    }

    /// Returns a reference to all betrayal records.
    pub fn betrayals(&self) -> &[BetrayalRecord] {
        &self.recent_betrayals
    }
}

/// Errors that can occur during template operations.
#[derive(Debug)]
pub enum TemplateError {
    /// IO error reading template file
    IoError(std::io::Error),
    /// Error parsing TOML
    TomlError(toml::de::Error),
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateError::IoError(e) => write!(f, "IO error: {}", e),
            TemplateError::TomlError(e) => write!(f, "TOML parse error: {}", e),
        }
    }
}

impl std::error::Error for TemplateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TemplateError::IoError(e) => Some(e),
            TemplateError::TomlError(e) => Some(e),
        }
    }
}

/// Generates commentary items from events and tensions.
#[derive(Debug, Clone)]
pub struct CommentaryGenerator {
    /// Templates for generating text
    templates: CommentaryTemplates,
    /// Configuration settings
    config: CommentaryConfig,
    /// Current tick for ID generation
    current_tick: u64,
    /// Sequence number for IDs
    commentary_sequence: u32,
}

impl CommentaryGenerator {
    /// Creates a new commentary generator with templates and config.
    pub fn new(templates: CommentaryTemplates, config: CommentaryConfig) -> Self {
        Self {
            templates,
            config,
            current_tick: 0,
            commentary_sequence: 0,
        }
    }

    /// Creates a generator with default templates and config.
    pub fn with_defaults() -> Self {
        Self::new(default_templates(), CommentaryConfig::default())
    }

    /// Loads templates from a file.
    pub fn from_template_file(path: &Path, config: CommentaryConfig) -> Result<Self, TemplateError> {
        let templates = CommentaryTemplates::from_file(path)?;
        Ok(Self::new(templates, config))
    }

    /// Sets the current tick for ID generation.
    pub fn set_current_tick(&mut self, tick: u64) {
        if tick != self.current_tick {
            self.current_tick = tick;
            self.commentary_sequence = 0;
        }
    }

    /// Generates a caption for an event if it meets the drama threshold.
    pub fn caption_event(
        &mut self,
        event: &Event,
        timestamp: sim_events::SimTimestamp,
    ) -> Option<CommentaryItem> {
        // Check minimum drama threshold
        if event.drama_score < self.config.min_drama_for_caption {
            return None;
        }

        // Get event type and subtype as strings
        let event_type_str = event_type_to_string(&event.event_type);
        let subtype_str = event_subtype_to_string(&event.subtype);

        // Try to find a template
        let template = self
            .templates
            .get_event_templates(&event_type_str, &subtype_str)
            .or_else(|| self.templates.get_type_templates(&event_type_str))
            .and_then(|templates| templates.choose(&mut rand::thread_rng()))?;

        // Fill the template
        let content = self.fill_event_template(template, event);
        let duration = self.calculate_duration(&content);

        let item_id = self.next_commentary_id();
        Some(
            CommentaryItem::new(item_id, timestamp, CommentaryType::EventCaption, content)
                .with_duration(duration)
                .with_priority(event.drama_score)
                .with_agents(event.all_agent_ids().into_iter().map(String::from).collect()),
        )
    }

    /// Generates dramatic irony commentary.
    pub fn generate_irony(
        &mut self,
        situation: &IronySituation,
        timestamp: sim_events::SimTimestamp,
    ) -> Option<CommentaryItem> {
        if !self.config.enable_dramatic_irony {
            return None;
        }

        // Find matching irony template
        let irony_template = self
            .templates
            .dramatic_irony
            .iter()
            .find(|t| t.pattern == situation.situation_type)?;

        let template = irony_template
            .templates
            .choose(&mut rand::thread_rng())?;

        // Fill the template
        let content = self.fill_irony_template(template, situation);
        let duration = self.calculate_duration(&content);

        let item_id = self.next_commentary_id();
        let mut item = CommentaryItem::new(
            item_id,
            timestamp,
            CommentaryType::DramaticIrony,
            content,
        )
        .with_duration(duration)
        .with_priority(0.8); // Irony is high priority

        // Add related agents
        let mut agents = vec![situation.unaware_agent_name.clone()];
        if let Some(ref betrayer) = situation.betrayer_name {
            agents.push(betrayer.clone());
        }
        item = item.with_agents(agents);

        Some(item)
    }

    /// Generates a teaser for a tension.
    pub fn generate_teaser(
        &mut self,
        tension: &Tension,
        timestamp: sim_events::SimTimestamp,
    ) -> Option<CommentaryItem> {
        if !self.config.enable_tension_teasers {
            return None;
        }

        let tension_type_str = format!("{:?}", tension.tension_type).to_lowercase();

        // Find matching teaser template
        let teaser_template = self
            .templates
            .tension_teasers
            .iter()
            .find(|t| t.tension_type == tension_type_str && tension.severity >= t.min_severity)?;

        let template = teaser_template
            .templates
            .choose(&mut rand::thread_rng())?;

        // Fill the template
        let content = self.fill_tension_template(template, tension);
        let duration = self.calculate_duration(&content);

        let item_id = self.next_commentary_id();
        let agents: Vec<String> = tension
            .key_agents
            .iter()
            .map(|a| a.agent_id.clone())
            .collect();

        Some(
            CommentaryItem::new(item_id, timestamp, CommentaryType::TensionTeaser, content)
                .with_duration(duration)
                .with_priority(tension.severity * 0.7)
                .with_agents(agents)
                .with_tension(&tension.tension_id),
        )
    }

    /// Fills an event template with data from the event.
    ///
    /// Supported placeholders:
    /// - {primary_name}, {primary_faction}, {primary_role}
    /// - {secondary_name}, {secondary_faction}
    /// - {location}
    /// - {affected_names} (comma-separated)
    pub fn fill_event_template(&self, template: &str, event: &Event) -> String {
        let mut result = template.to_string();

        // Primary actor
        result = result.replace("{primary_name}", &event.actors.primary.name);
        result = result.replace("{primary_faction}", &event.actors.primary.faction);
        result = result.replace("{primary_role}", &event.actors.primary.role);

        // Secondary actor (empty string if none)
        let secondary_name = event
            .actors
            .secondary
            .as_ref()
            .map(|s| s.name.as_str())
            .unwrap_or("");
        let secondary_faction = event
            .actors
            .secondary
            .as_ref()
            .map(|s| s.faction.as_str())
            .unwrap_or("");
        result = result.replace("{secondary_name}", secondary_name);
        result = result.replace("{secondary_faction}", secondary_faction);

        // Location
        result = result.replace("{location}", &event.actors.primary.location);

        // Affected names
        let affected_names: Vec<&str> = event
            .actors
            .affected
            .iter()
            .map(|a| a.name.as_str())
            .collect();
        result = result.replace("{affected_names}", &affected_names.join(", "));

        result
    }

    /// Fills an irony template with situation data.
    fn fill_irony_template(&self, template: &str, situation: &IronySituation) -> String {
        let mut result = template.to_string();

        result = result.replace("{unaware_agent}", &situation.unaware_agent_name);

        if let Some(ref betrayer) = situation.betrayer_name {
            result = result.replace("{betrayer}", betrayer);
        }

        if let Some(ref location) = situation.betrayal_location {
            result = result.replace("{betrayal_location}", location);
        }

        result = result.replace("{secret_info}", &situation.secret_info);

        result
    }

    /// Fills a tension template with tension data.
    fn fill_tension_template(&self, template: &str, tension: &Tension) -> String {
        let mut result = template.to_string();

        // Primary agent (if any)
        if let Some(agent) = tension.key_agents.first() {
            result = result.replace("{primary_name}", &agent.agent_id);
            result = result.replace("{primary_role}", &agent.role_in_tension);
        }

        // Location (if any)
        if let Some(location) = tension.key_locations.first() {
            result = result.replace("{location}", location);
        }

        // Summary
        result = result.replace("{summary}", &tension.summary);

        // Hook (first narrative hook if any)
        if let Some(hook) = tension.narrative_hooks.first() {
            result = result.replace("{hook}", hook);
        }

        result
    }

    /// Calculates display duration based on content length.
    fn calculate_duration(&self, content: &str) -> u32 {
        let base = self.config.base_display_duration_ticks;
        let extra = (content.len() as f32 * self.config.ticks_per_character) as u32;
        base + extra
    }

    /// Generates the next commentary ID.
    fn next_commentary_id(&mut self) -> String {
        self.commentary_sequence += 1;
        generate_commentary_id(self.current_tick, self.commentary_sequence)
    }

    /// Returns a reference to the templates.
    pub fn templates(&self) -> &CommentaryTemplates {
        &self.templates
    }

    /// Returns a reference to the config.
    pub fn config(&self) -> &CommentaryConfig {
        &self.config
    }
}

impl Default for CommentaryGenerator {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Converts EventType to a string for template lookup.
fn event_type_to_string(event_type: &EventType) -> String {
    match event_type {
        EventType::Movement => "movement",
        EventType::Communication => "communication",
        EventType::Betrayal => "betrayal",
        EventType::Loyalty => "loyalty",
        EventType::Conflict => "conflict",
        EventType::Cooperation => "cooperation",
        EventType::Faction => "faction",
        EventType::Archive => "archive",
        EventType::Ritual => "ritual",
        EventType::Resource => "resource",
        EventType::Death => "death",
        EventType::Birth => "birth",
    }
    .to_string()
}

/// Converts EventSubtype to a string for template lookup.
fn event_subtype_to_string(subtype: &EventSubtype) -> String {
    match subtype {
        EventSubtype::Movement(s) => format!("{:?}", s).to_lowercase(),
        EventSubtype::Communication(s) => format!("{:?}", s).to_lowercase(),
        EventSubtype::Betrayal(s) => format!("{:?}", s).to_lowercase(),
        EventSubtype::Loyalty(s) => format!("{:?}", s).to_lowercase(),
        EventSubtype::Conflict(s) => format!("{:?}", s).to_lowercase(),
        EventSubtype::Cooperation(s) => format!("{:?}", s).to_lowercase(),
        EventSubtype::Faction(s) => format!("{:?}", s).to_lowercase(),
        EventSubtype::Archive(s) => format!("{:?}", s).to_lowercase(),
        EventSubtype::Ritual(s) => format!("{:?}", s).to_lowercase(),
        EventSubtype::Resource(s) => format!("{:?}", s).to_lowercase(),
        EventSubtype::Death(s) => format!("{:?}", s).to_lowercase(),
        EventSubtype::Birth(s) => format!("{:?}", s).to_lowercase(),
    }
}

/// Returns default templates with common event captions.
pub fn default_templates() -> CommentaryTemplates {
    let mut event_captions = HashMap::new();

    // Betrayal events
    event_captions.insert(
        "betrayal.secretsharedwithenemy".to_string(),
        vec![
            "{primary_name} shares faction secrets with {secondary_name}".to_string(),
            "At {location}, {primary_name} crosses a line that cannot be uncrossed".to_string(),
            "A whispered betrayal: {primary_name} reveals secrets to {secondary_name}".to_string(),
        ],
    );
    event_captions.insert(
        "betrayal.defection".to_string(),
        vec![
            "{primary_name} abandons {primary_faction}".to_string(),
            "A traitor reveals themselves: {primary_name} defects".to_string(),
            "{primary_name} turns their back on {primary_faction}".to_string(),
        ],
    );
    event_captions.insert(
        "betrayal.sabotage".to_string(),
        vec![
            "{primary_name} sabotages their own faction".to_string(),
            "Sabotage in {location}: {primary_name}'s loyalty fractures".to_string(),
        ],
    );

    // Death events
    event_captions.insert(
        "death.killed".to_string(),
        vec![
            "{primary_name} has fallen".to_string(),
            "Death claims {primary_name}".to_string(),
            "{primary_name}'s story ends here".to_string(),
        ],
    );
    event_captions.insert(
        "death.natural".to_string(),
        vec![
            "{primary_name} passes peacefully".to_string(),
            "Time claims {primary_name}".to_string(),
        ],
    );
    event_captions.insert(
        "death.executed".to_string(),
        vec![
            "{primary_name} is executed".to_string(),
            "Justice—or vengeance—claims {primary_name}".to_string(),
        ],
    );

    // Ritual events
    event_captions.insert(
        "ritual.readingheld".to_string(),
        vec![
            "The faithful gather at {location}".to_string(),
            "{primary_name} opens the book of {primary_faction}".to_string(),
            "A ritual reading begins at {location}".to_string(),
        ],
    );
    event_captions.insert(
        "ritual.readingdisrupted".to_string(),
        vec![
            "The reading is disrupted!".to_string(),
            "Chaos at {location}: the ritual cannot continue".to_string(),
        ],
    );

    // Movement events
    event_captions.insert(
        "movement.travel".to_string(),
        vec![
            "{primary_name} journeys to {location}".to_string(),
            "{primary_name} arrives at {location}".to_string(),
        ],
    );
    event_captions.insert(
        "movement.flee".to_string(),
        vec![
            "{primary_name} flees in desperation".to_string(),
            "Fear drives {primary_name} away".to_string(),
        ],
    );

    // Conflict events
    event_captions.insert(
        "conflict.fight".to_string(),
        vec![
            "Violence erupts between {primary_name} and {secondary_name}".to_string(),
            "{primary_name} clashes with {secondary_name}".to_string(),
        ],
    );
    event_captions.insert(
        "conflict.duel".to_string(),
        vec![
            "{primary_name} faces {secondary_name} in single combat".to_string(),
            "A duel to settle old scores".to_string(),
        ],
    );

    // Cooperation events
    event_captions.insert(
        "cooperation.allianceformed".to_string(),
        vec![
            "{primary_name} and {secondary_name} forge an alliance".to_string(),
            "New bonds form between {primary_faction} and {secondary_faction}".to_string(),
        ],
    );
    event_captions.insert(
        "cooperation.trade".to_string(),
        vec![
            "{primary_name} trades with {secondary_name}".to_string(),
        ],
    );

    // Faction events
    event_captions.insert(
        "faction.join".to_string(),
        vec![
            "{primary_name} joins {primary_faction}".to_string(),
            "A new member for {primary_faction}: {primary_name}".to_string(),
        ],
    );
    event_captions.insert(
        "faction.exile".to_string(),
        vec![
            "{primary_name} is exiled from {primary_faction}".to_string(),
            "Cast out: {primary_name} loses everything".to_string(),
        ],
    );

    // Dramatic irony patterns
    let dramatic_irony = vec![
        IronyTemplate {
            pattern: "unaware_of_betrayal".to_string(),
            templates: vec![
                "{unaware_agent} still trusts {betrayer}—for now".to_string(),
                "If only {unaware_agent} knew what {betrayer} did".to_string(),
                "{unaware_agent} has no idea about {betrayer}'s treachery".to_string(),
            ],
            required_context: vec!["unaware_agent".to_string(), "betrayer".to_string()],
        },
        IronyTemplate {
            pattern: "walking_into_trap".to_string(),
            templates: vec![
                "{unaware_agent} walks unknowingly toward danger".to_string(),
                "They don't know what awaits them at {betrayal_location}".to_string(),
            ],
            required_context: vec!["unaware_agent".to_string()],
        },
    ];

    // Tension teasers
    let tension_teasers = vec![
        TeaserTemplate {
            tension_type: "brewingbetrayal".to_string(),
            templates: vec![
                "Loyalty frays at the edges...".to_string(),
                "Someone is having second thoughts".to_string(),
                "Trust is a fragile thing".to_string(),
            ],
            min_severity: 0.3,
        },
        TeaserTemplate {
            tension_type: "resourceconflict".to_string(),
            templates: vec![
                "Resources grow scarce...".to_string(),
                "Winter stores are running low...".to_string(),
                "There isn't enough for everyone".to_string(),
            ],
            min_severity: 0.4,
        },
        TeaserTemplate {
            tension_type: "successioncrisis".to_string(),
            templates: vec![
                "Leadership hangs in the balance".to_string(),
                "Who will lead when the dust settles?".to_string(),
            ],
            min_severity: 0.5,
        },
        TeaserTemplate {
            tension_type: "revengearc".to_string(),
            templates: vec![
                "Old wounds refuse to heal".to_string(),
                "Vengeance simmers beneath the surface".to_string(),
            ],
            min_severity: 0.4,
        },
    ];

    CommentaryTemplates {
        event_captions,
        dramatic_irony,
        context_reminders: Vec::new(),
        tension_teasers,
    }
}

/// Returns default templates as a TOML string.
pub fn default_templates_toml() -> String {
    r#"# Commentary Templates

[event_captions]
"betrayal.secretsharedwithenemy" = [
    "{primary_name} shares faction secrets with {secondary_name}",
    "At {location}, {primary_name} crosses a line that cannot be uncrossed",
]
"betrayal.defection" = [
    "{primary_name} abandons {primary_faction}",
    "A traitor reveals themselves: {primary_name} defects",
]
"death.killed" = [
    "{primary_name} has fallen",
    "Death claims {primary_name}",
]
"ritual.readingheld" = [
    "The faithful gather at {location}",
    "{primary_name} opens the book of {primary_faction}",
]
"movement.travel" = [
    "{primary_name} journeys to {location}",
]

[[dramatic_irony]]
pattern = "unaware_of_betrayal"
templates = [
    "{unaware_agent} still trusts {betrayer}—for now",
    "If only {unaware_agent} knew what {betrayer} did",
]
required_context = ["unaware_agent", "betrayer"]

[[tension_teasers]]
tension_type = "brewingbetrayal"
templates = [
    "Loyalty frays at the edges...",
    "Someone is having second thoughts",
]
min_severity = 0.3

[[tension_teasers]]
tension_type = "resourceconflict"
templates = [
    "Resources grow scarce...",
    "Winter stores are running low...",
]
min_severity = 0.4
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_events::{
        ActorSet, ActorSnapshot, AffectedActor, BetrayalSubtype, EventContext, EventOutcome,
        GeneralOutcome, MovementSubtype, Season, SimTimestamp, TensionStatus, TensionType,
    };

    fn test_timestamp() -> SimTimestamp {
        SimTimestamp::new(1000, 1, Season::Spring, 10)
    }

    fn make_betrayal_event() -> Event {
        let primary = ActorSnapshot::new(
            "agent_mira",
            "Mira of Thornwood",
            "thornwood",
            "scout",
            "eastern_bridge",
        );
        let secondary = ActorSnapshot::new(
            "agent_voss",
            "Voss the Quiet",
            "ironmere",
            "spymaster",
            "eastern_bridge",
        );

        Event {
            event_id: "evt_00001".to_string(),
            timestamp: test_timestamp(),
            event_type: EventType::Betrayal,
            subtype: EventSubtype::Betrayal(BetrayalSubtype::SecretSharedWithEnemy),
            actors: ActorSet::with_secondary(primary, secondary),
            context: EventContext::new("trust_eroded"),
            outcome: EventOutcome::General(GeneralOutcome::default()),
            drama_tags: vec!["betrayal".to_string()],
            drama_score: 0.85,
            connected_events: vec![],
        }
    }

    fn make_movement_event() -> Event {
        let actor = ActorSnapshot::new(
            "agent_mira",
            "Mira",
            "thornwood",
            "scout",
            "village_center",
        );

        Event {
            event_id: "evt_00002".to_string(),
            timestamp: test_timestamp(),
            event_type: EventType::Movement,
            subtype: EventSubtype::Movement(MovementSubtype::Travel),
            actors: ActorSet::primary_only(actor),
            context: EventContext::new("patrol"),
            outcome: EventOutcome::General(GeneralOutcome::default()),
            drama_tags: vec![],
            drama_score: 0.1,
            connected_events: vec![],
        }
    }

    fn make_tension() -> Tension {
        let mut tension = Tension::new(
            "tens_00001",
            TensionType::BrewingBetrayal,
            1000,
            "Mira's loyalty is wavering",
        );
        tension.severity = 0.6;
        tension.status = TensionStatus::Escalating;
        tension.add_agent_inline("agent_mira", "potential_traitor", "uncertain");
        tension.add_narrative_hook("Something is wrong with Mira");
        tension
    }

    #[test]
    fn test_default_templates() {
        let templates = default_templates();
        assert!(!templates.event_captions.is_empty());
        assert!(!templates.dramatic_irony.is_empty());
        assert!(!templates.tension_teasers.is_empty());
    }

    #[test]
    fn test_templates_from_toml() {
        let toml = default_templates_toml();
        let templates = CommentaryTemplates::from_str(&toml).unwrap();

        assert!(templates.event_captions.contains_key("betrayal.secretsharedwithenemy"));
        assert!(!templates.dramatic_irony.is_empty());
    }

    #[test]
    fn test_commentary_generator_creation() {
        let generator = CommentaryGenerator::with_defaults();
        assert!(!generator.templates().event_captions.is_empty());
    }

    #[test]
    fn test_fill_event_template() {
        let generator = CommentaryGenerator::with_defaults();
        let event = make_betrayal_event();

        let template = "{primary_name} betrays {secondary_name} at {location}";
        let filled = generator.fill_event_template(template, &event);

        assert_eq!(
            filled,
            "Mira of Thornwood betrays Voss the Quiet at eastern_bridge"
        );
    }

    #[test]
    fn test_fill_template_missing_secondary() {
        let generator = CommentaryGenerator::with_defaults();
        let event = make_movement_event();

        let template = "{primary_name} meets {secondary_name}";
        let filled = generator.fill_event_template(template, &event);

        // Secondary should be empty string
        assert_eq!(filled, "Mira meets ");
    }

    #[test]
    fn test_caption_event_high_drama() {
        let mut generator = CommentaryGenerator::with_defaults();
        generator.set_current_tick(1000);

        let event = make_betrayal_event();
        let caption = generator.caption_event(&event, test_timestamp());

        assert!(caption.is_some());
        let caption = caption.unwrap();
        assert_eq!(caption.commentary_type, CommentaryType::EventCaption);
        assert!(caption.priority > 0.0);
    }

    #[test]
    fn test_caption_event_low_drama_filtered() {
        let mut generator = CommentaryGenerator::with_defaults();
        generator.set_current_tick(1000);

        let event = make_movement_event(); // Low drama score
        let caption = generator.caption_event(&event, test_timestamp());

        assert!(caption.is_none()); // Should be filtered out
    }

    #[test]
    fn test_generate_irony() {
        let mut generator = CommentaryGenerator::with_defaults();
        generator.set_current_tick(1000);

        let situation = IronySituation {
            situation_type: "unaware_of_betrayal".to_string(),
            unaware_agent_name: "Corin".to_string(),
            unaware_agent_id: "agent_corin".to_string(),
            betrayer_name: Some("Mira".to_string()),
            betrayer_id: Some("agent_mira".to_string()),
            secret_info: "the secret meeting".to_string(),
            betrayal_location: Some("eastern_bridge".to_string()),
            betrayal_event_id: Some("evt_00001".to_string()),
        };

        let irony = generator.generate_irony(&situation, test_timestamp());

        assert!(irony.is_some());
        let irony = irony.unwrap();
        assert_eq!(irony.commentary_type, CommentaryType::DramaticIrony);
        assert!(irony.content.contains("Corin") || irony.content.contains("Mira"));
    }

    #[test]
    fn test_generate_teaser() {
        let mut generator = CommentaryGenerator::with_defaults();
        generator.set_current_tick(1000);

        let tension = make_tension();
        let teaser = generator.generate_teaser(&tension, test_timestamp());

        assert!(teaser.is_some());
        let teaser = teaser.unwrap();
        assert_eq!(teaser.commentary_type, CommentaryType::TensionTeaser);
        assert!(teaser.related_tension.is_some());
    }

    #[test]
    fn test_generate_teaser_low_severity_filtered() {
        let mut generator = CommentaryGenerator::with_defaults();
        generator.set_current_tick(1000);

        let mut tension = make_tension();
        tension.severity = 0.1; // Below threshold

        let teaser = generator.generate_teaser(&tension, test_timestamp());
        assert!(teaser.is_none());
    }

    #[test]
    fn test_commentary_id_generation() {
        let mut generator = CommentaryGenerator::with_defaults();
        generator.set_current_tick(1000);

        let event = make_betrayal_event();
        let caption1 = generator.caption_event(&event, test_timestamp());
        let caption2 = generator.caption_event(&event, test_timestamp());

        assert!(caption1.is_some());
        assert!(caption2.is_some());
        assert_ne!(caption1.unwrap().item_id, caption2.unwrap().item_id);
    }

    #[test]
    fn test_calculate_duration() {
        let generator = CommentaryGenerator::new(
            default_templates(),
            CommentaryConfig {
                base_display_duration_ticks: 100,
                ticks_per_character: 2.0,
                ..CommentaryConfig::default()
            },
        );

        let short = "Hi";
        let long = "This is a much longer piece of commentary text";

        let short_duration = generator.calculate_duration(short);
        let long_duration = generator.calculate_duration(long);

        assert!(long_duration > short_duration);
        assert_eq!(short_duration, 100 + 2 * 2); // base + len * rate
    }

    #[test]
    fn test_irony_disabled_in_config() {
        let mut generator = CommentaryGenerator::new(
            default_templates(),
            CommentaryConfig {
                enable_dramatic_irony: false,
                ..CommentaryConfig::default()
            },
        );
        generator.set_current_tick(1000);

        let situation = IronySituation {
            situation_type: "unaware_of_betrayal".to_string(),
            unaware_agent_name: "Corin".to_string(),
            unaware_agent_id: "agent_corin".to_string(),
            betrayer_name: Some("Mira".to_string()),
            betrayer_id: Some("agent_mira".to_string()),
            secret_info: "secret".to_string(),
            betrayal_location: None,
            betrayal_event_id: None,
        };

        let irony = generator.generate_irony(&situation, test_timestamp());
        assert!(irony.is_none());
    }

    #[test]
    fn test_event_type_to_string() {
        assert_eq!(event_type_to_string(&EventType::Betrayal), "betrayal");
        assert_eq!(event_type_to_string(&EventType::Movement), "movement");
        assert_eq!(event_type_to_string(&EventType::Death), "death");
    }

    #[test]
    fn test_event_subtype_to_string() {
        assert_eq!(
            event_subtype_to_string(&EventSubtype::Betrayal(BetrayalSubtype::Defection)),
            "defection"
        );
        assert_eq!(
            event_subtype_to_string(&EventSubtype::Movement(MovementSubtype::Travel)),
            "travel"
        );
    }

    #[test]
    fn test_get_event_templates() {
        let templates = default_templates();

        // Should find specific template
        let specific = templates.get_event_templates("betrayal", "defection");
        assert!(specific.is_some());

        // Should not find non-existent
        let missing = templates.get_event_templates("betrayal", "nonexistent");
        assert!(missing.is_none());
    }

    // ============ IronyDetector Tests ============

    fn make_betrayal_event_with_affected() -> Event {
        let primary = ActorSnapshot::new(
            "agent_mira",
            "Mira of Thornwood",
            "thornwood",
            "scout",
            "eastern_bridge",
        );
        let secondary = ActorSnapshot::new(
            "agent_voss",
            "Voss the Quiet",
            "ironmere",
            "spymaster",
            "eastern_bridge",
        );
        let affected = AffectedActor::new(
            "agent_corin",
            "Corin",
            "thornwood",
            "leader",
        );

        let mut actors = ActorSet::with_secondary(primary, secondary);
        actors.affected.push(affected);

        Event {
            event_id: "evt_00001".to_string(),
            timestamp: test_timestamp(),
            event_type: EventType::Betrayal,
            subtype: EventSubtype::Betrayal(BetrayalSubtype::SecretSharedWithEnemy),
            actors,
            context: EventContext::new("trust_eroded"),
            outcome: EventOutcome::General(GeneralOutcome::default()),
            drama_tags: vec!["betrayal".to_string()],
            drama_score: 0.85,
            connected_events: vec![],
        }
    }

    fn make_world_snapshot_with_trust(
        affected_id: &str,
        betrayer_id: &str,
        trust_level: f32,
    ) -> WorldSnapshot {
        use sim_events::{RelationshipSnapshot, AgentSnapshot as SnapshotAgent};

        let ts = test_timestamp();
        let mut snapshot = WorldSnapshot::new("snap_000001", ts, "test");

        // Add agents
        snapshot.agents.push(SnapshotAgent::new(
            affected_id,
            "Corin",
            "thornwood",
            "leader",
            "thornwood_hall",
        ));
        snapshot.agents.push(SnapshotAgent::new(
            betrayer_id,
            "Mira",
            "thornwood",
            "scout",
            "eastern_bridge",
        ));

        // Add relationship from affected -> betrayer
        let mut affected_relationships = HashMap::new();
        affected_relationships.insert(
            betrayer_id.to_string(),
            RelationshipSnapshot::new(trust_level, 0.5, 0.5),
        );
        snapshot.relationships.insert(affected_id.to_string(), affected_relationships);

        snapshot
    }

    #[test]
    fn test_irony_detector_creation() {
        let detector = IronyDetector::new();
        assert_eq!(detector.betrayal_count(), 0);
    }

    #[test]
    fn test_irony_detector_record_betrayal() {
        let mut detector = IronyDetector::new();
        let event = make_betrayal_event_with_affected();

        detector.record_betrayal(&event);
        assert_eq!(detector.betrayal_count(), 1);

        let records = detector.betrayals();
        assert_eq!(records[0].betrayer_id, "agent_mira");
        assert!(records[0].affected_ids.contains(&"agent_corin".to_string()));
    }

    #[test]
    fn test_irony_detector_ignores_non_betrayal() {
        let mut detector = IronyDetector::new();
        let event = make_movement_event();

        detector.record_betrayal(&event);
        assert_eq!(detector.betrayal_count(), 0);
    }

    #[test]
    fn test_betrayal_creates_irony_situation() {
        let mut detector = IronyDetector::new();
        let event = make_betrayal_event_with_affected();
        detector.record_betrayal(&event);

        // Create world state where Corin still trusts Mira
        let state = make_world_snapshot_with_trust("agent_corin", "agent_mira", 0.8);

        let situations = detector.detect_irony(&state);
        assert_eq!(situations.len(), 1);
        assert_eq!(situations[0].situation_type, "unaware_of_betrayal");
        assert_eq!(situations[0].unaware_agent_id, "agent_corin");
        assert_eq!(situations[0].betrayer_id, Some("agent_mira".to_string()));
    }

    #[test]
    fn test_irony_clears_when_trust_drops() {
        let mut detector = IronyDetector::new();
        let event = make_betrayal_event_with_affected();
        detector.record_betrayal(&event);

        // Create world state where Corin no longer trusts Mira
        let state = make_world_snapshot_with_trust("agent_corin", "agent_mira", 0.3);

        let situations = detector.detect_irony(&state);
        assert!(situations.is_empty()); // No irony - trust is low
    }

    #[test]
    fn test_mark_discovered_removes_irony() {
        let mut detector = IronyDetector::new();
        let event = make_betrayal_event_with_affected();
        detector.record_betrayal(&event);

        // Mark as discovered by Corin
        detector.mark_discovered("evt_00001", "agent_corin");

        // Even with high trust, no irony because it's discovered
        let state = make_world_snapshot_with_trust("agent_corin", "agent_mira", 0.9);
        let situations = detector.detect_irony(&state);
        assert!(situations.is_empty());
    }

    #[test]
    fn test_multiple_affected_agents() {
        let mut detector = IronyDetector::new();

        // Create event with multiple affected agents
        let primary = ActorSnapshot::new(
            "agent_mira",
            "Mira",
            "thornwood",
            "scout",
            "eastern_bridge",
        );
        let secondary = ActorSnapshot::new(
            "agent_voss",
            "Voss",
            "ironmere",
            "spymaster",
            "eastern_bridge",
        );

        let mut actors = ActorSet::with_secondary(primary, secondary);
        actors.affected.push(AffectedActor::new(
            "agent_corin", "Corin", "thornwood", "leader"
        ));
        actors.affected.push(AffectedActor::new(
            "agent_elena", "Elena", "thornwood", "scout"
        ));

        let event = Event {
            event_id: "evt_00002".to_string(),
            timestamp: test_timestamp(),
            event_type: EventType::Betrayal,
            subtype: EventSubtype::Betrayal(BetrayalSubtype::SecretSharedWithEnemy),
            actors,
            context: EventContext::new("trust_eroded"),
            outcome: EventOutcome::General(GeneralOutcome::default()),
            drama_tags: vec![],
            drama_score: 0.8,
            connected_events: vec![],
        };

        detector.record_betrayal(&event);

        // Both agents trust Mira
        let ts = test_timestamp();
        let mut state = WorldSnapshot::new("snap_001", ts, "test");

        use sim_events::{RelationshipSnapshot, AgentSnapshot as SnapshotAgent};

        state.agents.push(SnapshotAgent::new("agent_corin", "Corin", "thornwood", "leader", "hall"));
        state.agents.push(SnapshotAgent::new("agent_elena", "Elena", "thornwood", "scout", "market"));
        state.agents.push(SnapshotAgent::new("agent_mira", "Mira", "thornwood", "scout", "bridge"));

        let mut corin_rels = HashMap::new();
        corin_rels.insert("agent_mira".to_string(), RelationshipSnapshot::new(0.8, 0.5, 0.5));
        state.relationships.insert("agent_corin".to_string(), corin_rels);

        let mut elena_rels = HashMap::new();
        elena_rels.insert("agent_mira".to_string(), RelationshipSnapshot::new(0.7, 0.5, 0.5));
        state.relationships.insert("agent_elena".to_string(), elena_rels);

        let situations = detector.detect_irony(&state);
        assert_eq!(situations.len(), 2); // Both have irony situations
    }

    #[test]
    fn test_cleanup_old_betrayals() {
        let mut detector = IronyDetector::new();
        let event = make_betrayal_event_with_affected();
        detector.record_betrayal(&event);

        assert_eq!(detector.betrayal_count(), 1);

        // Cleanup with max age that would expire our betrayal (tick 1000)
        detector.cleanup(2000, 500); // Current tick 2000, max age 500
        assert_eq!(detector.betrayal_count(), 0);
    }

    #[test]
    fn test_cleanup_fully_discovered() {
        let mut detector = IronyDetector::new();
        let event = make_betrayal_event_with_affected();
        detector.record_betrayal(&event);

        // Mark as discovered
        detector.mark_discovered("evt_00001", "agent_corin");

        // Cleanup should remove fully discovered betrayals
        detector.cleanup(1001, 100000); // Recent enough, but fully discovered
        assert_eq!(detector.betrayal_count(), 0);
    }

    #[test]
    fn test_betrayal_record_is_discovered_by() {
        let event = make_betrayal_event_with_affected();
        let mut record = BetrayalRecord::from_event(&event).unwrap();

        assert!(!record.is_discovered_by("agent_corin"));

        record.discovered_by.insert("agent_corin".to_string());
        assert!(record.is_discovered_by("agent_corin"));
    }

    #[test]
    fn test_betrayal_record_is_fully_discovered() {
        let event = make_betrayal_event_with_affected();
        let mut record = BetrayalRecord::from_event(&event).unwrap();

        assert!(!record.is_fully_discovered());

        // Discover by all affected agents
        for id in record.affected_ids.clone() {
            record.discovered_by.insert(id);
        }

        assert!(record.is_fully_discovered());
    }

    #[test]
    fn test_irony_situation_constructor() {
        let situation = IronySituation::unaware_of_betrayal(
            "agent_corin",
            "Corin",
            "agent_mira",
            "Mira",
            "evt_00001",
            Some("eastern_bridge".to_string()),
        );

        assert_eq!(situation.situation_type, "unaware_of_betrayal");
        assert_eq!(situation.unaware_agent_id, "agent_corin");
        assert_eq!(situation.unaware_agent_name, "Corin");
        assert_eq!(situation.betrayer_id, Some("agent_mira".to_string()));
        assert_eq!(situation.betrayer_name, Some("Mira".to_string()));
        assert_eq!(situation.betrayal_event_id, Some("evt_00001".to_string()));
    }
}
