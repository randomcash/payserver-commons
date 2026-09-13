//! A single manifest dependency entry: `name:requirement`.

use std::fmt;
use std::str::FromStr;

use semver::{Version, VersionReq};
use serde::{Deserialize, Deserializer};

/// One entry of a manifest's `dependencies` list, e.g. `ethpayserver:^1.2.0`.
///
/// The host presents itself as a dependency under the name `ethpayserver`,
/// so the same type and the same matching rule cover a plugin declaring what
/// host it needs and a plugin declaring what other plugin it needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub requirement: VersionReq,
}

impl Dependency {
    /// Whether `version` satisfies this dependency's requirement.
    pub fn is_satisfied_by(&self, version: &Version) -> bool {
        self.requirement.matches(version)
    }
}

impl fmt::Display for Dependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.name, self.requirement)
    }
}

impl FromStr for Dependency {
    type Err = InvalidDependency;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, requirement) = s
            .split_once(':')
            .ok_or_else(|| InvalidDependency::Malformed(s.to_string()))?;
        if name.is_empty() {
            return Err(InvalidDependency::Malformed(s.to_string()));
        }
        let requirement = VersionReq::parse(requirement)
            .map_err(|e| InvalidDependency::BadRequirement(s.to_string(), e.to_string()))?;
        Ok(Self {
            name: name.to_string(),
            requirement,
        })
    }
}

impl<'de> Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
}

/// A `dependencies` entry that failed to parse.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidDependency {
    #[error("malformed dependency {0:?}, expected \"name:requirement\"")]
    Malformed(String),
    #[error("dependency {0:?} has an invalid version requirement: {1}")]
    BadRequirement(String, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_host_dependency() {
        let dep: Dependency = "ethpayserver:^1.2.0".parse().unwrap();
        assert_eq!(dep.name, "ethpayserver");
        assert!(dep.is_satisfied_by(&Version::parse("1.2.5").unwrap()));
        assert!(!dep.is_satisfied_by(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn rejects_missing_colon() {
        assert!("ethpayserver^1.2.0".parse::<Dependency>().is_err());
    }

    #[test]
    fn rejects_bad_requirement() {
        assert!("ethpayserver:not-a-version".parse::<Dependency>().is_err());
    }
}
