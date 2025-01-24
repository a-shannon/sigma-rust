import XCTest

@testable import ErgoLib

final class UnsignedBigIntTests: XCTestCase {
    func testConstant() {
        let val = UnsignedBigInt(long: 5)
        let a = Constant(withU256: val)
        XCTAssertEqual(try! a.toUnsignedBigInt(), val)
    }
    func testParsing() {
        XCTAssertEqual(try! UnsignedBigInt(str: "ff", radix: 16), UnsignedBigInt(long: 0xff))
        XCTAssertEqual(try! UnsignedBigInt(str: "10", radix: 10), UnsignedBigInt(long: 10))
    }
    func testArithmetic() {
        XCTAssert(UnsignedBigInt(long: 1) < UnsignedBigInt(long: 2))
        XCTAssertEqual(
            try! UnsignedBigInt(long: 5) + UnsignedBigInt(long: 3), UnsignedBigInt(long: 8))
        XCTAssertEqual(
            try! UnsignedBigInt(long: 5) - UnsignedBigInt(long: 3), UnsignedBigInt(long: 2))
        XCTAssertEqual(
            try! UnsignedBigInt(long: 5) * UnsignedBigInt(long: 3), UnsignedBigInt(long: 15))
        XCTAssertEqual(
            try! UnsignedBigInt(long: 5) % UnsignedBigInt(long: 3), UnsignedBigInt(long: 2))
        let modulus = UnsignedBigInt(long: 7)
        let a = UnsignedBigInt(long: 2)
        XCTAssertEqual(try! a.modMul(try! a.modInv(modulus), modulus), UnsignedBigInt(long: 1))
    }
}
