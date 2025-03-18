pub mod address;
pub mod constant;
pub mod context_extension;
pub mod ec_point;
pub(crate) mod ergo_box;
pub mod ergo_state_context;
pub mod header;
pub mod parameters;
pub mod token;

use address::{Address, NetworkPrefix};
use constant::{Constant, SType};
use context_extension::ContextExtension;
use ec_point::EcPoint;
use ergo_box::{BoxId, ErgoBox, ErgoBoxCandidate, NonMandatoryRegisterId, NonMandatoryRegisters};
use ergo_state_context::ErgoStateContext;
use header::{BlockId, Header, PreHeader};
use parameters::Parameters;
use pyo3::prelude::*;
use token::{Token, TokenId};

pub(crate) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let submodule = PyModule::new(m.py(), "chain")?;
    submodule.add_class::<NetworkPrefix>()?;
    submodule.add_class::<Address>()?;
    submodule.add_class::<EcPoint>()?;
    submodule.add_class::<ErgoBoxCandidate>()?;
    submodule.add_class::<ErgoBox>()?;
    submodule.add_class::<BoxId>()?;
    submodule.add_class::<TokenId>()?;
    submodule.add_class::<Token>()?;
    submodule.add_class::<NonMandatoryRegisterId>()?;
    submodule.add_class::<NonMandatoryRegisters>()?;
    submodule.add_class::<Constant>()?;
    submodule.add_class::<SType>()?;
    submodule.add_class::<BlockId>()?;
    submodule.add_class::<Header>()?;
    submodule.add_class::<PreHeader>()?;
    submodule.add_class::<ContextExtension>()?;
    submodule.add_class::<Parameters>()?;
    submodule.add_class::<ErgoStateContext>()?;
    m.add_submodule(&submodule)?;
    submodule
        .py()
        .import("sys")?
        .getattr("modules")?
        .set_item("ergo_lib_python.chain", submodule)?;
    Ok(())
}
