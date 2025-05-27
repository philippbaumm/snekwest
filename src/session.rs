use pyo3::prelude::*;
use pythonize::depythonize;
use reqwest::blocking::{Client, ClientBuilder};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::request_params::CertParameter;
use crate::request_params::{DataParameter, RequestParams, TimeoutParameter};
use crate::response::Response;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct ClientConfig {
    verify: bool,
    cert_hash: Option<String>,
    proxy_hash: Option<String>,
    allow_redirects: bool,
}

impl ClientConfig {
    fn from_params(params: &RequestParams) -> Self {
        Self {
            verify: params.verify.unwrap_or(true),
            cert_hash: params.cert.as_ref().map(|c| format!("{:?}", c)),
            proxy_hash: params.proxies.as_ref().map(|p| format!("{:?}", p)),
            allow_redirects: params.allow_redirects,
        }
    }
}

#[pyclass]
pub struct Session {
    clients: Mutex<HashMap<ClientConfig, Arc<Client>>>,
    cookie_jar: Mutex<HashMap<String, String>>,
}

impl Session {
    fn get_or_create_client(&self, params: &RequestParams) -> PyResult<Arc<Client>> {
        let config = ClientConfig::from_params(params);
        let mut clients = self.clients.lock().unwrap();

        if let Some(existing_client) = clients.get(&config) {
            return Ok(existing_client.clone());
        }

        let new_client = self.create_client_for_config(&config)?;
        clients.insert(config, new_client.clone());
        Ok(new_client)
    }

    fn merge_cookies(&self, params: &RequestParams) -> HashMap<String, String> {
        let mut all_cookies = {
            let session_cookies = self.cookie_jar.lock().unwrap();
            session_cookies.clone()
        };

        if let Some(request_cookies) = &params.cookies {
            all_cookies.extend(request_cookies.clone());
        }

        all_cookies
    }

    fn build_request(
        &self,
        client: &Client,
        params: &RequestParams,
        cookies: &HashMap<String, String>,
    ) -> PyResult<reqwest::blocking::RequestBuilder> {
        let mut request = client.request(
            params.method.parse().map_err(|_| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid HTTP method")
            })?,
            &params.url,
        );

        if let Some(query_params) = &params.params {
            request = request.query(query_params);
        }

        if let Some(headers) = &params.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        if !cookies.is_empty() {
            let cookie_header = cookies
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("; ");
            request = request.header("Cookie", cookie_header);
        }

        if let Some((username, password)) = &params.auth {
            request = request.basic_auth(username, Some(password));
        }

        Ok(request)
    }

    fn apply_body_and_timeout(
        &self,
        mut request: reqwest::blocking::RequestBuilder,
        params: &RequestParams,
    ) -> PyResult<reqwest::blocking::RequestBuilder> {
        if let Some(json_data) = &params.json {
            let json_string = self.serialize_json_body(json_data)?;
            request = request
                .header("Content-Type", "application/json")
                .body(json_string);
        } else if let Some(files) = &params.files {
            let mut form = reqwest::blocking::multipart::Form::new();
            for (field_name, file_path) in files {
                // TODO: This is simplified - real implementation should read file contents
                form = form.text(field_name.clone(), file_path.clone());
            }
            request = request.multipart(form);
        } else if let Some(data) = &params.data {
            request = match data {
                DataParameter::Form(form_data) => request.form(form_data),
                DataParameter::Raw(raw_bytes) => request.body(raw_bytes.clone()),
            };
        }

        if let Some(timeout_params) = &params.timeout {
            let timeout_duration = self.calculate_timeout(timeout_params);
            request = request.timeout(timeout_duration);
        }

        Ok(request)
    }

    fn serialize_json_body(&self, json_value: &PyObject) -> PyResult<String> {
        Python::with_gil(|py| {
            let rust_value: serde_json::Value = depythonize(json_value.bind(py)).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Failed to convert Python object to JSON: {}",
                    e
                ))
            })?;

            serde_json::to_string(&rust_value).map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "JSON serialization error: {}",
                    e
                ))
            })
        })
    }

    fn calculate_timeout(&self, timeout_params: &TimeoutParameter) -> std::time::Duration {
        match timeout_params {
            TimeoutParameter::Single(secs) => std::time::Duration::from_secs_f64(*secs),
            TimeoutParameter::Pair(connect_timeout, read_timeout) => {
                // Use the larger timeout for total request timeout
                // In a more sophisticated implementation, you'd handle these separately
                std::time::Duration::from_secs_f64(connect_timeout.max(*read_timeout))
            }
        }
    }

    fn update_session_cookies(&self, response: &reqwest::blocking::Response) {
        let mut jar = self.cookie_jar.lock().unwrap();

        for (name, value) in response.headers() {
            if name.as_str().to_lowercase() == "set-cookie" {
                if let Ok(cookie_str) = value.to_str() {
                    self.parse_and_store_cookie(&mut jar, cookie_str);
                }
            }
        }
    }

    fn parse_and_store_cookie(&self, jar: &mut HashMap<String, String>, cookie_str: &str) {
        // cookie parsing TODO: this is simplified
        if let Some(cookie_pair) = cookie_str.split(';').next() {
            if let Some((key, val)) = cookie_pair.split_once('=') {
                jar.insert(key.trim().to_string(), val.trim().to_string());
            }
        }
    }

    fn extract_response_data(
        &self,
        response: reqwest::blocking::Response,
        stream: bool,
    ) -> PyResult<Response> {
        let status = response.status().as_u16();
        let url = response.url().to_string();
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let body = if stream {
            // TODO: this is simplified
            Vec::new()
        } else {
            response
                .bytes()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
                .to_vec()
        };

        Ok(Response::new(status, url, headers, body))
    }

    fn create_client_for_config(&self, config: &ClientConfig) -> PyResult<Arc<Client>> {
        let mut builder = ClientBuilder::new();

        if !config.verify {
            builder = builder.danger_accept_invalid_certs(true);
        }

        if !config.allow_redirects {
            builder = builder.redirect(reqwest::redirect::Policy::none());
        }

        // Configure SSL certificate TODO: this is simplified
        // Configure proxy TODO: this is simplified

        let client = builder
            .build()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(Arc::new(client))
    }
}

#[pymethods]
impl Session {
    #[new]
    fn new() -> Self {
        Session {
            clients: Mutex::new(HashMap::new()),
            cookie_jar: Mutex::new(HashMap::new()),
        }
    }

    #[pyo3(signature = (
        method,
        url,
        *,
        params = None,
        data = None,
        json = None,
        headers = None,
        cookies = None,
        files = None,
        auth = None,
        timeout = None,
        allow_redirects = None,
        proxies = None,
        stream = None,
        verify = None,
        cert = None
    ))]
    fn make_request(
        &self,
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
    ) -> PyResult<Response> {
        let params = RequestParams::from_args(
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
            allow_redirects,
            proxies,
            stream,
            verify,
            cert,
        );

        let client: Arc<Client> = self.get_or_create_client(&params)?;
        let merged_cookies: HashMap<String, String> = self.merge_cookies(&params);
        let mut request: reqwest::blocking::RequestBuilder =
            self.build_request(&client, &params, &merged_cookies)?;
        request = self.apply_body_and_timeout(request, &params)?;

        let response = request
            .send()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        self.update_session_cookies(&response);
        self.extract_response_data(response, params.stream.unwrap_or(false))
    }

    fn close(&self) {
        let mut clients = self.clients.lock().unwrap();
        clients.clear();
        let mut cookies = self.cookie_jar.lock().unwrap();
        cookies.clear();
    }

    fn get_cookies(&self) -> HashMap<String, String> {
        let jar = self.cookie_jar.lock().unwrap();
        jar.clone()
    }

    fn set_cookies(&self, cookies: HashMap<String, String>) {
        let mut jar = self.cookie_jar.lock().unwrap();
        jar.extend(cookies);
    }
}
