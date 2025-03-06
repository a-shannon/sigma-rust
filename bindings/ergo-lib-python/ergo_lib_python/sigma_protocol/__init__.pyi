import typing
from ergo_lib_python import chain
__all__ = ['ProveDlog']
@typing.final
class ProveDlog:
    """
    Value representing public key of discrete logarithm signature protocol.
    """
    def __init__(self, ec_point: chain.EcPoint):
        ...
    @property
    def h(self) -> chain.EcPoint:
        """Public key of proposition"""
        ...
