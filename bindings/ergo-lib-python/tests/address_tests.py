import unittest
from ergo_lib_python.chain import Address, NetworkPrefix

class AddressTests(unittest.TestCase):
    def test_from_str(self):
        testnet_address_str = "3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN"
        testnet_address = Address(testnet_address_str)
        self.assertEqual(testnet_address.to_str(NetworkPrefix.Testnet), testnet_address_str)
        with self.assertRaises(ValueError):
            Address(testnet_address_str, network_prefix=NetworkPrefix.Mainnet)
        tree = testnet_address.ergo_tree()
        self.assertEqual(Address.recreate_from_ergo_tree(tree), testnet_address)
