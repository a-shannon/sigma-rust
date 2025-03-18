from ergo_lib_python import chain
__all__ = ['verify_signature']
def verify_signature(address: chain.Address, message: bytes, signature: bytes) -> bool:
    """Verify that `signature` is a valid signature from `address` for `message`

       :raises: ValueError: if address is not P2PK or verifier error occurs
    """
    ...
