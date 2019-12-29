#![allow(dead_code)]

pub trait TEnumFlags32 {
    type TRawType: std::convert::Into<u32> + std::convert::From<u32> + Copy + Clone;

    fn rawtype(&self) -> Self::TRawType;
}

pub struct SEnumFlags32<T: TEnumFlags32 + Copy> {
    raw: T::TRawType,
}

impl<T: TEnumFlags32 + Copy> From<T> for SEnumFlags32<T> {
    fn from(flag: T) -> Self {
        Self::none().or(flag)
    }
}

impl<T: TEnumFlags32 + Copy> Clone for SEnumFlags32<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: TEnumFlags32 + Copy> Copy for SEnumFlags32<T> {}

impl<T: TEnumFlags32 + Copy> SEnumFlags32<T> {
    pub fn none() -> Self {
        Self {
            raw: T::TRawType::from(0),
        }
    }

    pub fn all() -> Self {
        Self {
            raw: T::TRawType::from(std::u32::MAX),
        }
    }

    pub fn create(flags: &[T]) -> Self {
        let mut result = Self::none();
        for flag in flags {
            result = result.or(*flag);
        }
        result
    }

    pub fn and(&self, other: T) -> Self {
        let a: u32 = self.raw.into();
        let b: u32 = other.rawtype().into();
        let res: u32 = a & b;
        Self {
            raw: T::TRawType::from(res),
        }
    }

    pub fn or(&self, other: T) -> Self {
        let a: u32 = self.raw.into();
        let b: u32 = other.rawtype().into();
        let res: u32 = a | b;
        Self {
            raw: T::TRawType::from(res),
        }
    }

    pub fn rawtype(&self) -> T::TRawType {
        self.raw
    }
}
