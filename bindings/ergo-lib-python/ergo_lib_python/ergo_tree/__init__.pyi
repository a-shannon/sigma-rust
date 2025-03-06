"""ErgoTree"""
import typing
from typing import Self
from ergo_lib_python.chain import Constant

__all__ = ['ErgoTree']
@typing.final
class ErgoTree:
    """The root of ErgoScript IR. Serialized instances of this class are self sufficient and can be passed around."""
    @classmethod
    def from_bytes(cls, b: bytes) -> Self:
        """Parse ErgoTree from bytes"""
    def constants(self) -> list[Constant]:
        """Return constants (as stored in serialized ErgoTree) or an exception if constant parsing failed"""
    def with_constant(self, index: int, constant: Constant) -> Self:
        """Set constant at index to constant. Returns an exception if ErgoTree was not parsed or constant type does not match"""
    def __bytes__(self):
        ...
