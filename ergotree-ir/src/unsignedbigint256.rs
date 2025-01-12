//! 256-bit unsigned big integer type
use core::ops::{Div, Mul, Rem};

use bnum::{types::U256, BInt, BTryFrom, BUint};
use derive_more::{Add, AddAssign, BitAnd, BitOr, BitXor, Display, From, FromStr, Not, Sub};
use num_derive::{Num, One, Zero};
use num_traits::{Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedRem, CheckedSub, Signed};

use crate::{
    bigint256::BigInt256,
    serialization::{SigmaParsingError, SigmaSerializable},
};

#[derive(
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Display,
    From,
    FromStr,
    Copy,
    Clone,
    Zero,
    One,
    Num,
    Not,
    Add,
    AddAssign,
    Sub,
    BitAnd,
    BitOr,
    BitXor,
)]
/// Unsigned 256-bit integer type
pub struct UnsignedBigInt(U256);

impl UnsignedBigInt {
    /// Create a BigInt256 from a slice of bytes in big-endian format. Returns None if slice.len() > 32 || slice.len() == 0
    pub fn from_be_slice(slice: &[u8]) -> Option<Self> {
        // match scala implementation which returns exception with empty byte array, whereas bnum returns 0
        if slice.is_empty() {
            return None;
        }
        U256::from_be_slice(slice).map(Self)
    }
    fn widen_to<const N: usize>(&self) -> BUint<N> {
        #[allow(clippy::unwrap_used)]
        // widen_to is only used for internal operations (modulo) and all of those will use widen_to with N > 4
        <BUint<N> as BTryFrom<U256>>::try_from(self.0).unwrap()
    }
    fn from_wide<const N: usize>(num: BUint<N>) -> Option<Self> {
        <U256 as BTryFrom<_>>::try_from(num).ok().map(Self)
    }

    /// Perform (self + other) % modulus. Returns None if modulus == 0
    pub fn checked_mod_add(&self, other: Self, modulus: Self) -> Option<Self> {
        // Perform addition using 320-bit result. Addition of two 256-bit ints can fit in 257 bits but using 5 * 64-bit ints should produce more efficient arithmetic on most machines
        let self_wide = self.widen_to::<5>();
        let other_wide = other.widen_to::<5>();
        let modulus_wide = modulus.widen_to::<5>();
        #[allow(clippy::unwrap_used)] // modulus < 2.pow(256) so this will never fail
        Some(Self::from_wide((self_wide + other_wide).checked_rem(modulus_wide)?).unwrap())
    }

    /// Calculate (self - other) % modulus
    pub fn checked_mod_sub(&self, other: Self, modulus: Self) -> Option<Self> {
        let other_inv = modulus.checked_sub(&(other % modulus))?;
        self.checked_mod_add(other_inv, modulus)
    }

    /// Perform (self * other) % modulus. Returns None if modulus == 0
    pub fn checked_mod_mul(&self, other: Self, modulus: Self) -> Option<Self> {
        let self_wide = self.widen_to::<8>();
        let other_wide = other.widen_to::<8>();
        let modulus_wide = modulus.widen_to::<8>();
        #[allow(clippy::unwrap_used)] // modulus < 2.pow(256) so this will never fail
        Some(Self::from_wide((self_wide * other_wide).checked_rem(modulus_wide)?).unwrap())
    }

    /// Compute modular inverse x such that (self * x) mod modulus = 1
    /// Returns None if modulus == 0 or inverse does not exist
    pub fn mod_inv(&self, modulus: Self) -> Option<Self> {
        /// Run the extended euclidean algorithm ax + by = gcd(a, b). Returns (gcd(a, b), x).
        fn extended_euclidean(a: UnsignedBigInt, b: UnsignedBigInt) -> (BInt<5>, BInt<5>) {
            let mut a = (a % b).widen_to().cast_signed();
            let mut b = b.widen_to().cast_signed();
            if b.is_zero() {
                return (a, 1.into());
            }
            let mut x0x1: (BInt<5>, BInt<5>) = (1.into(), 0.into());
            while !b.is_zero() {
                let q = a / b;
                x0x1 = (x0x1.1, x0x1.0 - q * x0x1.1);
                let tmp = a - q * b;
                a = b;
                b = tmp;
            }
            (a, x0x1.0)
        }
        let (g, x) = extended_euclidean(*self, modulus);
        if g.is_one() {
            let inv = x.checked_rem_euclid(modulus.widen_to().cast_signed())?;
            #[allow(clippy::unwrap_used)]
            // unwrap is used here for clarity, since inv is computed mod b, it will always fit in 256 bits and so this branch will always return Some(_)
            Some(Self::from_wide(inv.cast_unsigned()).unwrap())
        } else {
            None
        }
    }
    /// Convert UnsignedBigInt to minimum number of bytes to represent it
    /// # Example
    /// ```
    /// # use ergotree_ir::unsignedbigint256::UnsignedBigInt;
    /// use num_traits::Num;
    ///
    /// let num = UnsignedBigInt::from_str_radix("ff", 16).unwrap();
    /// let num_bytes = num.to_be_vec();
    /// assert_eq!(num_bytes, vec![0xff]);
    /// assert_eq!(num, UnsignedBigInt::from_be_slice(&num_bytes).unwrap());
    ///
    /// let neg = UnsignedBigInt::from_str_radix("1", 16).unwrap();
    /// let neg_bytes = neg.to_be_vec();
    /// assert_eq!(neg_bytes, vec![0x01]);
    /// assert_eq!(neg, UnsignedBigInt::from_be_slice(&neg_bytes).unwrap());
    /// ```
    pub fn to_be_vec(&self) -> Vec<u8> {
        self.0.to_radix_be(256)
    }
}

impl From<u32> for UnsignedBigInt {
    fn from(value: u32) -> Self {
        Self(U256::from(value))
    }
}

impl TryFrom<UnsignedBigInt> for BigInt256 {
    type Error = &'static str;

    fn try_from(value: UnsignedBigInt) -> Result<Self, Self::Error> {
        if value.0.bit(U256::BITS - 1) {
            Err("UnsignedBigInt out of bounds")
        } else {
            Ok(BigInt256(value.0.cast_signed()))
        }
    }
}

impl TryFrom<BigInt256> for UnsignedBigInt {
    type Error = &'static str;

    fn try_from(value: BigInt256) -> Result<Self, Self::Error> {
        if value.is_negative() {
            Err("Can not convert negative BigInt to UnsignedBigInt")
        } else {
            Ok(Self(value.0.cast_unsigned()))
        }
    }
}

impl CheckedAdd for UnsignedBigInt {
    fn checked_add(&self, other: &Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }
}

impl CheckedSub for UnsignedBigInt {
    fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }
}

impl CheckedMul for UnsignedBigInt {
    fn checked_mul(&self, other: &Self) -> Option<Self> {
        self.0.checked_mul(other.0).map(Self)
    }
}

impl CheckedDiv for UnsignedBigInt {
    fn checked_div(&self, other: &Self) -> Option<Self> {
        self.0.checked_div(other.0).map(Self)
    }
}

impl CheckedRem for UnsignedBigInt {
    fn checked_rem(&self, v: &Self) -> Option<Self> {
        self.0.checked_rem(v.0).map(Self)
    }
}

impl Bounded for UnsignedBigInt {
    fn min_value() -> Self {
        Self(U256::min_value())
    }

    fn max_value() -> Self {
        Self(U256::max_value())
    }
}

impl Mul for UnsignedBigInt {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        UnsignedBigInt(self.0 * rhs.0)
    }
}

impl Div for UnsignedBigInt {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        UnsignedBigInt(self.0 / rhs.0)
    }
}

impl Rem for UnsignedBigInt {
    type Output = UnsignedBigInt;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0 % rhs.0)
    }
}

impl SigmaSerializable for UnsignedBigInt {
    fn sigma_serialize<W: crate::serialization::sigma_byte_writer::SigmaByteWrite>(
        &self,
        w: &mut W,
    ) -> crate::serialization::SigmaSerializeResult {
        let bytes = self.to_be_vec();
        w.put_u16(bytes.len() as u16)?;
        w.write_all(&bytes)?;
        Ok(())
    }

    fn sigma_parse<R: crate::serialization::sigma_byte_reader::SigmaByteRead>(
        r: &mut R,
    ) -> Result<Self, crate::serialization::SigmaParsingError> {
        let size = r.get_u16()?;
        if size > 32 {
            return Err(SigmaParsingError::ValueOutOfBounds(format!(
                "serialized BigInt size {0} bytes exceeds 32",
                size
            )));
        }
        let mut buf = vec![0u8; size as usize];
        r.read_exact(&mut buf)?;
        match UnsignedBigInt::from_be_slice(&buf) {
            Some(x) => Ok(x),
            None => Err(SigmaParsingError::ValueOutOfBounds(String::new())),
        }
    }
}

#[cfg(feature = "arbitrary")]
mod arbitrary {
    use proptest::{
        arbitrary::{any, Arbitrary},
        strategy::{BoxedStrategy, Strategy},
    };

    use super::UnsignedBigInt;

    impl Arbitrary for UnsignedBigInt {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            #[allow(clippy::unwrap_used)]
            any::<[u8; 32]>()
                .prop_map(|bytes| Self::from_be_slice(&bytes[..]).unwrap())
                .boxed()
        }
    }
}

#[cfg(test)]
#[cfg(feature = "arbitrary")]
#[allow(clippy::unwrap_used)]
mod test {
    use num_bigint::BigInt;
    use num_traits::{Euclid, Num, Zero};
    use proptest::prelude::*;

    use crate::serialization::SigmaSerializable;

    use super::UnsignedBigInt;
    // Inefficient impl of UnsignedBigInt -> BigInt, this is only used for tests so should be acceptable
    fn to_bigint(num: UnsignedBigInt) -> BigInt {
        BigInt::from_str_radix(&num.to_string(), 10).unwrap()
    }
    proptest! {
        #[test]
        fn ser_roundtrip(v in any ::<UnsignedBigInt>()) {
            assert_eq!(v, UnsignedBigInt::sigma_parse_bytes(&v.sigma_serialize_bytes().unwrap()).unwrap());
        }
        #[test]
        fn mod_add(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>(), c in any::<UnsignedBigInt>()) {
            let a_bigint = to_bigint(a);
            let b_bigint = to_bigint(b);
            let c_bigint = to_bigint(c);
            let res = a.checked_mod_add(b, c);
            if c != UnsignedBigInt::zero() {
                assert_eq!(to_bigint(res.unwrap()), ((a_bigint + b_bigint) % c_bigint));
            }
            else {
                assert!(res.is_none());
            }
        }
        #[test]
        fn mod_sub(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>(), c in any::<UnsignedBigInt>()) {
            let a_bigint = to_bigint(a);
            let b_bigint = to_bigint(b);
            let c_bigint = to_bigint(c);
            let res = a.checked_mod_sub(b, c);
            if c != UnsignedBigInt::zero() {
                assert_eq!(to_bigint(res.unwrap()), ((a_bigint - b_bigint).rem_euclid(&c_bigint)));
            }
            else {
                assert!(res.is_none());
            }
        }
        #[test]
        fn mod_mul(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>(), c in any::<UnsignedBigInt>()) {
            let a_bigint = to_bigint(a);
            let b_bigint = to_bigint(b);
            let c_bigint = to_bigint(c);
            let res = a.checked_mod_mul(b, c);
            if c != UnsignedBigInt::zero() {
                assert_eq!(to_bigint(res.unwrap()), (a_bigint * b_bigint) % c_bigint);
            }
            else {
                assert!(res.is_none());
            }
        }
        #[test]
        fn mod_inv(a in any::<UnsignedBigInt>(), b in any::<UnsignedBigInt>()) {
            let a_bigint = to_bigint(a);
            let b_bigint = to_bigint(b);
            let inverse = a.mod_inv(b);
            if let Some(inverse) = inverse {
                let inverse_bigint = a_bigint.modinv(&b_bigint);
                assert_eq!(a.checked_mod_mul(inverse, b).unwrap(), 1.into());
                assert_eq!(to_bigint(inverse), inverse_bigint.unwrap())
            }
            else if !b.is_zero() {
                assert!(a_bigint.modinv(&b_bigint).is_none());
            }
        }
    }
}
