import unittest
from ergo_lib_python.chain import Constant, SType, ErgoBox

class TestConstant(unittest.TestCase):
    def test_i32(self):
        c = Constant.from_i32(999)
        self.assertEqual(c.tpe, SType.SInt())
        encoded = bytes(c)
        decoded = Constant.from_bytes(encoded)
        self.assertEqual(c, decoded)
        with self.assertRaises(OverflowError):
            Constant.from_i32(2 ** 31)
        with self.assertRaises(OverflowError):
            Constant.from_i32(-(2 ** 31) - 1)

    def test_i64(self):
        c = Constant.from_i64(999)
        self.assertEqual(c.tpe, SType.SLong())
        encoded: bytes = bytes(c)
        decoded = Constant.from_bytes(encoded)
        self.assertEqual(c, decoded)
        with self.assertRaises(OverflowError):
            Constant.from_i64(2 ** 63)
        with self.assertRaises(OverflowError):
            Constant.from_i32(-(2 ** 63) - 1)
    def test_bytes(self):
        b = bytes([1, 1, 2, 255])
        c = Constant(b)
        self.assertEqual(c.tpe, SType.SColl(SType.SByte()))
        encoded = bytes(c)
        decoded = Constant.from_bytes(encoded)
        self.assertEqual(c, decoded)
    def test_tuple(self):
        box = ErgoBox.from_json("""
                    {
                      "boxId": "e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e",
                      "value": 67500000000,
                      "ergoTree": "100204a00b08cd021dde34603426402615658f1d970cfa7c7bd92ac81a8b16eeebff264d59ce4604ea02d192a39a8cc7a70173007301",
                      "assets": [],
                      "creationHeight": 284761,
                      "additionalRegisters": {},
                      "transactionId": "9148408c04c2e38a6402a7950d6157730fa7d49e9ab3b9cadec481d7769918e9",
                      "index": 1
                    }
                """)
        c = Constant((Constant.from_i32(1), Constant(box), Constant(bytes([1, 2]))))
        self.assertEqual(c.value, (Constant.from_i32(1), Constant(box), Constant(bytes([1, 2]))))
        self.assertEqual(c.tpe, SType.STuple((SType.SInt(), SType.SBox(), SType.SColl(SType.SByte()))))
        #test that pattern matching works
        match c.tpe:
            case SType.STuple((SType.SInt(), SType.SBox(), SType.SColl(SType.SByte()))):
                ...
            case _:
                assert False
        encoded = bytes(c)
        decoded = Constant.from_bytes(encoded)
        self.assertEqual(c, decoded)
