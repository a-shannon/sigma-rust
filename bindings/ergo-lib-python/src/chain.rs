pub mod address;
pub mod constant;
pub mod context_extension;
pub mod ergo_box;
pub mod ergo_state_context;
pub mod header;
pub mod parameters;
pub mod token;

use address::{Address, NetworkPrefix};
use constant::Constant;
use context_extension::ContextExtension;
use ergo_box::{BoxId, ErgoBox, ErgoBoxCandidate, NonMandatoryRegisterId};
use ergo_state_context::ErgoStateContext;
use header::{BlockId, Header, PreHeader};
use parameters::Parameters;
use pyo3::prelude::*;
use token::{Token, TokenId};
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<NetworkPrefix>()?;
    m.add_class::<Address>()?;
    m.add_class::<ErgoBoxCandidate>()?;
    m.add_class::<ErgoBox>()?;
    m.add_class::<BoxId>()?;
    m.add_class::<TokenId>()?;
    m.add_class::<Token>()?;
    m.add_class::<NonMandatoryRegisterId>()?;
    m.add_class::<Constant>()?;
    m.add_class::<BlockId>()?;
    m.add_class::<Header>()?;
    m.add_class::<PreHeader>()?;
    m.add_class::<ContextExtension>()?;
    m.add_class::<Parameters>()?;
    m.add_class::<ErgoStateContext>()?;
    Ok(())
}
