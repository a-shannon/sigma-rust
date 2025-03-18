import typing
from typing import Self, Any, Literal, Optional
from ergo_lib_python import transaction, multi_sig
from ergo_lib_python.sigma_protocol import ProveDlog
from ergo_lib_python.chain import Address, EcPoint, Token, ErgoBox, ErgoStateContext

__all__ = ['SecretKey', 'MnemonicGenerator', 'ExtSecretKey', 'ExtPubKey', 'DerivationPath', 'BoxSelection', 'Wallet', 'ErgoBoxAssetsData', 'select_boxes_simple', 'to_seed']
@typing.final
class DerivationPath:
    def __init__(self, acc: int = 0, address_indices: list[int] = [0]):
        """
        Create derivation path for a given account index (hardened) and address indices
        `m / 44' / 429' / acc' / 0 / address[0] / address[1] / ...`
        or `m / 44' / 429' / acc' / 0` (equivalent to `DerivationPath(0, [0])`) if address indices are empty

        change is always zero according to EIP-3
        acc is expected as a 31-bit value (32th bit should not be set)
        DerivationPath(0, [0])

        :param acc: account index (0 <= acc < 2**31)
        :param address_indices:
        """
        ...
    @classmethod
    def master_path(cls) -> Self:
        """Root derivation path (empty path)"""
        ...
    @classmethod
    def from_str(cls, path: str) -> Self:
        """Create a derivation path from a formatted string, e.g. `m / 44' / 429' / acc' / 0 / address[0] / address[1] / ...`
           :raises ValueError: if path is not valid
        """

    @property
    def depth(self) -> int:
        """Length of derivation path"""
        ...
    def next(self) -> Self:
        """Returns a new path with the last element of the deriviation path being increased, e.g. m/1/2 -> m/1/3
           :raises ValueError: if path is empty
        """
        ...
    def ledger_bytes(self) -> bytes:
        """For Sign Transaction command of Ergo Ledger App Protocol"""
        ...
    def __str__(self) -> str:
        """String representation of path E.g. m/44'/429'/0'/0/1"""
        ...

@typing.final
class ExtPubKey:
    """Extended public key implemented according to `BIP-32 <https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki>`_"""
    def __init__(self, public_key_bytes: bytes, chain_code: bytes, derivation_path: DerivationPath):
        ":param public_key_bytes: SEC-1 encoded compressed public key"
        ...
    def public_key(self) -> EcPoint:
        ...
    def address(self) -> Address:
        """Create P2PK address from currently derived public key"""
    def child(self, index: int) -> Self:
        """Soft derivation of the child public key with a given index

           :param index: Soft index (0 <= index <= 2**31)
           :raises ValueError: if index is out of bounds
        """
        ...
    def derive(self, up_path: DerivationPath) -> Self:
        """Derive a new extended public key from the derivation path
        :raises ValueError: if `up_path` includes hardened path or does not extend current derivation path"""
        ...

    @property
    def pub_key_bytes(self) -> bytes:
        """Return current public key as SEC-1 encoded compressed bytes"""
        ...
    @property
    def chain_code(self) -> bytes:
        ...
    @property
    def derivation_path(self) -> DerivationPath:
        """Derivation path"""
        ...

@typing.final
class SecretKey:
    """Secret Key type"""
    @classmethod
    def random_dlog(cls) -> Self:
       """Generate random DlogProverInput"""
       ...
    @classmethod
    def random_dht(cls) -> Self:
        """Generate random DhTupleProverInput"""
        ...
    @classmethod
    def from_json(cls, s: str | dict) -> Self:
        """Deserialize SecretKey from JSON"""
        ...
    @classmethod
    def from_bytes(cls, b: bytes) -> Self:
        """Parse SecretKey from bytes"""
    def public_image(self) -> ProveDlog | Any:
        """Return public image of SecretKey. If SecretKey is dlog then :py:class:`ProveDlog` is returned.
           Currently returning ProveDHTuple public keys is unsupported and will raise a ValueError.
        """
        ...
    def json(self) -> str:
        """Serialize SecretKey to JSON"""
        ...
    def __bytes__(self) -> bytes:
        ...

@typing.final
class ExtSecretKey:
    """Extended secret key implemented according to BIP-32
    """
    @classmethod
    def from_mnemonic(cls, mnemonic_phrase: str, password: str = ""):
        """Create ExtSecretKey from mnemonic
            :param mnemonic_phrase: seed phrase
            :param password: optional seed password
        """
        ...
    @classmethod
    def derive_master(cls, seed: bytes) -> Self:
        """Create new ExtSecretKey from seed.
        Example:

            >>> mnemonic = MnemonicGenerator("english", 128).generate()
            >>> seed = to_seed(mnemonic, password="")
            >>> ext_secret_key = ExtSecretKey.derive_master(seed)
            """
        ...
    def path(self) -> DerivationPath:
        """Derivation path associated with this ExtSecretKey"""
        ...
    def child(self, index: str) -> Self:
        """Derive a new extended secret key from the provided index
           The index is in the form of soft or hardened indices. For example: 4 or 4' respectively
        """
        ...
    def derive(self, up_path: DerivationPath) -> Self:
        """Derives new :py:class:`ExtSecretKey` from `up_path`"""
    def secret_key(self) -> SecretKey:
        """Return secret key"""
        ...
    def public_key(self) -> ExtPubKey:
        """Returns extended public key"""
        ...
    ...


@typing.final
class Wallet:
    """Wallet that can be used to generate proofs"""
    def __init__(self, secrets: list[SecretKey]):
        ...
    def add_secret(self, secret: SecretKey):
        """Add a secret to wallet"""
        ...
    def sign_transaction(self,
        tx: transaction.UnsignedTransaction | transaction.ReducedTransaction,
        boxes_to_spend: list[ErgoBox] = ...,
        data_boxes: list[ErgoBox] = ...,
        state_context: Optional[ErgoStateContext] = None,
        *,
        hints_bag: Optional[multi_sig.TransactionHintsBag] = None) -> transaction.Transaction:
            """Sign a transaction

               :param tx: Transaction to be signed
               :param boxes_to_spend: List of boxes to spend, this is not required if tx is a :py:class:`ReducedTransaction`
               :param data_boxes: List of data boxes, this is not required if tx is a :py:class:`ReducedTransaction`
               :param state_context: Current blockchain context, this is not required if tx is a :py:class:`ReducedTransaction`
               :param hints_bag: Hints to provide for multi-signature transactions
               :raises WalletException: if signing fails (secret not available, input script reduces to false, not enough boxes provided, etc)
            """
            ...
    def generate_commitments(self,
            tx: transaction.UnsignedTransaction | transaction.ReducedTransaction,
            boxes_to_spend: list[ErgoBox] = ...,
            data_boxes: list[ErgoBox] = ...,
            state_context: Optional[ErgoStateContext] = None
            ) -> multi_sig.TransactionHintsBag:
                """Generate commitments for a transaction. When signing a transaction/input that requires multiple signatures. The first step requires all signers participating to generate commitments

                   :param tx: Transaction to generate commitments for
                   :param boxes_to_spend: List of boxes to spend, this is not required if tx is a :py:class:`ReducedTransaction`
                   :param data_boxes: List of data boxes, this is not required if tx is a :py:class:`ReducedTransaction`
                   :param state_context: Current blockchain context, this is not required if tx is a :py:class:`ReducedTransaction`
                   :raises WalletException: if generating commitments fails(secret not available, input script reduces to false, not enough boxes provided, etc)

                """
                ...
    def sign_tx_input(self,
                tx: transaction.UnsignedTransaction,
                input_idx: int,
                boxes_to_spend: list[ErgoBox],
                data_boxes: list[ErgoBox],
                state_context: ErgoStateContext,
                *,
                hints_bag: Optional[multi_sig.TransactionHintsBag] = None
        ) -> transaction.Input:
            """Generate signatures for a single input

               :param tx: Transaction to sign
               :param input_idx: index of input (0-based) to sign
               :param boxes_to_spend: list of boxes to spend
               :param data_boxes: list of data boxes
               :param state_context: Current blockchain context
               :param hints_bag: Optional hints for generating signature
            """
            ...
    def sign_message_using_p2pk(self, address: Address, message: bytes) -> bytes:
        """Generate a signature for an arbitrary message

            Safety
            _______
                Signing (untrusted) arbitrary messages using this is not safe, since it may allow unintentionally signing a transaction
                Users of this function should take care to perform domain seperation, see EIP-44: https://github.com/ergoplatform/eips/blob/master/eip-0044.md

            :param address: P2PK address to generate signature for
            :param message: message to sign

            :raises ValueError: if address is not a p2pk address
            :raises WalletException: if signing fails
        """

@typing.final
class MnemonicGenerator:
    """Generator for mnemonic phrases"""
    def __init__(self, language: Literal["english", "chinese_simplified", "chinese_traditional", "french", "italian", "japanese", "korean", "spanish"] | str, strength: Literal[128, 160, 192, 224, 256] | int):
        """:param language: language to generate mnemonic in
           :param strength: strength of seed in bits
           :raises ValueError: if language or strength is not a valid value
        """
    def generate(self) -> str:
        """Generate mnemonic sentence using randomly-generated entropy"""
        ...
    def from_entropy(self, entropy: bytes) -> str:
        """Generate mnemonic using user-provided entropy

           :raises ValueError: if entropy length is not valid
        """
        ...
def to_seed(mnemonic_phrase: str, password: str = "") -> bytes:
    """Convert a mnemonic into a 512-bit seed"""
    ...

@typing.final
class ErgoBoxAssetsData:
    def __init__(self, value: int, tokens: list[Token]):
        ...
    @property
    def value(self) -> int:
        ...
    @property
    def tokens(self) -> list[Token]:
        ...

@typing.final
class BoxSelection:
    """Represents a list of boxes selected that satisfy a target value (in ergs) and target tokens. See: :py:func:`select_boxes_simple`"""
    def __init__(self, boxes: list[ErgoBox], change_boxes: list[ErgoBoxAssetsData]):
        """Build a new box selection

        :param boxes: list of boxes that are selected
        :param change_boxes: List of change boxes, that will be sent to change address when building a Transaction using this selection
        :raises ValueError: if boxes are empty or greater than max allowed (32767)
        """
    @property
    def boxes(self) -> list[ErgoBox]:
        ...
    @property
    def change_boxes(self) -> list[ErgoBoxAssetsData]:
        ...

def select_boxes_simple(inputs: list[ErgoBox], target_balance: int, target_tokens: list[Token]) -> BoxSelection:
    """Select boxes from `inputs` whose value sums to `target_balance` and tokens add up to `target_tokens`
       This uses a simple strategy of sorting inputs by target assets and selecting inputs until target balance is reached

       :param inputs: list of boxes to select from
       :param target_balance: Target balance (in nanoERGs) to reach
       :param target_tokens: List of tokens to reach
       :raises ValueError: if target_balance or target tokens cannot be satisfied by inputs
    """
