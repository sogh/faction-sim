//! Core simulation logic: agents, trust, memory, factions.

pub mod agent;
pub mod trust;
pub mod memory;
pub mod faction;
pub mod world;

pub use agent::Agent;
pub use faction::Faction;
pub use world::World;
