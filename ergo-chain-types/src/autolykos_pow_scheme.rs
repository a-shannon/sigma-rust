//! Autolykos Pow puzzle scheme implementation
//!
//! See for reference implmentation - <https://github.com/ergoplatform/ergo/blob/f7b91c0be00531c6d042c10a8855149ca6924373/src/main/scala/org/ergoplatform/mining/AutolykosPowScheme.scala>
//!
//! Based on k-sum problem, so general idea is to find k numbers in a table of size N, such that
//! sum of numbers (or a hash of the sum) is less than target value.
//!
//! See <https://docs.ergoplatform.com/ErgoPow.pdf> for details
//!
//! CPU Mining process is implemented in inefficient way and should not be used in real environment.
//!
//! See <https://github.com/ergoplatform/ergo/papers/yellow/pow/ErgoPow.tex> for full description
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use bounded_integer::{BoundedI32, BoundedU64};
use derive_more::From;
use k256::{elliptic_curve::PrimeField, Scalar};
use num_bigint::{BigInt, Sign};
use num_traits::Num;
use sigma_ser::ScorexSerializationError;
use sigma_util::hash::blake2b256_hash;

use crate::Header;

/// The "compact" format is an encoding of a whole number `N` using an unsigned 32 bit number.
/// This number encodes a base-256 scientific notation representation of `N` (similar to a floating
/// point format):
///  - The most significant 8 bits represent the number of bytes necessary to represent `N` in
///    two's-complement form; denote it by `exponent`.
///  - The lower 23 bits are the mantissa(significand).
///  - Bit number 24 (0x800000) represents the sign of N.
///
/// There are 2 cases to consider:
///  - If `exponent >= 3` then `N` is represented by
///      `(-1^sign) * mantissa * 256^(exponent-3)`
///    E.g. suppose the compact form is given in hex-format by `0x04123456`. Mantissa is `0x123456`
///    and `exponent == 4`. So `N == 0x123456 * 265^1`. Now note that we need exactly 4 bytes to
///    represent `N`; 3 bytes for the mantissa and 1 byte for the rest. In base-256:
///      `N == B(0x12)B(0x34)B(0x56)0`
///    where `B(y)` denotes the base-256 representation of a hex-number `y` (note how each base-256
///    digit is represented by a single-byte).
///  - If `exponent < 3` then `N` is represented by the `exponent`-most-significant-bytes of the
///    mantissa. E.g. suppose the compact form is given in hex-format by `0x01003456`. Noting that
///    each hex-digit is represented by 4-bits, our `exponent == 0x01` which is `1` base-10.  The
///    mantissa is represented by `0x003456` and it's most signficant byte is `0x00`. Therefore
///    `N == 0`.
///
/// Satoshi's original implementation used BN_bn2mpi() and BN_mpi2bn(). MPI uses the most
/// significant bit of the first byte as sign. Thus 0x1234560000 is compact 0x05123456 and
/// 0xc0de000000 is compact 0x0600c0de. Compact 0x05c0de00 would be -0x40de000000.
///
/// Bitcoin only uses this "compact" format for encoding difficulty targets, which are unsigned
/// 256bit quantities.  Thus, all the complexities of the sign bit and using base 256 are probably
/// an implementation accident.
pub fn decode_compact_bits(n_bits: u64) -> BigInt {
    let compact = n_bits as i64;
    let size = ((compact >> 24) as i32) & 0xFF;
    if size == 0 {
        return BigInt::from(0);
    }
    let mut buf: Vec<i8> = vec![0; size as usize];
    if size >= 1 {
        // Store the first byte of the mantissa
        buf[0] = (((compact >> 16) as i32) & 0xFF) as i8;
    }
    if size >= 2 {
        buf[1] = (((compact >> 8) as i32) & 0xFF) as i8;
    }
    if size >= 3 {
        buf[2] = ((compact as i32) & 0xFF) as i8;
    }

    let is_negative = (buf[0] as i32) & 0x80 == 0x80;
    if is_negative {
        buf[0] &= 0x7f;
        let buf: Vec<_> = buf.into_iter().map(|x| x as u8).collect();
        -BigInt::from_signed_bytes_be(&buf)
    } else {
        let buf: Vec<_> = buf.into_iter().map(|x| x as u8).collect();
        BigInt::from_signed_bytes_be(&buf)
    }
}

/// Order of the secp256k1 elliptic curve as BigInt
pub fn order_bigint() -> BigInt {
    #[allow(clippy::unwrap_used)]
    BigInt::from_str_radix(Scalar::MODULUS, 16).unwrap()
}

/// Autolykos PoW puzzle scheme implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutolykosPowScheme {
    /// Represents the number of elements in one solution. **Important assumption**: `k <= 32`.
    k: BoundedU64<1, 32>,
    /// Let `N` denote the initial table size. Then `n` is the value satisfying `N = 2 ^ n`.
    /// **Important assumption**: `n < 31`.
    n: BoundedI32<1, 30>,
}

impl AutolykosPowScheme {
    /// Get hit for Autolykos header (to test it then against PoW target)
    pub fn pow_hit(&self, header: &Header) -> Result<BigInt, AutolykosPowSchemeError> {
        if header.version == 1 {
            header
                .autolykos_solution
                .pow_distance
                .as_ref()
                .cloned()
                .ok_or(AutolykosPowSchemeError::MissingPowDistanceParameter)
        } else {
            // hit for version 2
            let msg = blake2b256_hash(&header.serialize_without_pow()?).to_vec();
            let nonce = header.autolykos_solution.nonce.clone();
            let height_bytes = header.height.to_be_bytes();

            let mut concat = msg.clone();
            concat.extend(&nonce);

            // `N` from autolykos paper
            let big_n = self.calc_big_n(header.version, header.height);
            let seed_hash = self.calc_seed_v2(big_n, &msg, &nonce, &height_bytes)?;
            let indexes = self.gen_indexes(&seed_hash, big_n);

            let f2 = indexes.into_iter().fold(BigInt::from(0u32), |acc, idx| {
                // This is specific to autolykos v2.
                let mut concat = vec![];
                concat.extend_from_slice(&idx.to_be_bytes());
                concat.extend(&height_bytes);
                concat.extend(&self.calc_big_m());
                acc + BigInt::from_bytes_be(Sign::Plus, &blake2b256_hash(&concat)[1..])
            });

            // sum as byte array is always about 32 bytes
            #[allow(clippy::unwrap_used)]
            let array = as_unsigned_byte_array(32, f2).unwrap();
            Ok(BigInt::from_bytes_be(Sign::Plus, &*blake2b256_hash(&array)))
        }
    }

    /// Constant data to be added to hash function to increase its calculation time
    pub fn calc_big_m(&self) -> Vec<u8> {
        (0u64..1024).flat_map(|x| x.to_be_bytes()).collect()
    }

    /// Computes `J` (denoted by `seed` in Ergo implementation) line 4, algorithm 1 of Autolykos v2
    /// in ErgoPow paper.
    pub fn calc_seed_v2(
        &self,
        big_n: usize,
        msg: &[u8],
        nonce: &[u8],
        header_height_bytes: &[u8],
    ) -> Result<Box<[u8; 32]>, AutolykosPowSchemeError> {
        let mut concat: Vec<u8> = vec![];
        concat.extend(msg);
        concat.extend(nonce);

        let pre_i8 = BigInt::from_bytes_be(Sign::Plus, &(blake2b256_hash(&concat)[(32 - 8)..]));

        // Note that `N` parameter has an upper bound of 2,147,387,550 which can fit in a `i32` (4
        // bytes), so the truncation for `i` below is safe.
        let i =
            as_unsigned_byte_array(4, pre_i8.modpow(&BigInt::from(1u32), &BigInt::from(big_n)))?;

        let big_m = self.calc_big_m();

        concat = i;
        concat.extend(header_height_bytes);
        concat.extend(&big_m);
        let f = blake2b256_hash(&concat);

        concat = f[1..].to_vec();
        concat.extend(msg);
        concat.extend(nonce);
        Ok(blake2b256_hash(&concat))
    }

    /// Returns a list of size `k` with numbers in [0,`N`)
    pub fn gen_indexes(&self, seed_hash: &[u8; 32], big_n: usize) -> Vec<u32> {
        let mut res = vec![];
        let mut extended_hash: Vec<u8> = seed_hash.to_vec();
        extended_hash.extend(&seed_hash[..3]);
        for i in 0..self.k.get() {
            let i = i as usize;
            res.push(
                BigInt::from_bytes_be(Sign::Plus, &extended_hash[i..(i + 4)])
                    .modpow(&BigInt::from(1u32), &BigInt::from(big_n))
                    .to_u32_digits()
                    .1[0],
            );
        }
        res
    }

    /// Calculates table size (N value) for a given height (moment of time)
    pub fn calc_big_n(&self, header_version: u8, header_height: u32) -> usize {
        // Number of elements in a table to find k-sum problem solution on top of
        let n_base = 2i32.pow(self.n.get() as u32) as usize;
        if header_version == 1 {
            n_base
        } else {
            // On this height, the table (`N` value) will stop to grow
            let n_increasement_height_max = 4198400;
            let height = usize::min(n_increasement_height_max, header_height as usize);

            // Initial height since which table (`N` value) starting to increase by 5% per `IncreasePeriodForN` blocks
            let increase_start = 600 * 1024;
            if height < increase_start {
                n_base
            } else {
                // Table size (`N`) increased every 50 * 1024 blocks
                let increase_period_for_big_n = 50 * 1024;
                let iters_number = (height - increase_start) / increase_period_for_big_n + 1;
                (1..=iters_number).fold(n_base, |acc, _| acc / 100 * 105)
            }
        }
    }
}

impl Default for AutolykosPowScheme {
    fn default() -> Self {
        // The following paramter values are mandated by Ergo-node Autolykos implementation.
        #[allow(clippy::unwrap_used)]
        AutolykosPowScheme {
            k: BoundedU64::new(32).unwrap(),
            n: BoundedI32::new(26).unwrap(),
        }
    }
}

/// Port of BouncyCastle's BigIntegers::asUnsignedByteArray method.
fn as_unsigned_byte_array(
    length: usize,
    big_int: BigInt,
) -> Result<Vec<u8>, AutolykosPowSchemeError> {
    let bytes = big_int.to_signed_bytes_be();
    if bytes.len() == length {
        return Ok(bytes);
    }

    let start = usize::from(bytes[0] == 0);
    let count = bytes.len() - start;
    if count > length {
        return Err(AutolykosPowSchemeError::BigIntToFixedByteArrayError);
    }
    let mut res: Vec<_> = vec![0; length];
    res[(length - count)..].copy_from_slice(&bytes[start..]);
    Ok(res)
}

/// Autolykos POW scheme error
#[derive(PartialEq, Eq, Debug, Clone, From)]
pub enum AutolykosPowSchemeError {
    /// Scorex-serialization error
    ScorexSerializationError(ScorexSerializationError),
    /// Error occurring when trying to convert a `BigInt` into a fixed-length byte-array.
    BigIntToFixedByteArrayError,
    /// Occurs when `Header.version == 1` and the `pow_distance` parameter is None.
    MissingPowDistanceParameter,
}

/// The following tests are taken from <https://github.com/ergoplatform/ergo/blob/f7b91c0be00531c6d042c10a8855149ca6924373/src/test/scala/org/ergoplatform/mining/AutolykosPowSchemeSpec.scala#L43-L130>
#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use num_bigint::ToBigInt;
    use sigma_ser::ScorexSerializable;

    use super::*;

    #[test]
    fn test_calc_big_n() {
        let pow = AutolykosPowScheme::default();
        let n_base = 2i32.pow(pow.n.get() as u32) as usize;

        // autolykos v1
        assert_eq!(pow.calc_big_n(1, 700000), n_base);
        assert_eq!(pow.calc_big_n(1, 100000), n_base);
        assert_eq!(pow.calc_big_n(1, 70000000), n_base);

        // autolykos v2
        assert_eq!(pow.calc_big_n(2, 500000), n_base);
        assert_eq!(pow.calc_big_n(2, 600000), n_base);
        assert_eq!(pow.calc_big_n(2, 600 * 1024), 70464240);
        assert_eq!(pow.calc_big_n(2, 650 * 1024), 73987410);
        assert_eq!(pow.calc_big_n(2, 700000), 73987410);
        assert_eq!(pow.calc_big_n(2, 788400), 81571035); // 3 years
        assert_eq!(pow.calc_big_n(2, 1051200), 104107290); // 4 years
        assert_eq!(pow.calc_big_n(2, 4198400), 2143944600); // max height
        assert_eq!(pow.calc_big_n(2, 41984000), 2143944600);
    }

    #[test]
    fn test_first_increase_in_big_n() {
        // Test vectors for first increase in N value (height 614,400)
        let json = r#"
          {
            "extensionId" : "00cce45975d87414e8bdd8146bc88815be59cd9fe37a125b5021101e05675a18",
            "difficulty" : "16384",
            "votes" : "000000",
            "timestamp" : 4928911477310178288,
            "size" : 223,
            "stateRoot" : "5c8c00b8403d3701557181c8df800001b6d5009e2201c6ff807d71808c00019780",
            "height" : 614400,
            "nBits" : 37748736,
            "version" : 2,
            "id" : "5603a937ec1988220fc44fb5022fb82d5565b961f005ebb55d85bd5a9e6f801f",
            "adProofsRoot" : "5d3f80dcff7f5e7f59007294c180808d0158d1ff6ba10000f901c7f0ef87dcff",
            "transactionsRoot" : "f17fffacb6ff7f7f1180d2ff7f1e24ffffe1ff937f807f0797b9ff6ebdae007e",
            "extensionHash" : "1480887f80007f4b01cf7f013ff1ffff564a0000b9a54f00770e807f41ff88c0",
            "powSolutions" : {
              "pk" : "03bedaee069ff4829500b3c07c4d5fe6b3ea3d3bf76c5c28c1d4dcdb1bed0ade0c",
              "n" : "0000000000003105"
             },
            "adProofsId" : "dec129290a763f4de41f04e87e2b661dd59758af6bdd00dd51f5d97c3a8cb9b5",
            "transactionsId" : "eba1dd82cf51147232e09c1f72b37c554c30f63274d5093bff36849a83472a42",
            "parentId" : "ac2101807f0000ca01ff0119db227f202201007f62000177a080005d440896d0"
          }
          "#;

        let header: Header = serde_json::from_str(json).unwrap();
        assert_eq!(header.height, 614400);

        let msg = base16::encode_lower(&*blake2b256_hash(&header.serialize_without_pow().unwrap()));
        assert_eq!(
            msg,
            "548c3e602a8f36f8f2738f5f643b02425038044d98543a51cabaa9785e7e864f"
        );

        let pow = AutolykosPowScheme::default();
        assert_eq!(pow.calc_big_n(header.version, header.height), 70464240);

        // Vector obtained from a miner dev
        let hit = pow.pow_hit(&header).unwrap();
        assert_eq!(
            hit,
            BigInt::from_signed_bytes_be(
                &base16::decode("0002fcb113fe65e5754959872dfdbffea0489bf830beb4961ddc0e9e66a1412a")
                    .unwrap()
            )
        );

        // Check decoded header.nBits
        let decoded = decode_compact_bits(header.n_bits);

        // Target `b` from encoded difficulty `nBits`
        let target_b = order_bigint() / decoded;
        assert_eq!(
            target_b,
            BigInt::parse_bytes(
                b"7067388259113537318333190002971674063283542741642755394446115914399301849",
                10
            )
            .unwrap()
        );

        assert_eq!(
            base16::encode_lower(
                &header
                    .autolykos_solution
                    .miner_pk
                    .scorex_serialize_bytes()
                    .unwrap()
            ),
            "03bedaee069ff4829500b3c07c4d5fe6b3ea3d3bf76c5c28c1d4dcdb1bed0ade0c"
        );

        assert_eq!(
            base16::encode_lower(&header.autolykos_solution.nonce),
            "0000000000003105"
        );

        // Check that header is valid
        assert!(hit < target_b);
    }

    #[test]
    fn test_invalid_header() {
        let json = "{\"extensionId\":\"277907e4e5e42f27e928e6101cc4fec173bee5d7728794b73d7448c339c380e5\",\"difficulty\":\"1325481984\",\"votes\":\"000000\",\"timestamp\":1611225263165,\"size\":219,\"stateRoot\":\"c0d0b5eafd07b22487dac66628669c42a242b90bef3e1fcdc76d83140d58b6bc0e\",\"height\":2870,\"nBits\":72286528,\"version\":2,\"id\":\"5b0ce6711de6b926f60b67040cc4512804517785df375d063f1bf1d75588af3a\",\"adProofsRoot\":\"49453875a43035c7640dee2f905efe06128b00d41acd2c8df13691576d4fd85c\",\"transactionsRoot\":\"770cbb6e18673ed025d386487f15d3252115d9a6f6c9b947cf3d04731dd6ab75\",\"extensionHash\":\"9bc7d54583c5d44bb62a7be0473cd78d601822a626afc13b636f2cbff0d87faf\",\"powSolutions\":{\"pk\":\"0288114b0586efea9f86e4587f2071bc1c85fb77e15eba96b2769733e0daf57903\",\"w\":\"0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798\",\"n\":\"000100000580a91b\",\"d\":0},\"adProofsId\":\"4fc36d59bf26a672e01fbfde1445bd66f50e0f540f24102e1e27d0be1a99dfbf\",\"transactionsId\":\"d196ef8a7ef582ab1fdab4ef807715183705301c6ae2ff0dcbe8f1d577ba081f\",\"parentId\":\"ab19e6c7a4062979dddb534df83f236d1b949c7cef18bcf434a67e87c593eef9\"}";
        let header: Header = serde_json::from_str(json).unwrap();
        let pow = AutolykosPowScheme::default();
        // Check decoded header.nBits
        let decoded = decode_compact_bits(header.n_bits);

        // Target `b` from encoded difficulty `nBits`
        let target_b = order_bigint() / decoded;
        let hit = pow.pow_hit(&header).unwrap();

        assert!(hit >= target_b);
    }

    #[test]
    fn test_decode_n_bits() {
        // Following example taken from https://btcinformation.org/en/developer-reference#target-nbits
        let n_bits = 0x181bc330;
        assert_eq!(
            decode_compact_bits(n_bits),
            BigInt::parse_bytes(b"1bc330000000000000000000000000000000000000000000", 16).unwrap()
        );

        let n_bits = 0x01003456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&0x00).unwrap()
        );

        let n_bits = 0x01123456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&0x12).unwrap()
        );

        let n_bits = 0x04923456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&-0x12345600).unwrap()
        );

        let n_bits = 0x04123456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&0x12345600).unwrap()
        );

        let n_bits = 0x05123456;
        assert_eq!(
            decode_compact_bits(n_bits),
            ToBigInt::to_bigint(&0x1234560000i64).unwrap()
        );

        let n_bits = 16842752;
        assert_eq!(decode_compact_bits(n_bits), BigInt::from(1_u8));
    }
}
