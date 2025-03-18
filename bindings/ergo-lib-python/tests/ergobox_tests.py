import unittest
from ergo_lib_python.chain import BoxId, ErgoBox
import json

class ErgoBoxTests(unittest.TestCase):
    def test_json(self):
        box_json = """
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
        """
        box = ErgoBox.from_json(box_json)
        # test equivalence between from_json(str) and from_json(dict)
        self.assertEqual(box, ErgoBox.from_json(json.loads(box_json)))
        self.assertEqual(box.box_id, BoxId("e56847ed19b3dc6b72828fcfb992fdf7310828cf291221269b7ffc72fd66706e"))
        self.assertEqual(box.creation_height, 284761)
        self.assertEqual(box.value, 67500000000)
