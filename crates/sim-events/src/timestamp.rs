//! Simulation Timestamp Types
//!
//! Handles simulation time with both tick-based and human-readable date formats.
//!
//! # Example
//!
//! ```
//! use sim_events::{SimTimestamp, SimDate, Season};
//!
//! let ts = SimTimestamp::new(100, 1, Season::Spring, 15);
//! assert_eq!(ts.tick, 100);
//! assert_eq!(ts.date.to_string(), "year_1.spring.day_15");
//! ```

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

/// Number of days in each season.
pub const DAYS_PER_SEASON: u8 = 30;

/// Number of ticks per simulated day.
pub const TICKS_PER_DAY: u64 = 100;

/// Season of the year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// Returns the next season in order.
    pub fn next(self) -> Self {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }

    /// Returns true if this is the last season of the year.
    pub fn is_year_end(self) -> bool {
        matches!(self, Season::Winter)
    }
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Season::Spring => write!(f, "spring"),
            Season::Summer => write!(f, "summer"),
            Season::Autumn => write!(f, "autumn"),
            Season::Winter => write!(f, "winter"),
        }
    }
}

impl FromStr for Season {
    type Err = ParseDateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "spring" => Ok(Season::Spring),
            "summer" => Ok(Season::Summer),
            "autumn" => Ok(Season::Autumn),
            "winter" => Ok(Season::Winter),
            _ => Err(ParseDateError::InvalidSeason(s.to_string())),
        }
    }
}

/// Human-readable simulation date.
///
/// Serializes to strings like "year_3.winter.day_12".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimDate {
    pub year: u32,
    pub season: Season,
    pub day: u8,
}

impl SimDate {
    /// Creates a new SimDate.
    pub fn new(year: u32, season: Season, day: u8) -> Self {
        Self { year, season, day }
    }

    /// Creates a SimDate for the start of the simulation.
    pub fn start() -> Self {
        Self {
            year: 1,
            season: Season::Spring,
            day: 1,
        }
    }

    /// Advances the date by one day, handling season and year rollovers.
    pub fn advance_day(&mut self) {
        self.day += 1;
        if self.day > DAYS_PER_SEASON {
            self.day = 1;
            let was_year_end = self.season.is_year_end();
            self.season = self.season.next();
            if was_year_end {
                self.year += 1;
            }
        }
    }
}

impl fmt::Display for SimDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "year_{}.{}.day_{}", self.year, self.season, self.day)
    }
}

/// Error type for parsing SimDate from strings.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseDateError {
    InvalidFormat(String),
    InvalidYear(String),
    InvalidSeason(String),
    InvalidDay(String),
}

impl fmt::Display for ParseDateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseDateError::InvalidFormat(s) => {
                write!(f, "invalid date format: '{}', expected 'year_N.season.day_M'", s)
            }
            ParseDateError::InvalidYear(s) => write!(f, "invalid year: '{}'", s),
            ParseDateError::InvalidSeason(s) => write!(f, "invalid season: '{}'", s),
            ParseDateError::InvalidDay(s) => write!(f, "invalid day: '{}'", s),
        }
    }
}

impl std::error::Error for ParseDateError {}

impl FromStr for SimDate {
    type Err = ParseDateError;

    /// Parses a SimDate from a string like "year_3.winter.day_12".
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(ParseDateError::InvalidFormat(s.to_string()));
        }

        // Parse year from "year_N"
        let year_part = parts[0];
        let year = year_part
            .strip_prefix("year_")
            .ok_or_else(|| ParseDateError::InvalidFormat(s.to_string()))?
            .parse::<u32>()
            .map_err(|_| ParseDateError::InvalidYear(year_part.to_string()))?;

        // Parse season
        let season = parts[1].parse::<Season>()?;

        // Parse day from "day_M"
        let day_part = parts[2];
        let day = day_part
            .strip_prefix("day_")
            .ok_or_else(|| ParseDateError::InvalidFormat(s.to_string()))?
            .parse::<u8>()
            .map_err(|_| ParseDateError::InvalidDay(day_part.to_string()))?;

        Ok(SimDate { year, season, day })
    }
}

// Custom serialization for SimDate - serialize as a string
impl Serialize for SimDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SimDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// A point in simulation time.
///
/// Contains both a monotonic tick counter and a human-readable date.
///
/// # Example
///
/// ```
/// use sim_events::{SimTimestamp, Season};
///
/// let ts = SimTimestamp::new(100, 1, Season::Spring, 15);
/// assert_eq!(ts.tick, 100);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimTimestamp {
    /// Monotonically increasing simulation tick.
    pub tick: u64,
    /// Human-readable date.
    pub date: SimDate,
}

impl SimTimestamp {
    /// Creates a new SimTimestamp.
    pub fn new(tick: u64, year: u32, season: Season, day: u8) -> Self {
        Self {
            tick,
            date: SimDate::new(year, season, day),
        }
    }

    /// Creates a timestamp for the start of the simulation.
    pub fn start() -> Self {
        Self {
            tick: 0,
            date: SimDate::start(),
        }
    }

    /// Creates a SimTimestamp from a tick and existing SimDate.
    pub fn from_date(tick: u64, date: SimDate) -> Self {
        Self { tick, date }
    }

    /// Increments the tick counter by one.
    pub fn advance_tick(&mut self) {
        self.tick += 1;
    }

    /// Advances the date by one day.
    ///
    /// Also increments the tick by TICKS_PER_DAY.
    pub fn advance_day(&mut self) {
        self.tick += TICKS_PER_DAY;
        self.date.advance_day();
    }

    /// Returns the current year.
    pub fn year(&self) -> u32 {
        self.date.year
    }

    /// Returns the current season.
    pub fn season(&self) -> Season {
        self.date.season
    }

    /// Returns the current day.
    pub fn day(&self) -> u8 {
        self.date.day
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_season_display() {
        assert_eq!(Season::Spring.to_string(), "spring");
        assert_eq!(Season::Summer.to_string(), "summer");
        assert_eq!(Season::Autumn.to_string(), "autumn");
        assert_eq!(Season::Winter.to_string(), "winter");
    }

    #[test]
    fn test_season_parse() {
        assert_eq!("spring".parse::<Season>().unwrap(), Season::Spring);
        assert_eq!("Summer".parse::<Season>().unwrap(), Season::Summer);
        assert_eq!("AUTUMN".parse::<Season>().unwrap(), Season::Autumn);
        assert_eq!("winter".parse::<Season>().unwrap(), Season::Winter);
    }

    #[test]
    fn test_season_next() {
        assert_eq!(Season::Spring.next(), Season::Summer);
        assert_eq!(Season::Summer.next(), Season::Autumn);
        assert_eq!(Season::Autumn.next(), Season::Winter);
        assert_eq!(Season::Winter.next(), Season::Spring);
    }

    #[test]
    fn test_sim_date_display() {
        let date = SimDate::new(3, Season::Winter, 12);
        assert_eq!(date.to_string(), "year_3.winter.day_12");
    }

    #[test]
    fn test_sim_date_parse() {
        let date: SimDate = "year_3.winter.day_12".parse().unwrap();
        assert_eq!(date.year, 3);
        assert_eq!(date.season, Season::Winter);
        assert_eq!(date.day, 12);
    }

    #[test]
    fn test_sim_date_roundtrip() {
        let original = SimDate::new(5, Season::Summer, 15);
        let string = original.to_string();
        let parsed: SimDate = string.parse().unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_sim_date_advance_day() {
        let mut date = SimDate::new(1, Season::Spring, 1);
        date.advance_day();
        assert_eq!(date.day, 2);
        assert_eq!(date.season, Season::Spring);
        assert_eq!(date.year, 1);
    }

    #[test]
    fn test_sim_date_season_rollover() {
        // Day 30 of winter -> Day 1 of spring, new year
        let mut date = SimDate::new(1, Season::Winter, 30);
        date.advance_day();
        assert_eq!(date.day, 1);
        assert_eq!(date.season, Season::Spring);
        assert_eq!(date.year, 2);
    }

    #[test]
    fn test_sim_date_season_rollover_mid_year() {
        // Day 30 of spring -> Day 1 of summer, same year
        let mut date = SimDate::new(1, Season::Spring, 30);
        date.advance_day();
        assert_eq!(date.day, 1);
        assert_eq!(date.season, Season::Summer);
        assert_eq!(date.year, 1);
    }

    #[test]
    fn test_sim_date_year_rollover() {
        // Day 30 of winter year 1 -> Day 1 of spring year 2
        let mut date = SimDate::new(1, Season::Winter, 30);
        date.advance_day();
        assert_eq!(date.year, 2);
        assert_eq!(date.season, Season::Spring);
        assert_eq!(date.day, 1);
    }

    #[test]
    fn test_sim_timestamp_new() {
        let ts = SimTimestamp::new(100, 1, Season::Spring, 15);
        assert_eq!(ts.tick, 100);
        assert_eq!(ts.date.year, 1);
        assert_eq!(ts.date.season, Season::Spring);
        assert_eq!(ts.date.day, 15);
    }

    #[test]
    fn test_sim_timestamp_start() {
        let ts = SimTimestamp::start();
        assert_eq!(ts.tick, 0);
        assert_eq!(ts.date.year, 1);
        assert_eq!(ts.date.season, Season::Spring);
        assert_eq!(ts.date.day, 1);
    }

    #[test]
    fn test_sim_timestamp_advance_tick() {
        let mut ts = SimTimestamp::start();
        ts.advance_tick();
        assert_eq!(ts.tick, 1);
    }

    #[test]
    fn test_sim_timestamp_advance_day() {
        let mut ts = SimTimestamp::start();
        ts.advance_day();
        assert_eq!(ts.tick, TICKS_PER_DAY);
        assert_eq!(ts.date.day, 2);
    }

    #[test]
    fn test_sim_timestamp_serialization() {
        let ts = SimTimestamp::new(84729, 3, Season::Winter, 12);
        let json = serde_json::to_string(&ts).unwrap();
        assert_eq!(json, r#"{"tick":84729,"date":"year_3.winter.day_12"}"#);
    }

    #[test]
    fn test_sim_timestamp_deserialization() {
        let json = r#"{"tick":84729,"date":"year_3.winter.day_12"}"#;
        let ts: SimTimestamp = serde_json::from_str(json).unwrap();
        assert_eq!(ts.tick, 84729);
        assert_eq!(ts.date.year, 3);
        assert_eq!(ts.date.season, Season::Winter);
        assert_eq!(ts.date.day, 12);
    }

    #[test]
    fn test_sim_timestamp_roundtrip() {
        let original = SimTimestamp::new(84729, 3, Season::Winter, 12);
        let json = serde_json::to_string(&original).unwrap();
        let parsed: SimTimestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_sim_date_serialize_as_string() {
        let date = SimDate::new(3, Season::Winter, 12);
        let json = serde_json::to_string(&date).unwrap();
        // Should serialize as a plain string, not an object
        assert_eq!(json, r#""year_3.winter.day_12""#);
    }

    #[test]
    fn test_sim_date_deserialize_from_string() {
        let json = r#""year_3.winter.day_12""#;
        let date: SimDate = serde_json::from_str(json).unwrap();
        assert_eq!(date.year, 3);
        assert_eq!(date.season, Season::Winter);
        assert_eq!(date.day, 12);
    }

    #[test]
    fn test_season_serialization() {
        assert_eq!(serde_json::to_string(&Season::Spring).unwrap(), r#""spring""#);
        assert_eq!(serde_json::to_string(&Season::Summer).unwrap(), r#""summer""#);
        assert_eq!(serde_json::to_string(&Season::Autumn).unwrap(), r#""autumn""#);
        assert_eq!(serde_json::to_string(&Season::Winter).unwrap(), r#""winter""#);
    }

    #[test]
    fn test_parse_date_error() {
        assert!("invalid".parse::<SimDate>().is_err());
        assert!("year_one.spring.day_1".parse::<SimDate>().is_err());
        assert!("year_1.invalid.day_1".parse::<SimDate>().is_err());
        assert!("year_1.spring.day_one".parse::<SimDate>().is_err());
    }

    #[test]
    fn test_full_year_cycle() {
        let mut date = SimDate::new(1, Season::Spring, 1);

        // Advance through a full year (4 seasons * 30 days = 120 days)
        for _ in 0..120 {
            date.advance_day();
        }

        assert_eq!(date.year, 2);
        assert_eq!(date.season, Season::Spring);
        assert_eq!(date.day, 1);
    }
}
