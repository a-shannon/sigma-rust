import typing
from typing import Optional, Self
from ergo_lib_python.chain import BoxId, ContextExtension, ErgoBox, ErgoBoxCandidate, ErgoStateContext, Address, Token
from ergo_lib_python.wallet import BoxSelection
__all__ = ['UnsignedInput', 'Input', 'DataInput', 'ProverResult', 'UnsignedTransaction', 'Transaction', 'ReducedTransaction', 'TxBuilder', 'TxId']

@typing.final
class TxId:
    """Identifier of :py:class:`Transaction`"""
    def __init__(self, val: str | bytes):
        """
        :param val: base-16 encoded str or raw bytes
        """
        ...
    def __bytes__(self) -> bytes:
        ...
    ...

@typing.final
class UnsignedInput:
    """Unsigned (without proofs) transaction input"""
    def __init__(self, box_id: BoxId, ext: Optional[ContextExtension] = None):
        """:param box_id: id of the box to be spent
           :param ext: user-defined variables to be put into context
        """
    @property
    def box_id(self) -> BoxId:
        ...
    @property
    def context_extension(self) -> ContextExtension:
        ...

@typing.final
class ProverResult:
    """Represents result of prover (signatures, if required)"""
    @classmethod
    def from_bytes(cls, b: bytes) -> Self:
        """raises: SigmaParsingException: if parsing fails"""
        ...
    @property
    def proof(self) -> bytes:
        ...
    @property
    def extension(self) -> ContextExtension:
        ...
    def json(self) -> str:
        ...
    def __bytes__(self) -> bytes:
        ...

    ...
@typing.final
class Input:
    """Signed transaction input"""
    def __init__(self, box_id: BoxId, spending_proof: ProverResult):
        ...
    @classmethod
    def from_unsigned_input(cls, unsigned_input: UnsignedInput, proof_bytes: bytes) -> Self:
        ...
    @property
    def box_id(self) -> BoxId:
        ...
    @property
    def spending_proof(self) -> ProverResult:
        ...
    def __bytes__(self) -> bytes:
        ...

@typing.final
class DataInput:
    """Read-only inputs that won't be spent by the transaction"""
    def __init__(self, box_id: BoxId):
        ...
    @property
    def box_id(self) -> BoxId:
        ...

@typing.final
class UnsignedTransaction:
    """Unsigned (inputs without proofs) transaction"""
    def __init__(self, inputs: list[UnsignedInput], data_inputs: list[DataInput], output_candidates: list[ErgoBoxCandidate]):
        """:raises ValueError: if inputs or output_candidates are empty or greater than maximum allowed (32767)"""
        ...
    @property
    def id(self) -> TxId:
        ...
    @property
    def inputs(self) -> list[UnsignedInput]:
        ...
    @property
    def data_inputs(self) -> list[DataInput]:
        ...
    @property
    def output_candidates(self) -> list[ErgoBoxCandidate]:
        ...
    @property
    def outputs(self) -> list[ErgoBox]:
        ...
    @classmethod
    def from_json(cls, s: str | dict) -> Self:
        ...
    def json(self) -> str:
        ...

@typing.final
class Transaction:
    """Signed Transaction type"""
    def __init__(self, inputs: list[Input], data_inputs: list[DataInput], output_candidates: list[ErgoBoxCandidate]):
        ...
    @classmethod
    def from_unsigned_tx(cls, unsigned_tx: UnsignedTransaction, proofs: list[bytes]) -> Self:
        """Create :py:class:`Transaction` from `UnsignedTransaction`

           :param unsigned_tx: UnsignedTransaction
           :param proofs: list of proofs (serialized bytes) for each input
           :raises ValueError: if `len(proofs) != len(unsigned_tx.inputs)`
        """
        ...
    @classmethod
    def from_json(cls, json: str | dict) -> Self:
        """:raises JsonException: if deserialization fails"""
        ...
    @classmethod
    def from_bytes(cls, b: bytes) -> Self:
        """:raises SigmaParsingException: if deserialization fails"""
        ...
    @property
    def id(self) -> TxId:
        ...
    @property
    def inputs(self) -> list[Input]:
        ...
    @property
    def data_inputs(self) -> list[DataInput]:
        ...
    @property
    def output_candidates(self) -> list[ErgoBoxCandidate]:
        ...
    @property
    def outputs(self) -> list[ErgoBox]:
        ...
    def __bytes__(self) -> bytes:
        ...
    def json(self) -> str:
        ...

@typing.final
class ReducedTransaction:
    """
    Represent `reduced` transaction, i.e. unsigned transaction where each unsigned input is augmented with ReducedInput which contains a script reduction result.
    After an unsigned transaction is reduced it can be signed without context.
    Thus, it can be serialized and transferred for example to Cold Wallet and signed
    in an environment where secrets are known.
    see EIP-19 for more details - <https://github.com/ergoplatform/eips/blob/f280890a4163f2f2e988a0091c078e36912fc531/eip-0019.md>
    """
    @classmethod
    def from_unsigned_tx(cls, unsigned_tx: UnsignedTransaction, boxes_to_spend: list[ErgoBox], data_boxes: list[ErgoBox], state_context: ErgoStateContext) -> Self:
        """Reduce unsigned_tx

           :raises ValueError: if box for an input or data input is missing
           :raises WalletException: if reduction fails
        """
        ...
    @classmethod
    def from_bytes(cls, b: bytes) -> Self:
        ...
    def __bytes__(self) -> bytes:
        ...
    def unsigned_tx(self) -> UnsignedTransaction:
        ...

@typing.final
class TxBuilder:
    """Builder for a :py:class:`UnsignedTransaction`"""
    def __init__(self, box_selection: BoxSelection, output_candidates: list[ErgoBoxCandidate], current_height: int, fee_amount: int, change_address: Address):
        """:param box_selection: list of boxes to be spent, and change boxes to be created
           :param output_candidates: list of outputs to be created
           :param current_height: chain height that will be used in additional created boxes (change boxes, fee box)
           :param fee_amount: miner fee (higher values will speed up inclusion time)
           :param change_address: change (input value sum - output value sum) will be sent to this address
        """
    def set_data_inputs(self, data_inputs: list[DataInput]):
        """Set data inputs"""
    def set_context_extension(self, box_id: BoxId, context_extension: ContextExtension):
        """Set input `box_id`'s context extension"""
    def set_token_burn_permit(self, tokens: list[Token]):
        """List of tokens that are allowed to be burned"""
    def build(self) -> UnsignedTransaction:
        """
        Build transaction

        :raises ValueError: if building tx fails
        """
        ...
