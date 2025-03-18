import unittest
import base64
from ergo_lib_python.chain import TokenId, BoxId

class TokenTests(unittest.TestCase):
    id_str = "19475d9a78377ff0f36e9826cec439727bea522f6ffa3bda32e20d2f8b3103ac"
    def test_from_str(self):
        id = TokenId(self.id_str)
        self.assertEqual(bytes(id), base64.b16decode(self.id_str, casefold=True))
    def test_from_box_id(self):
        box_id = BoxId(self.id_str)
        self.assertEqual(bytes(box_id), bytes(TokenId.from_box_id(box_id)))
