// TODO there must be a tool to express pre- and post- conditions in Rust
/// A type bound to some invariant.
pub(crate) trait HasInvariant: Sized {
    /// #### Description
    /// Check the invariant of this type. Only active in debug builds.
    #[inline]
    fn debug_invariant(&self) {
        #[cfg(debug_assertions)]
        self.condition();
    }
    /// Implement this to define the invariant.
    #[inline]
    fn condition(&self) {}
}

/// #### Description
/// A small DSL to write conditions in propositional logic.
macro_rules! proposition {
    // Inductive Cases
    (($($inner:tt)+)) => { $crate::util::debug::proposition!($($inner)+) };
    (not $a:tt $($rest:tt)*) => { (!($crate::util::debug::proposition!($a))) $($rest)* };
    (!$a:tt $($rest:tt)*) => { (!($crate::util::debug::proposition!($a))) $($rest)* };
    ($a:tt and $($rest:tt)+) => { ($crate::util::debug::proposition!($a) && $crate::util::debug::proposition!($($rest)+)) };
    ($a:tt or $($rest:tt)+) => { ($crate::util::debug::proposition!($a) || $crate::util::debug::proposition!($($rest)+))};
    ($a:tt implies $($rest:tt)+) => { (!$crate::util::debug::proposition!($a) || $crate::util::debug::proposition!($($rest)+)) };
    ($a:tt => $($rest:tt)+) => { ($crate::util::debug::proposition!($a implies $($rest)+)) };
    ($a:tt iff $($rest:tt)+) => { ($crate::util::debug::proposition!($a) == $crate::util::debug::proposition!($($rest)+)) };
    ($a:tt <=> $($rest:tt)+) => { ($crate::util::debug::proposition!($a iff $($rest)+)) };
    // Base Case
    ($a:expr) => { $a };
}
pub(crate) use proposition;
