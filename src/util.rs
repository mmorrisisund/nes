use std::ops::{BitAnd, BitAndAssign, BitOrAssign, Not, Shl, Shr, Sub};

pub trait BitFlag<T> {
    fn get(&self, flag: T) -> T;
    fn contains(&self, flag: T) -> bool;
    fn set_flag(&mut self, flag: T);
    fn unset_flag(&mut self, flag: T);
}

impl<T> BitFlag<T> for T
where
    T: Clone
        + Copy
        + PartialEq
        + Shr<T, Output = T>
        + Shl<T, Output = T>
        + Sub<T, Output = T>
        + BitAnd<T, Output = T>
        + BitAndAssign<T>
        + BitOrAssign<T>
        + Not<Output = T>
        + From<u8>
        + std::fmt::UpperHex
        + Sized,
{
    fn get(&self, flag: T) -> T {
        *self >> flag & T::from(1)
    }

    fn contains(&self, flag: T) -> bool {
        self.get(flag) == T::from(1)
    }

    fn set_flag(&mut self, flag: T) {
        *self |= T::from(1) << flag;
    }

    fn unset_flag(&mut self, flag: T) {
        *self &= !(T::from(1) << flag);
    }
}
