"""Implementation of Non-Interactive Proof of Proof of Work based on [KMZ17]_

.. [KMZ17] https://fc20.ifca.ai/preproceedings/74.pdf"""
import typing
from typing import Optional, Self
from ergo_lib_python import chain

__all__ = ['NipopowProof', 'NipopowVerifier', 'PoPowHeader']
@typing.final
class PoPowHeader:
    """Represents the block header and unpacked interlinks"""
    @classmethod
    def from_json(cls, json: dict | str) -> Self:
        """Deserialize :py:class`PoPowHeader` from JSON
            :raises JsonException: if parsing fails
        """
        ...
    def json(self) -> str:
        """Serialize to JSON"""
    @property
    def header(self) -> chain.Header:
        ...
    @property
    def interlinks(self) -> list[chain.BlockId]:
        ...
    def check_interlinks_proof(self) -> bool:
        ...
@typing.final
class NipopowProof:
    """Structure representing a NiPoPow proof"""
    @classmethod
    def from_json(cls, json: dict | str) -> Self:
        """Deserialize :py:class:`NipopowProof` from JSON

           :param json:
           :raises JsonException: if parsing fails
        """
        ...
    def is_better_than(self, that: Self) -> bool:
        """Implementation of the >= algorithm from [KMZ17]_, algorithm 4"""
        ...
    @property
    def suffix_head(self) -> PoPowHeader:
        """First header of suffix"""
        ...
    def json(self) -> str:
        """Serialize to JSON"""
        ...

@typing.final
class NipopowVerifier:
    """A verifier for PoPoW proofs. During its lifetime, it processes many proofs with the aim of deducing what is the best (subchain) rooted at the specific genesis"""

    def __init__(self, genesis_block_id: chain.BlockId):
        ...
    def best_proof(self) -> Optional[NipopowProof]:
        """Return best proof verifier has received so far"""
        ...
    def best_chain(self) -> list[chain.Header]:
        """Return list of headers that are part of proof"""
        ...
    def process(self, new_proof: NipopowProof):
        """Process a new :py:class:`NipopowProof`, and update best chain if `new_proof` is better"""
        ...
