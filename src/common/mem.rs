use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

pub trait MemAccess {
    fn read(&mut self, address: u16) -> u8;

    fn peek(&self, address: u16) -> u8;

    fn read_u16(&mut self, address: u16) -> u16 {
        let lo = self.read(address);
        let hi = self.read(address.wrapping_add(1));

        u16::from_le_bytes([lo, hi])
    }

    fn peek_u16(&self, address: u16) -> u16 {
        let lo = self.peek(address);
        let hi = self.peek(address.wrapping_add(1));

        u16::from_le_bytes([lo, hi])
    }

    fn write(&mut self, address: u16, value: u8);
}

pub trait Reset {
    fn reset(&mut self);
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Memory(Vec<u8>);

impl Memory {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_size(size: usize) -> Self {
        Self(vec![0; size])
    }
}

impl From<Vec<u8>> for Memory {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl Deref for Memory {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Memory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for Memory {
    type Item = u8;

    type IntoIter = std::vec::IntoIter<u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Memory {
    type Item = &'a u8;

    type IntoIter = std::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Memory {
    type Item = &'a mut u8;

    type IntoIter = std::slice::IterMut<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl Debug for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Memory")
            .field("len", &self.0.len())
            .field("capacity", &self.0.capacity())
            .finish()
    }
}
