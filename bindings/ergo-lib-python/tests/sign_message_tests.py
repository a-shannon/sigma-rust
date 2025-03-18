import unittest
from ergo_lib_python.wallet import SecretKey, Wallet
from ergo_lib_python.chain import Address
from ergo_lib_python.verifier import verify_signature
import random

class SignMessageTests(unittest.TestCase):
    def test_sign_message(self):
        secret_key = SecretKey.random_dlog()
        address = Address.p2pk(secret_key.public_image())
        wallet = Wallet([secret_key])
        message = b"Hello World!"
        signature = wallet.sign_message_using_p2pk(address, message)
        self.assertTrue(verify_signature(address, message, signature))
        self.assertFalse(verify_signature(address, b"other message", signature))
        fake_signature = random.randbytes(len(signature))
        self.assertFalse(verify_signature(address, message, fake_signature))
        other_address = Address.p2pk(SecretKey.random_dlog().public_image())
        self.assertFalse(verify_signature(other_address, message, signature))
