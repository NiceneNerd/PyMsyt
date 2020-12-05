use msyt::{Endianness, Msyt};
use pyo3::{
    create_exception,
    exceptions::PyException,
    prelude::*,
    types::{PyBytes, PyDict, PyString},
    wrap_pyfunction,
};
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
};

create_exception!(pymsyt, MsytError, PyException);

#[pymodule]
fn pymsyt(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Msbt>()?;
    m.add_wrapped(wrap_pyfunction!(create)).unwrap();
    m.add_wrapped(wrap_pyfunction!(export)).unwrap();
    Ok(())
}

/// Export an MSBT file or directory of MSBT files to YAML or JSON.
/// 
/// :param input: The MSBT file or folder of MSBT files to export from
/// :type input: str (**must** be str, cannot be pathlike)
/// :param output: The path to export to. Defaults to same folder with new extension.
/// :type output: str (**must** be str, cannot be pathlike), optional
/// :param json: Whether to output as JSON instead of YAML, optional
/// :type json: bool. Defaults to False.
/// :raises MsytError: Raises an `MsytError` if export fails for any reason.
#[pyfunction]
#[text_signature = "(input, output=None, json=False)"]
fn export(input: String, output: Option<String>, json: Option<bool>) -> PyResult<()> {
    fn export_single<P: AsRef<Path>>(input: P, output: P, json: bool) -> PyResult<()> {
        let msyt = Msyt::from_msbt_file(&input)
            .map_err(|e| MsytError::new_err(format!("Could not read MSBT file: {:?}", e)))?;
        fs::create_dir_all(output.as_ref().parent().unwrap()).map_err(|e| {
            MsytError::new_err(format!(
                "Could not create parent folder for MSYT file: {:?}",
                e
            ))
        })?;
        let mut file = fs::File::create(&output)?;
        match json {
            true => serde_json::to_writer(&mut file, &msyt).map_err(|e| {
                MsytError::new_err(format!("Could not serialize MSBT to JSON: {:?}", e))
            }),
            false => serde_yaml::to_writer(&mut file, &msyt).map_err(|e| {
                MsytError::new_err(format!("Could not serialize MSBT to YAML: {:?}", e))
            }),
        }
    }

    let input: PathBuf = input.into();
    if input.is_dir() {
        let output: PathBuf = if let Some(output) = output {
            output.into()
        } else {
            input.clone()
        };
        fs::create_dir_all(&output)
            .map_err(|e| MsytError::new_err(format!("Could not create output folder: {:?}", e)))?;
        let paths: Vec<PathBuf> = glob::glob(input.join("**/*.msbt").to_str().unwrap())
            .unwrap()
            .filter_map(|f| f.ok())
            .collect();
        paths
            .par_iter()
            .try_for_each(|f| {
                export_single(
                    f,
                    &output
                        .join(f.strip_prefix(&input).unwrap())
                        .with_extension("msyt"),
                    json.unwrap_or(false),
                )
            })
            .map_err(|e| MsytError::new_err(format!("Failed to create MSYT files: {:?}", e)))?;
        Ok(())
    } else if input.is_file() {
        let output = if let Some(output) = output {
            output.into()
        } else {
            input.with_extension("msyt")
        };
        export_single(&input, &output, json.unwrap_or(false))
    } else {
        Err(MsytError::new_err(format!(
            "{} is not a valid file or folder",
            input.to_string_lossy()
        )))
    }
}

/// Creates an MSBT file or directory of MSBT files from YAML or JSON.
/// 
/// :param input: The YAML or JSON file or folder of files to create from
/// :type input: str (**must** be str, cannot be pathlike)
/// :param big_endian: Whether to serialize as big endian
/// :type big_endian: bool
/// :param output: The path to output created MSBT files. Defaults to same folder with new extension.
/// :type output: str (**must** be str, cannot be pathlike), optional
/// :raises MsytError: Raises an `MsytError` if export fails for any reason.
#[pyfunction]
#[text_signature = "(input, big_endian, output=None)"]
fn create(input: String, big_endian: bool, output: Option<String>) -> PyResult<()> {
    fn create_single<P: AsRef<Path>>(input: P, output: P, big_endian: bool) -> PyResult<()> {
        let text = fs::read_to_string(input)?;
        let msyt: Msyt = match serde_yaml::from_str(&text) {
            Ok(m) => m,
            Err(_) => serde_json::from_str(&text).map_err(|e| {
                MsytError::new_err(format!(
                    "Could not parse text as valid MSYT YAML or JSON: {:?}",
                    e
                ))
            })?,
        };
        fs::create_dir_all(output.as_ref().parent().unwrap()).map_err(|e| {
            MsytError::new_err(format!(
                "Could not create parent folder for MSBT file: {:?}",
                e
            ))
        })?;
        msyt.write_as_msbt(
            &mut fs::File::create(output)?,
            match big_endian {
                true => Endianness::Big,
                false => Endianness::Little,
            },
        )
        .map_err(|e| MsytError::new_err(format!("Could not write MSBT file: {:?}", e)))?;
        Ok(())
    }

    let input: PathBuf = input.into();
    if input.is_dir() {
        let output: PathBuf = if let Some(output) = output {
            output.into()
        } else {
            input.clone()
        };
        fs::create_dir_all(&output)
            .map_err(|e| MsytError::new_err(format!("Could not create output folder: {:?}", e)))?;
        let paths: Vec<PathBuf> = glob::glob(input.join("**/*.msyt").to_str().unwrap())
            .unwrap()
            .filter_map(|f| f.ok())
            .collect();
        paths
            .par_iter()
            .try_for_each(|f| {
                create_single(
                    f,
                    &output
                        .join(f.strip_prefix(&input).unwrap())
                        .with_extension("msbt"),
                    big_endian,
                )
            })
            .map_err(|e| MsytError::new_err(format!("Failed to create MSBT files: {:?}", e)))?;
        Ok(())
    } else if input.is_file() {
        let output = if let Some(output) = output {
            output.into()
        } else {
            input.with_extension("msbt")
        };
        create_single(&input, &output, big_endian)
    } else {
        Err(MsytError::new_err(format!(
            "{} is not a valid file or folder",
            input.to_string_lossy()
        )))
    }
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
    /// Parses an MSBT file from a bytes object
    ///
    /// :param data: The bytes of the MSBT file to parse.
    /// :type data: bytes (*only* bytes proper, *not* byteslike)
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
    #[text_signature = "($self, big_endian, /)"]
    pub fn to_binary(&self, big_endian: bool) -> PyResult<Py<PyAny>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(PyBytes::new(
            py,
            &self
                .msyt
                .clone()
                .into_msbt_bytes(match big_endian {
                    true => Endianness::Big,
                    false => Endianness::Little,
                })
                .map_err(|e| {
                    MsytError::new_err(format!("Failed to serialize MSBT file: {:?}", e))
                })?,
        )
        .into())
    }

    /// Generates a YAML representation of this MSBT file.
    ///
    /// :return: Returns the MSBT as a YAML string.
    /// :rtype: str
    /// :raises MsytError: Raises an `MsytError` if serialization fails.
    #[text_signature = "($self)"]
    pub fn to_yaml(&self) -> PyResult<String> {
        Ok(serde_yaml::to_string(&self.msyt)
            .map_err(|e| MsytError::new_err(format!("Failed to dump MSBT to YAML: {:?}", e)))?)
    }

    /// Generates a JSON representation of this MSBT file.
    ///
    /// :return: Returns the MSBT as a JSON string.
    /// :rtype: str
    /// :raises MsytError: Raises an `MsytError` if serialization fails.
    #[text_signature = "($self)"]
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
    #[text_signature = "($self)"]
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
