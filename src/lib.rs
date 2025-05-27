mod exceptions;
mod request_params;
mod response;
mod session;

use exceptions::*;
use pyo3::prelude::*;
use response::Response;
use session::Session;

#[pymodule]
fn _bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("RequestException", m.py().get_type::<RequestException>())?;
    m.add("HTTPError", m.py().get_type::<HTTPError>())?;
    m.add("ConnectionError", m.py().get_type::<ConnectionError>())?;
    m.add("ProxyError", m.py().get_type::<ProxyError>())?;
    m.add("SSLError", m.py().get_type::<SSLError>())?;
    m.add("Timeout", m.py().get_type::<Timeout>())?;
    m.add("ConnectTimeout", m.py().get_type::<ConnectTimeout>())?;
    m.add("ReadTimeout", m.py().get_type::<ReadTimeout>())?;
    m.add("URLRequired", m.py().get_type::<URLRequired>())?;
    m.add("TooManyRedirects", m.py().get_type::<TooManyRedirects>())?;
    m.add("MissingSchema", m.py().get_type::<MissingSchema>())?;
    m.add("InvalidSchema", m.py().get_type::<InvalidSchema>())?;
    m.add("InvalidURL", m.py().get_type::<InvalidURL>())?;
    m.add("InvalidHeader", m.py().get_type::<InvalidHeader>())?;
    m.add("InvalidProxyURL", m.py().get_type::<InvalidProxyURL>())?;
    m.add(
        "ChunkedEncodingError",
        m.py().get_type::<ChunkedEncodingError>(),
    )?;
    m.add(
        "ContentDecodingError",
        m.py().get_type::<ContentDecodingError>(),
    )?;
    m.add(
        "StreamConsumedError",
        m.py().get_type::<StreamConsumedError>(),
    )?;
    m.add("RetryError", m.py().get_type::<RetryError>())?;
    m.add(
        "UnrewindableBodyError",
        m.py().get_type::<UnrewindableBodyError>(),
    )?;
    m.add("InvalidJSONError", m.py().get_type::<InvalidJSONError>())?;
    m.add("JSONDecodeError", m.py().get_type::<JSONDecodeError>())?;
    m.add("RequestsWarning", m.py().get_type::<RequestsWarning>())?;
    m.add("FileModeWarning", m.py().get_type::<FileModeWarning>())?;
    m.add(
        "RequestsDependencyWarning",
        m.py().get_type::<RequestsDependencyWarning>(),
    )?;

    m.add_class::<Session>()?;
    m.add_class::<Response>()?;
    Ok(())
}
