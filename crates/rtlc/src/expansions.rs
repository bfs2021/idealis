use crate::{CoreError, SafetyProgram, SafetyResult, Value};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Successor {
    pub action: u32,
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Expansion {
    successors: Vec<Successor>,
    digest: String,
    #[serde(skip)]
    verified: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SafetyExpansionCache {
    format: String,
    version: u32,
    build_digest: String,
    transition_identity: String,
    entries: BTreeMap<String, Expansion>,
    #[serde(skip)]
    capacity: usize,
    #[serde(skip)]
    requested_build: String,
    #[serde(skip)]
    pub(crate) stats: ExpansionStats,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpansionStats {
    pub hits: u32,
    pub misses: u32,
    pub writes: u32,
    pub invalidated: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ExpansionOptions<'a> {
    pub enabled: bool,
    pub build_digest: &'a str,
    pub max_entries: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IncrementalSafetyResult {
    pub result: SafetyResult,
    pub stats: ExpansionStats,
}

impl Default for SafetyExpansionCache {
    fn default() -> Self {
        Self {
            format: "idealis.safety-expansions".into(),
            version: 1,
            build_digest: String::new(),
            transition_identity: String::new(),
            entries: BTreeMap::new(),
            capacity: 0,
            requested_build: String::new(),
            stats: ExpansionStats::default(),
        }
    }
}

fn encode(value: &impl Serialize) -> Result<String, CoreError> {
    serde_json::to_string(value).map_err(|error| CoreError::protocol(&error.to_string()))
}

impl SafetyExpansionCache {
    pub(crate) fn prepare(&mut self, build_digest: &str, capacity: u32) -> Result<(), CoreError> {
        self.validate_header()?;
        if build_digest.len() != 64
            || !build_digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(CoreError::protocol("Expected expansion backend SHA-256"));
        }
        self.requested_build = build_digest.into();
        self.capacity = capacity as usize;
        self.stats = ExpansionStats::default();
        Ok(())
    }

    pub(crate) fn bind_validated(&mut self, program: &SafetyProgram) -> Result<(), CoreError> {
        let identity = encode(&(
            program.version,
            &program.variables,
            &program.definitions,
            &program.next,
        ))?;
        let invalidated =
            self.build_digest != self.requested_build || self.transition_identity != identity;
        self.stats = ExpansionStats {
            invalidated: invalidated && !self.entries.is_empty(),
            ..ExpansionStats::default()
        };
        if invalidated {
            self.entries.clear();
            self.build_digest.clone_from(&self.requested_build);
            self.transition_identity = identity;
        }
        while self.entries.len() > self.capacity {
            self.entries.pop_last();
        }
        Ok(())
    }

    fn validate_header(&self) -> Result<(), CoreError> {
        if self.format != "idealis.safety-expansions" || self.version != 1 {
            return Err(CoreError::protocol("Unsupported expansion cache"));
        }
        Ok(())
    }

    fn digest(&self, key: &str, successors: &[Successor]) -> Result<String, CoreError> {
        Ok(format!(
            "{:x}",
            Sha256::digest(
                encode(&(
                    &self.build_digest,
                    &self.transition_identity,
                    key,
                    successors
                ))?
                .as_bytes()
            )
        ))
    }

    pub(crate) fn key(values: &[Value]) -> Result<String, CoreError> {
        encode(&values)
    }

    pub(crate) fn lookup(
        &mut self,
        key: &str,
        width: usize,
        actions: usize,
    ) -> Result<Option<&[Successor]>, CoreError> {
        if let Some(entry) = self.entries.get(key) {
            if !entry.verified {
                if entry.digest != self.digest(key, &entry.successors)? {
                    return Err(CoreError::protocol("Expansion digest mismatch"));
                }
                for successor in &entry.successors {
                    if successor.action as usize >= actions || successor.values.len() != width {
                        return Err(CoreError::protocol("Invalid cached successor"));
                    }
                    for value in &successor.values {
                        if value.clone().normalize()? != *value {
                            return Err(CoreError::protocol("Noncanonical cached successor"));
                        }
                    }
                }
                self.entries.get_mut(key).unwrap().verified = true;
            }
            self.stats.hits += 1;
            Ok(Some(&self.entries[key].successors))
        } else {
            self.stats.misses += 1;
            Ok(None)
        }
    }

    pub(crate) fn can_insert(&self) -> bool {
        self.entries.len() < self.capacity
    }

    pub(crate) fn insert(
        &mut self,
        key: String,
        successors: Vec<Successor>,
    ) -> Result<(), CoreError> {
        if self.entries.len() < self.capacity {
            let digest = self.digest(&key, &successors)?;
            self.entries.insert(
                key,
                Expansion {
                    successors,
                    digest,
                    verified: true,
                },
            );
            self.stats.writes += 1;
        }
        Ok(())
    }

    pub fn merge(&mut self, other: Self, capacity: u32) -> Result<(), CoreError> {
        self.validate_header()?;
        other.validate_header()?;
        if self.build_digest != other.build_digest
            || self.transition_identity != other.transition_identity
        {
            return Err(CoreError::protocol("Different expansion namespaces"));
        }
        for (key, entry) in self.entries.iter().chain(&other.entries) {
            if entry.digest != self.digest(key, &entry.successors)? {
                return Err(CoreError::protocol("Expansion digest mismatch"));
            }
        }
        for (key, entry) in &other.entries {
            if let Some(stored) = self.entries.get(key)
                && stored.successors != entry.successors
            {
                return Err(CoreError::protocol("Conflicting complete expansions"));
            }
        }
        self.entries.extend(other.entries);
        self.capacity = capacity as usize;
        while self.entries.len() > self.capacity {
            self.entries.pop_last();
        }
        Ok(())
    }
}
