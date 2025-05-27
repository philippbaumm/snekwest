use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyFloat, PyInt, PyString, PyTuple};
use std::collections::HashMap;

#[pyclass]
#[derive(Debug, FromPyObject)]
pub struct RequestParams {
    pub method: String,
    pub url: String,
    pub params: Option<HashMap<String, String>>, // # TODO: simplified arg
    pub data: Option<DataParameter>,             // # TODO: simplified arg
    pub json: Option<PyObject>,
    pub headers: Option<HashMap<String, String>>,
    pub cookies: Option<HashMap<String, String>>,
    pub files: Option<HashMap<String, String>>, // # TODO: simplified arg
    pub auth: Option<(String, String)>,
    pub timeout: Option<TimeoutParameter>,
    pub allow_redirects: bool,
    pub proxies: Option<HashMap<String, String>>,
    pub stream: Option<bool>,
    pub verify: Option<bool>,
    pub cert: Option<CertParameter>,
}

impl RequestParams {
    // Constructor that mirrors requests.Session.request signature
    pub fn from_args(
        method: String,
        url: String,
        params: Option<HashMap<String, String>>,
        data: Option<DataParameter>,
        json: Option<PyObject>,
        headers: Option<HashMap<String, String>>,
        cookies: Option<HashMap<String, String>>,
        files: Option<HashMap<String, String>>,
        auth: Option<(String, String)>,
        timeout: Option<TimeoutParameter>,
        allow_redirects: Option<bool>,
        proxies: Option<HashMap<String, String>>,
        stream: Option<bool>,
        verify: Option<bool>,
        cert: Option<CertParameter>,
    ) -> Self {
        Self {
            method,
            url,
            params,
            data,
            json,
            headers,
            cookies,
            files,
            auth,
            timeout,
            allow_redirects: allow_redirects.unwrap_or(true), // Default like requests
            proxies,
            stream,
            verify,
            cert,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataParameter {
    Form(HashMap<String, String>),
    Raw(Vec<u8>),
}

impl<'py> FromPyObject<'py> for DataParameter {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        // Try dict for form data
        if let Ok(dict) = ob.downcast::<PyDict>() {
            let map: HashMap<String, String> = dict.extract()?;
            return Ok(DataParameter::Form(map));
        }

        // Try string
        if let Ok(s) = ob.downcast::<PyString>() {
            return Ok(DataParameter::Raw(s.to_string().into_bytes()));
        }

        // Try bytes
        if let Ok(bytes) = ob.downcast::<PyBytes>() {
            return Ok(DataParameter::Raw(bytes.as_bytes().to_vec()));
        }

        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "data must be dict, string, or bytes",
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeoutParameter {
    Single(f64),
    Pair(f64, f64),
}

impl<'py> FromPyObject<'py> for TimeoutParameter {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(f) = ob.downcast::<PyFloat>() {
            return Ok(TimeoutParameter::Single(f.extract()?));
        }

        if let Ok(i) = ob.downcast::<PyInt>() {
            return Ok(TimeoutParameter::Single(i.extract::<f64>()?));
        }

        if let Ok(tuple) = ob.downcast::<PyTuple>() {
            if tuple.len() == 2 {
                let first: f64 = tuple.get_item(0)?.extract()?;
                let second: f64 = tuple.get_item(1)?.extract()?;
                return Ok(TimeoutParameter::Pair(first, second));
            }
        }
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Expected float or tuple of two floats",
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CertParameter {
    Single(String),
    Pair(String, String),
}

impl<'py> FromPyObject<'py> for CertParameter {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        if let Ok(s) = ob.downcast::<PyString>() {
            return Ok(CertParameter::Single(s.to_string()));
        }

        if let Ok(tuple) = ob.downcast::<PyTuple>() {
            if tuple.len() == 2 {
                let first: String = tuple.get_item(0)?.extract()?;
                let second: String = tuple.get_item(1)?.extract()?;
                return Ok(CertParameter::Pair(first, second));
            }
        }

        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Expected string or tuple of two strings",
        ))
    }
}
