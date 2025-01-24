// re-export num_traits here so it can be used in ergo-lib-c
pub use num_traits::*;

// hack to avoid cbindgen outputting an opaque type since it can't see type of UnsignedBigInt
#[repr(C)]
#[derive(Copy, Clone)]
pub struct UnsignedBigInt(pub [u64; 4]);
pub use ergo_lib::ergotree_ir::unsignedbigint256::UnsignedBigInt as UnsignedBigIntRaw;
impl From<UnsignedBigInt> for UnsignedBigIntRaw {
    fn from(value: UnsignedBigInt) -> Self {
        UnsignedBigIntRaw::from_limbs(value.0)
    }
}
impl From<UnsignedBigIntRaw> for UnsignedBigInt {
    fn from(value: UnsignedBigIntRaw) -> Self {
        UnsignedBigInt(value.to_limbs())
    }
}
pub type UnsignedBigIntPtr = *mut UnsignedBigInt;
pub type ConstUnsignedBigIntPtr = *const UnsignedBigInt;
