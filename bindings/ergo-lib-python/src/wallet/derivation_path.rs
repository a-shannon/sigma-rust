use derive_more::{From, Into};
use ergo_lib::wallet::derivation_path::{
    self, ChildIndexError, ChildIndexHardened, ChildIndexNormal,
};
use pyo3::{pyclass, pymethods, PyResult};

use crate::to_value_error;

/// According to
/// BIP-44 <https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki>
/// and EIP-3 <https://github.com/ergoplatform/eips/blob/master/eip-0003.md>
#[pyclass(frozen, eq)]
#[derive(PartialEq, Eq, Debug, Clone, From, Into)]
pub struct DerivationPath(pub(crate) derivation_path::DerivationPath);

#[pymethods]
impl DerivationPath {
    /// Create derivation path for a given account index (hardened) and address indices
    /// `m / 44' / 429' / acc' / 0 / address[0] / address[1] / ...`
    /// or `m / 44' / 429' / acc' / 0` if address indices are empty
    /// change is always zero according to EIP-3
    /// acc is expected as a 31-bit value (32th bit should not be set)
    #[new]
    #[pyo3(signature = (acc=0, address_indices=vec![0]), text_signature = "(acc=0, address_indices=[0])")]
    pub fn new(acc: u32, address_indices: Vec<u32>) -> PyResult<DerivationPath> {
        let acc = ChildIndexHardened::from_31_bit(acc).map_err(to_value_error)?;
        let address_indices = address_indices
            .iter()
            .map(|i| ChildIndexNormal::normal(*i))
            .collect::<Result<Vec<ChildIndexNormal>, ChildIndexError>>()
            .map_err(to_value_error)?;
        Ok(Self(derivation_path::DerivationPath::new(
            acc,
            address_indices,
        )))
    }

    /// Create root derivation path
    #[staticmethod]
    pub fn master_path() -> Self {
        Self(derivation_path::DerivationPath::master_path())
    }

    /// Returns the length of the derivation path
    pub fn depth(&self) -> usize {
        self.0.depth()
    }

    /// Returns a new path with the last element of the deriviation path being increased, e.g. m/1/2 -> m/1/3
    /// Returns an empty path error if the path is empty (master node)
    pub fn next(&self) -> PyResult<DerivationPath> {
        Ok(Self(self.0.next().map_err(to_value_error)?))
    }

    /// String representation of derivation path
    /// E.g m/44'/429'/0'/0/1
    pub fn __str__(&self) -> String {
        self.0.to_string()
    }

    /// Create a derivation path from a formatted string
    /// E.g "m/44'/429'/0'/0/1"
    #[staticmethod]
    pub fn from_str(path: &str) -> PyResult<DerivationPath> {
        Ok(Self(
            path.parse::<derivation_path::DerivationPath>()
                .map_err(to_value_error)?,
        ))
    }

    /// For 0x21 Sign Transaction command of Ergo Ledger App Protocol
    /// P2PK Sign (0x0D) instruction
    /// Sign calculated TX hash with private key for provided BIP44 path.
    /// Data:
    ///
    /// Field
    /// Size (B)
    /// Description
    ///
    /// BIP32 path length
    /// 1
    /// Value: 0x02-0x0A (2-10). Number of path components
    ///
    /// First derivation index
    /// 4
    /// Big-endian. Value: 44’
    ///
    /// Second derivation index
    /// 4
    /// Big-endian. Value: 429’ (Ergo coin id)
    ///
    /// Optional Third index
    /// 4
    /// Big-endian. Any valid bip44 hardened value.
    /// ...
    /// Optional Last index
    /// 4
    /// Big-endian. Any valid bip44 value.
    ///
    pub fn ledger_bytes(&self) -> Vec<u8> {
        self.0.ledger_bytes()
    }
}
