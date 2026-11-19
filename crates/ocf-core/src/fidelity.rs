#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum FidelityClass {
    F0 = 0,
    F1 = 1,
    F2 = 2,
    F3 = 3,
}

impl FidelityClass {
    pub fn as_u8(self) -> u8 { self as u8 }
    pub fn from_u8(v: u8) -> Option<Self> {
        match v { 0 => Some(Self::F0), 1 => Some(Self::F1), 2 => Some(Self::F2), 3 => Some(Self::F3), _ => None }
    }
    pub fn is_lossless(self) -> bool { matches!(self, Self::F0 | Self::F1) }

    /// Returns true if a chunk stored at `stored` can satisfy a read requesting `requested`.
    /// Lower fidelity number = higher fidelity, so stored <= requested is allowed.
    pub fn satisfies(stored: FidelityClass, requested: FidelityClass) -> bool {
        stored <= requested
    }
}
