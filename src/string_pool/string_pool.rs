//! Interns strings and assigns stable numeric identifiers.
//!
//! A `StringPool` stores each distinct string once and returns a compact
//! `StrId` that can be copied, compared, and stored cheaply.
//!
//! # Properties
//! - Each unique string is stored exactly once.
//! - `StrId` is stable for the lifetime of the pool.
//! - Equality and hashing are O(1) on `StrId`.
//!
//! # Typical usage
//! ```ignore
//! let mut pool = StringPool::new();
//! let goblin = pool.intern("Goblin");
//! let orc = pool.intern("Orc");
//!
//! assert_eq!(pool.resolve(goblin), Some("Goblin"));
//! ```
//!
//! # Use cases
//! - Game entities (monster names, item types)
//! - AST nodes and symbols
//! - Configuration keys
//! - Logging and telemetry
//!
//! # Notes
//! - The pool owns all strings.
//! - Removing strings is intentionally unsupported.
use core::fmt;
use std::{collections::HashMap, fmt::Formatter};

use crate::string_pool;

/// Identifier for an interned string.
///
/// Cheap to copy and compare. Meaningful only together with the
/// `StringPool` that created it.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct StrId(usize);

impl std::fmt::Debug for StrId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "StrId({})", self.0)
    }
}

struct DebugStrId<'a> {
    id: StrId,
    pool: &'a StringPool,
}

impl<'a> std::fmt::Debug for DebugStrId<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.pool.resolve(self.id).unwrap_or("<invalid>");
        write!(f, "StrId({}:\"{}\")", self.id.0, s)
    }
}

#[derive(Default)]
struct StringPool {
    //stores each string with StrId as index O(1) resolve
    strings: Vec<String>,
    //allows for O(1) intern
    map: HashMap<String, StrId>,
}

struct Iter<'a> {
    data: &'a [String],
    curr: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let s = self.data.get(self.curr).map(|s| s.as_str());
        self.curr += 1;
        s
    }
}

impl StringPool {
    /// Creates an empty string pool.
    pub fn new() -> Self {
        StringPool::default()
    }

    /// Interns a string and returns its `StrId`.
    ///
    /// If the string already exists, returns the existing id.
    pub fn intern(&mut self, s: &str) -> StrId {
        if let Some(strid) = self.map.get(s) {
            *strid
        } else {
            let new_strid = StrId(self.strings.len());
            self.strings.push(s.to_string());
            self.map.insert(s.to_string(), new_strid);
            new_strid
        }
    }

    /// Resolves an id back to its string.
    /// Returns `None` if the id is invalid for this pool.
    pub fn resolve(&self, id: StrId) -> Option<&str> {
        self.strings.get(id.0).map(|s| s.as_str())
    }

    /// Returns the number of interned strings.
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Returns true if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if the pool contains the given string.
    pub fn contains(&self, s: &str) -> bool {
        self.map.contains_key(s)
    }

    /// Returns true if the id was created by this pool.
    pub fn contains_id(&self, id: StrId) -> bool {
        self.strings.len() > id.0
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            data: &self.strings,
            curr: 0,
        }
    }

    /// Returns a debug wrapper that prints `StrId(42:"Goblin")`.
    pub fn dbg<'a>(&'a self, id: StrId) -> DebugStrId<'a> {
        todo!()
    }
}

impl IntoIterator for StringPool {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.strings.into_iter()
    }
}

impl<A: AsRef<str>> FromIterator<A> for StringPool {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut string_pool = StringPool::default();
        for s in iter {
            string_pool.intern(s.as_ref());
        }
        string_pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_string_pool() {
        let string_pool = StringPool::default();
        assert!(string_pool.is_empty());
        assert_eq!(string_pool.len(), 0);
    }

    #[test]
    fn test_intern() {
        let mut string_pool = StringPool::default();
        let goblin_id = string_pool.intern("Goblin");
        let goblin_id2 = string_pool.intern("Goblin");
        // make sure that they are assigned same ID
        assert_eq!(goblin_id, goblin_id2);
        assert!(!string_pool.is_empty());
        assert_eq!(string_pool.len(), 1);

        let orc_id = string_pool.intern("Orc");
        assert_eq!(string_pool.len(), 2);
        assert_ne!(goblin_id, orc_id);

        for _ in 0..10 {
            let orc_id_i = string_pool.intern("Orc");
            assert_eq!(string_pool.len(), 2);
            assert_eq!(orc_id_i, orc_id);
        }
    }
    #[test]
    fn test_resolve() {
        let mut string_pool = StringPool::default();
        let goblin_id = string_pool.intern("Goblin");
        let _ = string_pool.intern("Goblin");
        let orc_id = string_pool.intern("Orc");
        assert_eq!(string_pool.resolve(goblin_id), Some("Goblin"));
        assert_eq!(string_pool.resolve(orc_id), Some("Orc"));
        assert_eq!(string_pool.resolve(StrId(2)), None);
    }

    #[test]
    fn test_contains() {
        let mut string_pool = StringPool::default();
        let goblin_id = string_pool.intern("Goblin");
        let goblin_id2 = string_pool.intern("Goblin");
        let orc_id = string_pool.intern("Orc");

        assert!(string_pool.contains("Goblin"));
        assert!(!string_pool.contains("goblin"));
        assert!(string_pool.contains_id(goblin_id));
        assert!(string_pool.contains_id(goblin_id2));
        assert!(string_pool.contains_id(orc_id));
        assert!(!string_pool.contains_id(StrId(2)));
        assert!(string_pool.contains_id(StrId(1)));
    }

    #[test]
    fn test_iter() {
        let mut pool = StringPool::new();
        let _ = pool.intern("Goblin");
        let _ = pool.intern("Orc");
        let _ = pool.intern("Troll");

        let collected: Vec<_> = pool.iter().collect();
        assert_eq!(collected, vec!["Goblin", "Orc", "Troll"]);
    }

    #[test]
    fn test_into_iter() {
        let mut pool = StringPool::new();
        pool.intern("Goblin");
        pool.intern("Orc");

        let collected: Vec<_> = pool.into_iter().collect();
        assert_eq!(collected, vec!["Goblin".to_string(), "Orc".to_string()]);
    }

    #[test]
    fn test_from_iter() {
        let names = vec!["Goblin", "Orc", "Troll"];
        let pool: StringPool = names.clone().into_iter().collect();

        assert_eq!(pool.len(), names.len());

        for name in names {
            assert!(pool.contains(name));
        }
    }
}
