//! CaribeFiscal — SDK Rust para la API e-CF de la DGII (República Dominicana).
//!
//! Dependencias (Cargo.toml):
//!   reqwest    = { version = "0.12", features = ["blocking", "json"] }
//!   serde_json = "1"
//!
//! Ejemplo:
//!   use caribefiscal::CaribeFiscal;
//!   let cf = CaribeFiscal::new("https://ecf.omegaerp.do", "ck_live_xxx");
//!   let res = cf.emit_invoice(&serde_json::json!({ "ecfType": "31", /* ... */ }))?;
//!   println!("{}", res["eNCF"]);

use std::time::Duration;
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::Value;

#[derive(Debug)]
pub struct CaribeFiscalError {
    pub message: String,
    pub status: u16,
    pub body: Option<Value>,
}

impl std::fmt::Display for CaribeFiscalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CaribeFiscalError({}): {}", self.status, self.message)
    }
}
impl std::error::Error for CaribeFiscalError {}

type Result<T> = std::result::Result<T, CaribeFiscalError>;

pub struct CaribeFiscal {
    base_url: String,
    api_key: String,
    client: Client,
}

impl CaribeFiscal {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("no se pudo construir el cliente HTTP");
        CaribeFiscal {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            client,
        }
    }

    // ── e-CF ──────────────────────────────────────────────────────────────────
    pub fn emit_invoice(&self, payload: &Value) -> Result<Value> { self.req(Method::POST, "/api/v1/ecf/submit", Some(payload)) }
    pub fn get_status(&self, track_id: &str) -> Result<Value>    { self.req(Method::GET, &format!("/api/v1/ecf/status/{}", enc(track_id)), None) }
    pub fn get_invoice(&self, encf: &str) -> Result<Value>       { self.req(Method::GET, &format!("/api/v1/ecf/{}", enc(encf)), None) }
    pub fn cancel_invoice(&self, encf: &str, motivo: &str) -> Result<Value> {
        self.req(Method::POST, "/api/v1/ecf/cancel", Some(&serde_json::json!({ "encf": encf, "motivo": motivo })))
    }
    pub fn get_queue(&self) -> Result<Value>                     { self.req(Method::GET, "/api/v1/ecf/queue", None) }
    pub fn list_invoices(&self, query: &[(&str, &str)]) -> Result<Value> { self.req(Method::GET, &format!("/api/v1/ecf/{}", qs(query)), None) }
    pub fn list_inbound(&self, query: &[(&str, &str)]) -> Result<Value>  { self.req(Method::GET, &format!("/api/v1/inbound{}", qs(query)), None) }

    /// Devuelve los bytes del PDF.
    pub fn download_pdf(&self, encf: &str) -> Result<Vec<u8>> {
        let url = format!("{}/api/v1/ecf/{}/pdf", self.base_url, enc(encf));
        let resp = self.client.get(&url).header("X-API-Key", &self.api_key).send()
            .map_err(|e| net_err(&e.to_string()))?;
        let status = resp.status().as_u16();
        let bytes = resp.bytes().map_err(|e| net_err(&e.to_string()))?;
        if status >= 400 {
            return Err(CaribeFiscalError { message: format!("HTTP {}", status), status, body: None });
        }
        Ok(bytes.to_vec())
    }

    // ── NCF ───────────────────────────────────────────────────────────────────
    pub fn get_sequences(&self) -> Result<Value>                { self.req(Method::GET, "/api/v1/ncf/sequences", None) }
    pub fn next_ncf(&self, ecf_type: &str) -> Result<Value>     { self.req(Method::GET, &format!("/api/v1/ncf/next/{}", enc(ecf_type)), None) }
    pub fn configure_sequence(&self, seq: &Value) -> Result<Value> { self.req(Method::POST, "/api/v1/ncf/configure", Some(seq)) }
    pub fn deactivate_sequence(&self, ecf_type: &str) -> Result<Value> { self.req(Method::DELETE, &format!("/api/v1/ncf/{}", enc(ecf_type)), None) }

    // ── RNC ───────────────────────────────────────────────────────────────────
    pub fn lookup_rnc(&self, rnc: &str) -> Result<Value>        { self.req(Method::GET, &format!("/api/v1/rnc/{}", enc(rnc)), None) }
    pub fn autocomplete_rnc(&self, q: &str) -> Result<Value>    { self.req(Method::GET, &format!("/api/v1/rnc/autocomplete{}", qs(&[("q", q)])), None) }
    pub fn bulk_rnc(&self, rncs: &[&str]) -> Result<Value>      { self.req(Method::POST, "/api/v1/rnc/bulk", Some(&serde_json::json!({ "rncs": rncs }))) }

    // ── Reportes ────────────────────────────────────────────────────────────────
    pub fn report_606(&self, s: &str, e: &str) -> Result<Value> { self.report("606", s, e) }
    pub fn report_607(&self, s: &str, e: &str) -> Result<Value> { self.report("607", s, e) }
    pub fn report_608(&self, s: &str, e: &str) -> Result<Value> { self.report("608", s, e) }
    pub fn report_it1(&self, s: &str, e: &str) -> Result<Value> { self.report("it1", s, e) }
    pub fn report_isr(&self, s: &str, e: &str) -> Result<Value> { self.report("isr", s, e) }
    pub fn report_ir2(&self, body: &Value) -> Result<Value>     { self.req(Method::POST, "/api/v1/reports/ir2", Some(body)) }
    pub fn report_ir1(&self, body: &Value) -> Result<Value>     { self.req(Method::POST, "/api/v1/reports/ir1", Some(body)) }
    pub fn report_tss(&self, periodo: &str, empleados: &Value) -> Result<Value> {
        self.req(Method::POST, "/api/v1/reports/tss", Some(&serde_json::json!({ "periodo": periodo, "empleados": empleados })))
    }
    fn report(&self, tipo: &str, start: &str, end: &str) -> Result<Value> {
        self.req(Method::POST, &format!("/api/v1/reports/{}", tipo),
                 Some(&serde_json::json!({ "startDate": start, "endDate": end })))
    }

    // ── Empresa ───────────────────────────────────────────────────────────────
    pub fn get_config(&self) -> Result<Value>                   { self.req(Method::GET, "/api/v1/company/config", None) }
    pub fn update_config(&self, cfg: &Value) -> Result<Value>   { self.req(Method::PUT, "/api/v1/company/config", Some(cfg)) }

    // ── Núcleo HTTP ─────────────────────────────────────────────────────────
    fn req(&self, method: Method, path: &str, body: Option<&Value>) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let mut rb = self.client.request(method, &url)
            .header("X-API-Key", &self.api_key)
            .header("Accept", "application/json");
        if let Some(b) = body {
            rb = rb.json(b);
        }
        let resp = rb.send().map_err(|e| net_err(&e.to_string()))?;
        let status = resp.status().as_u16();
        let text = resp.text().map_err(|e| net_err(&e.to_string()))?;
        let data: Value = if text.is_empty() { Value::Null } else {
            serde_json::from_str(&text).unwrap_or(Value::Null)
        };
        if status >= 400 {
            let message = data.get("error").and_then(|v| v.as_str())
                .map(|s| s.to_string()).unwrap_or_else(|| format!("HTTP {}", status));
            return Err(CaribeFiscalError { message, status, body: Some(data) });
        }
        Ok(data)
    }
}

fn net_err(msg: &str) -> CaribeFiscalError {
    CaribeFiscalError { message: format!("Error de red: {}", msg), status: 0, body: None }
}

/// Codificación percent simple para segmentos de path / valores de query.
fn enc(s: &str) -> String {
    s.bytes().map(|b| match b {
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (b as char).to_string(),
        _ => format!("%{:02X}", b),
    }).collect()
}

fn qs(params: &[(&str, &str)]) -> String {
    let parts: Vec<String> = params.iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| format!("{}={}", enc(k), enc(v)))
        .collect();
    if parts.is_empty() { String::new() } else { format!("?{}", parts.join("&")) }
}
