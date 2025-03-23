import ErgoLibC

struct ArithmeticError: Error {
}
internal func wrapOp(closure: (_ out: ErgoLibC.UnsignedBigIntPtr) -> UInt32) throws
    -> UnsignedBigInt
{
    var out = ergo_lib_u256_new()
    let res = closure(&out)
    if res != 0 {
        throw ArithmeticError()
    }
    return UnsignedBigInt(inner: out)
}
class UnsignedBigInt: Equatable, Comparable {
    internal var inner: ErgoLibC.UnsignedBigInt
    /// Decode string in a given base
    init(str: String, radix: UInt32) throws {
        let res = try str.withCString { cs in
            try wrapOp(closure: { ergo_lib_u256_from_str_radix(cs, radix, $0) })
        }
        self.inner = res.inner
    }
    init(long: UInt64) {
        self.inner = ergo_lib_u256_from_long(long)
    }
    init() {
        self.inner = ergo_lib_u256_new()
    }
    internal init(inner: ErgoLibC.UnsignedBigInt) {
        self.inner = inner
    }
    func modAdd(_ b: UnsignedBigInt, _ modulus: UnsignedBigInt) throws -> UnsignedBigInt {
        return try wrapOp(closure: { ergo_lib_u256_mod_add(&self.inner, &b.inner, &modulus.inner, $0)})
    }
    func modSub(_ b: UnsignedBigInt, _ modulus: UnsignedBigInt) throws -> UnsignedBigInt {
        return try wrapOp(closure: { ergo_lib_u256_mod_sub(&self.inner, &b.inner, &modulus.inner, $0)})
    }
    func modMul(_ b: UnsignedBigInt, _ modulus: UnsignedBigInt) throws -> UnsignedBigInt {
        return try wrapOp(closure: { ergo_lib_u256_mod_mul(&self.inner, &b.inner, &modulus.inner, $0)})
    }
    func modInv(_ modulus: UnsignedBigInt) throws -> UnsignedBigInt {
        return try wrapOp(closure: { ergo_lib_u256_mod_inv(&self.inner, &modulus.inner, $0)})
    }
    static func + (a: UnsignedBigInt, b: UnsignedBigInt) throws -> UnsignedBigInt {
        return try wrapOp(closure: { ergo_lib_u256_add(&a.inner, &b.inner, $0) })
    }
    static func - (a: UnsignedBigInt, b: UnsignedBigInt) throws -> UnsignedBigInt {
        return try wrapOp(closure: { ergo_lib_u256_sub(&a.inner, &b.inner, $0) })
    }
    static func * (a: UnsignedBigInt, b: UnsignedBigInt) throws -> UnsignedBigInt {
        return try wrapOp(closure: { ergo_lib_u256_mul(&a.inner, &b.inner, $0) })
    }
    static func % (a: UnsignedBigInt, b: UnsignedBigInt) throws -> UnsignedBigInt {
        return try wrapOp(closure: { ergo_lib_u256_mod(&a.inner, &b.inner, $0) })
    }
    static func == (lhs: UnsignedBigInt, rhs: UnsignedBigInt) -> Bool {
        ergo_lib_u256_eq(&lhs.inner, &rhs.inner)
    }
    static func < (lhs: UnsignedBigInt, rhs: UnsignedBigInt) -> Bool {
        ergo_lib_u256_cmp(&lhs.inner, &rhs.inner) < 0
    }

}
