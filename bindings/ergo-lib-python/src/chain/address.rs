use derive_more::From;
use ergo_lib::ergotree_ir::chain::address::{self, AddressEncoder};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyDict};

use crate::{ergo_tree::ErgoTree, to_value_error};
#[pyclass(eq, frozen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkPrefix {
    Mainnet = 0x00,
    Testnet = 0x10,
}

impl From<NetworkPrefix> for address::NetworkPrefix {
    fn from(value: NetworkPrefix) -> Self {
        #[allow(clippy::unwrap_used)] // All variants of python NetworkPrefix are valid
        address::NetworkPrefix::try_from(value as u8).unwrap()
    }
}

/**
 * An address is a short string corresponding to some script used to protect a box. Unlike (string-encoded) binary
 * representation of a script, an address has some useful characteristics:
 *
 * - Integrity of an address could be checked., as it is incorporating a checksum.
 * - A prefix of address is showing network and an address type.
 * - An address is using an encoding (namely, Base58) which is avoiding similarly l0Oking characters, friendly to
 * double-clicking and line-breaking in emails.
 *
 *
 *
 * An address is encoding network type, address type, checksum, and enough information to watch for a particular scripts.
 *
 * Possible network types are:
 * Mainnet - 0x00
 * Testnet - 0x10
 *
 * For an address type, we form content bytes as follows:
 *
 * P2PK - serialized (compressed) public key
 * P2SH - first 192 bits of the Blake2b256 hash of serialized script bytes
 * P2S  - serialized script
 *
 * Address examples for testnet:
 *
 * 3   - P2PK (3WvsT2Gm4EpsM9Pg18PdY6XyhNNMqXDsvJTbbf6ihLvAmSb7u5RN)
 * ?   - P2SH (rbcrmKEYduUvADj9Ts3dSVSG27h54pgrq5fPuwB)
 * ?   - P2S (Ms7smJwLGbUAjuWQ)
 *
 * for mainnet:
 *
 * 9  - P2PK (9fRAWhdxEsTcdb8PhGNrZfwqa65zfkuYHAMmkQLcic1gdLSV5vA)
 * ?  - P2SH (8UApt8czfFVuTgQmMwtsRBZ4nfWquNiSwCWUjMg)
 * ?  - P2S (4MQyML64GnzMxZgm, BxKBaHkvrTvLZrDcZjcsxsF7aSsrN73ijeFZXtbj4CXZHHcvBtqSxQ)
 */
#[pyclass(eq, frozen)]
#[derive(From, Clone, PartialEq, Eq)]
pub struct Address(pub(crate) address::Address);

#[pymethods]
impl Address {
    /// Build a new address from a str, ErgoTree or bytes
    #[new]
    #[pyo3(signature = (**kwds))]
    fn new(kwds: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        let kwds =
            kwds.ok_or_else(|| PyValueError::new_err("No arguments given to Address.new"))?;

        let encoder = if let Some(prefix) = kwds.get_item("network_prefix")? {
            Some(AddressEncoder::new(address::NetworkPrefix::from(
                prefix.extract::<NetworkPrefix>()?,
            )))
        } else {
            None
        };
        match kwds.get_item("str")? {
            Some(s) => {
                let s = s.extract::<&str>()?;
                if let Some(encoder) = encoder {
                    encoder
                        .parse_address_from_str(s)
                        .map_err(to_value_error)
                        .map(Self)
                } else {
                    AddressEncoder::unchecked_parse_address_from_str(s)
                        .map_err(to_value_error)
                        .map(Self)
                }
            }
            None => match kwds.get_item("bytes")? {
                Some(bytes) => {
                    AddressEncoder::unchecked_parse_address_from_bytes(bytes.extract::<&[u8]>()?)
                        .map_err(to_value_error)
                        .map(Self)
                }
                None => Err(PyValueError::new_err("expected str= or bytes= argument")),
            },
        }
    }
    /// Re-create the address from ErgoTree that was built from the address
    /// This is the inverse of Address.ergo_tree()
    #[staticmethod]
    fn recreate_from_ergo_tree(tree: &ErgoTree) -> PyResult<Self> {
        address::Address::recreate_from_ergo_tree(&tree.0)
            .map(Self)
            .map_err(to_value_error)
    }
    /// Create an ErgoTree script from the address
    fn ergo_tree(&self) -> PyResult<ErgoTree> {
        self.0.script().map(Into::into).map_err(to_value_error)
    }

    #[pyo3(signature = (network_prefix=NetworkPrefix::Mainnet))]
    fn to_str(&self, network_prefix: NetworkPrefix) -> String {
        AddressEncoder::new(network_prefix.into()).address_to_str(&self.0)
    }
}

#[cfg(test)]
mod test {
    use super::NetworkPrefix;
    #[test]
    fn eq_network_prefix() {
        assert_eq!(
            NetworkPrefix::Testnet as u8,
            ergo_lib::ergotree_ir::chain::address::NetworkPrefix::Testnet as u8
        );
        assert_eq!(
            NetworkPrefix::Mainnet as u8,
            ergo_lib::ergotree_ir::chain::address::NetworkPrefix::Mainnet as u8
        );
    }
}
