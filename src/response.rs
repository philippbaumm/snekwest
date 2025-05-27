use pyo3::prelude::*;
use pythonize::pythonize;
use std::collections::HashMap;
use std::sync::Arc;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Response {
    #[pyo3(get)]
    pub status: u16,
    #[pyo3(get)]
    pub url: String,
    #[pyo3(get)]
    pub headers: HashMap<String, String>,
    body: Arc<Vec<u8>>,
}

impl Response {
    pub fn new(status: u16, url: String, headers: HashMap<String, String>, body: Vec<u8>) -> Self {
        Self {
            status,
            url,
            headers,
            body: Arc::new(body),
        }
    }
}

#[pymethods]
impl Response {
    fn json<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match serde_json::from_slice::<serde_json::Value>(&self.body) {
            Ok(json_value) => pythonize(py, &json_value)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())),
            Err(e) => {
                let json_module = py.import("json")?;
                let json_decode_error = json_module.getattr("JSONDecodeError")?;

                let error_msg = format!("Expecting value: {}", e);
                let doc = String::from_utf8_lossy(&self.body);
                let pos = e.column().saturating_sub(1);

                Err(PyErr::from_value(json_decode_error.call1((
                    error_msg,
                    doc.as_ref(),
                    pos,
                ))?))
            }
        }
    }

    fn text(&self) -> PyResult<String> {
        match String::from_utf8(self.body.as_ref().clone()) {
            Ok(text) => Ok(text),
            Err(_) => Ok(String::from_utf8_lossy(self.body.as_ref()).into_owned()),
        }
    }

    fn content(&self) -> Vec<u8> {
        self.body.as_ref().clone()
    }
}
