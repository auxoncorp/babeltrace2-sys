use std::collections::BTreeMap;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum EnvValue {
    Integer(i64),
    String(String),
}

/// Trace environment key-value store
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Env {
    pub(crate) entries: BTreeMap<String, EnvValue>,
}

impl Env {
    pub fn entries(&self) -> &BTreeMap<String, EnvValue> {
        &self.entries
    }
}
