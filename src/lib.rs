use msyt::{Endianness, Msyt};
use pyo3::{
    create_exception,
    exceptions::PyException,
    prelude::*,
    types::{PyDict, PyString},
};

create_exception!(pymsyt, MsytError, PyException);

#[pymodule]
fn pymsyt(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Msbt>()?;
    Ok(())
}

#[pyclass]
/// Class representing an MSBT file. This is just a thin wrapper over the `Msyt` type from
/// the Rust MSYT project providing a few convenient methods for Python use. Note that
/// manipulating the contents of an MSBT in code can only be done staightforwardly by converting
/// with `to_dict()` and `from_dict()`. This class cannot be directly instantiated. Instead,
/// create it through static parsing methods. Example use:
/// ```python
/// from pymsyt import Msbt
/// data = open("ArmorHead.msbt", "rb").read()
/// msbt = Msbt.from_binary(data)
/// msyt_text = msbt.to_yaml() # Convert MSBT to MSYT YAML
/// json_text = msbt.to_json() # Convert MSBT to JSON
/// msbt_dict = msbt.to_dict() # Convert to an editable Python dictionary
/// for entry, contents in msbt_dict["entries"].items() # Iterate MSBT text entries
///     print(f"{entry} = {contents}")
/// msbt_dict["entries"]["Armor_999_Head"] = { # Adding a new text entry
///     "contents": [{"text":"Some new helmet"}]
/// }
/// open("ArmorHead.msbt", "wb").write( # Saving modified file
///     Msbt.from_dict(msbt_dict).to_binary(big_endian=True)
/// )
/// ```
pub struct Msbt {
    msyt: Msyt,
}

#[pymethods]
impl Msbt {
    /// Parses an MSBT file from a byteslike object
    ///
    /// :param data: The bytes of the MSBT file to parse.
    /// :type data: byteslike
    /// :return: Returns a parsed `pymsyt.Msbt` class representing the MSBT file.
    /// :rtype: `pymsyt.Msbt`
    /// :raises MsytError: Raises an `MsytError` if parsing fails.
    #[staticmethod]
    #[text_signature = "(data, /)"]
    pub fn from_binary(data: &[u8]) -> PyResult<Self> {
        let msyt = Msyt::from_msbt_bytes(data)
            .map_err(|e| MsytError::new_err(format!("Failed to parse MSBT file: {:?}", e)))?;
        Ok(Msbt { msyt })
    }

    /// Serializes this MSBT file to bytes.
    ///
    /// :param big_endian: Whether to serialize as big endian (Wii U) or little endian (Switch)
    /// :type big_endian: bool, optional
    /// :return: Returns the MSBT file as a bytes object.
    /// :rtype: bytes
    /// :raises MsytError: Raises an `MsytError` if serialization fails.
    #[text_signature = "(big_endian, /)"]
    pub fn to_binary(&self, big_endian: bool) -> PyResult<Vec<u8>> {
        Ok(self
            .msyt
            .clone()
            .into_msbt_bytes(match big_endian {
                true => Endianness::Big,
                false => Endianness::Little,
            })
            .map_err(|e| MsytError::new_err(format!("Failed to serialize MSBT file: {:?}", e)))?)
    }

    /// Generates a YAML representation of this MSBT file.
    ///
    /// :return: Returns the MSBT as a YAML string.
    /// :rtype: str
    /// :raises MsytError: Raises an `MsytError` if serialization fails.
    #[text_signature = "()"]
    pub fn to_yaml(&self) -> PyResult<String> {
        Ok(serde_yaml::to_string(&self.msyt)
            .map_err(|e| MsytError::new_err(format!("Failed to dump MSBT to YAML: {:?}", e)))?)
    }

    /// Generates a JSON representation of this MSBT file.
    ///
    /// :return: Returns the MSBT as a JSON string.
    /// :rtype: str
    /// :raises MsytError: Raises an `MsytError` if serialization fails.
    #[text_signature = "()"]
    pub fn to_json(&self) -> PyResult<String> {
        Ok(serde_json::to_string(&self.msyt)
            .map_err(|e| MsytError::new_err(format!("Failed to dump MSBT to JSON: {:?}", e)))?)
    }

    /// Parses an MSBT file from a YAML representation.
    ///
    /// :param yaml: The text of the YAML to parse.
    /// :type yaml: str
    /// :return: Returns a parsed `pymsyt.Msbt` from the YAML text.
    /// :rtype: `pymsyt.Msbt`
    /// :raises MsytError: Raises an `MsytError` if parsing fails.
    #[staticmethod]
    #[text_signature = "(yaml, /)"]
    pub fn from_yaml(yaml: String) -> PyResult<Self> {
        Ok(Self {
            msyt: serde_yaml::from_str(&yaml).map_err(|e| {
                MsytError::new_err(format!("Could not parse YAML to MSBT: {:?}", e))
            })?,
        })
    }

    /// Parses an MSBT file from a JSON representation.
    ///
    /// :param json: The text of the JSON to parse.
    /// :type json: str
    /// :return: Returns a parsed `pymsyt.Msbt` from the JSON text.
    /// :rtype: `pymsyt.Msbt`
    /// :raises MsytError: Raises an `MsytError` if parsing fails.
    #[staticmethod]
    #[text_signature = "(json, /)"]
    pub fn from_json(json: String) -> PyResult<Self> {
        Ok(Self {
            msyt: serde_json::from_str(&json).map_err(|e| {
                MsytError::new_err(format!("Could not parse JSON to MSBT: {:?}", e))
            })?,
        })
    }

    /// Converts the MSBT contents to a Python dict.
    ///
    /// :return: Returns the MSBT as a Python dict.
    /// :rtype: dict
    /// :raises MsytError: Raises an `MsytError` if conversion fails.
    #[text_signature = "()"]
    pub fn to_dict(&self) -> PyResult<Py<PyAny>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let text = self.to_json()?;
        let json = PyModule::import(py, "json")?;
        let dict = json.call("loads", (text,), None).map_err(|e| {
            MsytError::new_err(format!("Could not serialize MSBT to Python dict: {:?}", e))
        })?;
        Ok(Py::from(dict))
    }

    /// Parses an MSBT file from a Python dictionary.
    ///
    /// :param dict: The Python dictionary to parse.
    /// :type dict: dict
    /// :return: Returns a parsed `pymsyt.Msbt` from the Python dict.
    /// :rtype: `pymsyt.Msbt`
    /// :raises MsytError: Raises an `MsytError` if parsing fails.
    #[staticmethod]
    #[text_signature = "(dict, /)"]
    pub fn from_dict(dict: &PyDict) -> PyResult<Self> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let json = PyModule::import(py, "json")?;
        let res = json.call("dumps", (dict,), None)?;
        let text = res.downcast::<PyString>()?;
        Self::from_json(text.to_string())
    }
}
