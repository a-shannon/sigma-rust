use derive_more::{From, Into};
use ergo_lib::ergotree_ir::{
    mir::constant::{self, Literal},
    serialization::SigmaSerializable,
    types::stuple::STuple,
};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyFloat, PyInt, PyTuple},
};

use crate::{errors::SigmaSerializationError, to_value_error};
/// Constant value that can be used in ErgoBox registers, ErgoTree constants and ContextExtension
#[pyclass(eq)]
#[derive(PartialEq, Eq, Clone, Debug, From, Into)]
pub struct Constant(constant::Constant);

#[pymethods]
impl Constant {
    #[new]
    fn new(arg: &Bound<'_, PyAny>) -> PyResult<Self> {
        if arg.is_exact_instance_of::<PyInt>() | arg.is_exact_instance_of::<PyFloat>() {
            return Err(PyValueError::new_err("Constant.new does not support numeric type as argument. Use Constant.from_i64, from_i32, etc instead"));
        }
        match arg.extract::<&[u8]>() {
            Ok(byte_array) => Ok(Self(constant::Constant::from(byte_array.to_owned()))),
            Err(e) => match arg.downcast_exact::<PyTuple>() {
                Ok(tuple) => from_tuple(tuple),
                Err(e) => match arg.extract::<Vec<Constant>>() {
                    Ok(arr) => Ok(Self(
                        constant::Constant::coll_from_iter(
                            arr.into_iter().map(|constant| constant.0),
                        )
                        .map_err(to_value_error)?,
                    )),
                    Err(e) => Err(PyValueError::new_err(
                        "Constant.new expected bytes, array of Constants, or tuple of Constants",
                    )),
                },
            },
        }
    }
    #[staticmethod]
    fn from_i64(v: i64) -> Constant {
        Constant(constant::Constant::from(v))
    }
    #[staticmethod]
    fn from_i32(v: i32) -> Constant {
        Constant(constant::Constant::from(v))
    }
    #[staticmethod]
    fn from_i16(v: i16) -> Constant {
        Constant(constant::Constant::from(v))
    }
    #[staticmethod]
    fn from_i8(v: i8) -> Constant {
        Constant(constant::Constant::from(v))
    }

    /// Serialize Constant as byte array
    fn __bytes__(&self) -> PyResult<Vec<u8>> {
        self.0
            .sigma_serialize_bytes()
            .map_err(SigmaSerializationError::from)
            .map_err(Into::into)
    }
    /// Parse serialized Constant from byte-array
    #[staticmethod]
    fn from_bytes(bytes: &[u8]) -> PyResult<Self> {
        constant::Constant::sigma_parse_bytes(bytes)
            .map(Self)
            .map_err(to_value_error)
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

fn from_tuple(tuple: &Bound<'_, PyTuple>) -> PyResult<Constant> {
    let mut tpes = vec![];
    let mut items = vec![];
    for item in tuple.iter() {
        let Constant(constant::Constant { tpe, v }) = item.extract::<Constant>()?;
        tpes.push(tpe);
        items.push(v);
    }
    Ok(Constant(constant::Constant {
        tpe: STuple::try_from(tpes).map_err(to_value_error)?.into(),
        v: Literal::Tup(items.try_into().map_err(to_value_error)?),
    }))
}
