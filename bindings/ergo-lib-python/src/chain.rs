pub mod address;
mod ergo_box;
mod token;

use address::{Address, NetworkPrefix};
use ergo_box::ErgoBoxCandidate;
use pyo3::prelude::*;
use token::{Token, TokenId};
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<NetworkPrefix>()?;
    m.add_class::<Address>()?;
    m.add_class::<ErgoBoxCandidate>()?;
    m.add_class::<TokenId>()?;
    m.add_class::<Token>()?;
    Ok(())
}
