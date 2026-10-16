#[cfg(feature = "datom")]
pub mod client;
pub mod generated;
pub mod projection;
pub mod registry;
pub mod roles;
pub mod role_policy;
pub mod service;

pub use registry::{ExpandsSkills, RegistryError, ResolvedSkill, SkillEntry, SkillRegistry};

pub mod startup;
mod startup_metadata;
