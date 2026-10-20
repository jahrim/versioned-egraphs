use indexmap::IndexSet;
use std::sync::{Mutex, OnceLock};

use crate::util::memory::{HasMemory, Memory};

/// A unique identifier, represented as a 32-bit index.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(u32);
impl From<usize> for Id {
    #[inline(always)]
    fn from(n: usize) -> Id {
        Id(n as u32)
    }
}
impl From<Id> for usize {
    #[inline(always)]
    fn from(id: Id) -> usize {
        id.0 as usize
    }
}
impl std::fmt::Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl HasMemory for Id {
    fn memory(&self) -> Memory {
        Memory::of_bytes(std::mem::size_of_val(self))
    }
}
/// A struct that can be interpreted as an index.
pub trait AsIndex {
    /// #### Return
    /// This struct as an index for a [Vec].
    fn to_i(self) -> usize;
}
impl AsIndex for Id {
    #[inline(always)]
    fn to_i(self) -> usize {
        Into::<usize>::into(self)
    }
}

/// A range over [Id]s.
pub struct IdRange {
    current: u32,
    end: u32,
}
impl IdRange {
    #[inline(always)]
    pub fn new(start: usize, end: usize) -> IdRange {
        IdRange { current: start as u32, end: end as u32 }
    }
}
impl Iterator for IdRange {
    type Item = Id;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let id = Id::from(self.current as usize);
            self.current += 1;
            Some(id)
        } else {
            None
        }
    }
}

/// A factory for [Id]s.
pub trait IdFactory {
    type Id;
    /// #### Return
    /// A new [Id].
    fn create_id() -> Self::Id;
}

/// A unique identifier for an element.
pub struct UniqueIds;
impl IdFactory for UniqueIds {
    type Id = Id;
    fn create_id() -> Self::Id {
        use std::sync::atomic::*;
        static ID_COUNT: AtomicU32 = AtomicU32::new(0);
        Id::from(ID_COUNT.fetch_add(1, Ordering::Relaxed) as usize)
    }
}

/// Utilities for generating identifiers bound to machine-time.
pub struct TimeIds;
impl IdFactory for TimeIds {
    type Id = u128;
    //// #### Return
    /// A new identifier bound to the current machine-time.
    fn create_id() -> Self::Id {
        use std::time::SystemTime;
        SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos()
    }
}

/// Utilities for generating identifiers bound to unique names.
pub struct SymbolIds;
/// The names bound to [SymbolIds]. Names are leaked when first added, as they
/// are never removed from the vocabulary.
static VOCABULARY: OnceLock<Mutex<IndexSet<&'static str>>> = OnceLock::new();
impl SymbolIds {
    pub fn from_string(string: &str) -> Id {
        let mut v = VOCABULARY.get_or_init(|| Mutex::new(Default::default())).lock().unwrap();
        let index = match v.get_index_of(string) {
            Some(index) => index,
            None => v.insert_full(Box::leak(Box::from(string))).0,
        };
        Id::from(index)
    }
    pub fn to_string(id: Id) -> Option<&'static str> {
        let v = VOCABULARY.get_or_init(|| Mutex::new(Default::default())).lock().unwrap();
        v.get_index(id.into()).copied()
    }
}
