"""
CaribeFiscal — SDK Python para la API e-CF de la DGII (República Dominicana).

Requiere `requests` (`pip install requests`). Compatible con Python 3.8+.

Ejemplo:
    from caribefiscal import CaribeFiscal
    cf = CaribeFiscal("https://ecf.omegaerp.do", "ck_live_xxx")
    res = cf.emit_invoice({...})
    print(res["eNCF"])
"""
from __future__ import annotations

from typing import Any, Dict, List, Optional
import requests


class CaribeFiscalError(Exception):
    def __init__(self, message: str, status: int = 0, body: Any = None):
        super().__init__(message)
        self.status = status
        self.body = body


class CaribeFiscal:
    def __init__(self, base_url: str, api_key: str, timeout: int = 120):
        self.base_url = base_url.rstrip("/")
        self.timeout = timeout
        self._session = requests.Session()
        self._session.headers.update({"X-API-Key": api_key, "Accept": "application/json"})

    # ── e-CF ────────────────────────────────────────────────────────────────
    def emit_invoice(self, payload: Dict[str, Any]) -> Dict[str, Any]:
        return self._req("POST", "/api/v1/ecf/submit", payload)

    def get_status(self, track_id: str) -> Dict[str, Any]:
        return self._req("GET", f"/api/v1/ecf/status/{track_id}")

    def get_invoice(self, encf: str) -> Dict[str, Any]:
        return self._req("GET", f"/api/v1/ecf/{encf}")

    def cancel_invoice(self, encf: str, motivo: str) -> Dict[str, Any]:
        return self._req("POST", "/api/v1/ecf/cancel", {"encf": encf, "motivo": motivo})

    def get_queue(self) -> Dict[str, Any]:
        return self._req("GET", "/api/v1/ecf/queue")

    def list_invoices(self, **filters: Any) -> Dict[str, Any]:
        return self._req("GET", "/api/v1/ecf/", params=filters)

    def list_inbound(self, **filters: Any) -> Dict[str, Any]:
        return self._req("GET", "/api/v1/inbound", params=filters)

    def download_pdf(self, encf: str) -> bytes:
        """Devuelve los bytes del PDF."""
        r = self._session.get(f"{self.base_url}/api/v1/ecf/{encf}/pdf", timeout=self.timeout)
        if r.status_code >= 400:
            raise CaribeFiscalError(f"HTTP {r.status_code}", r.status_code, r.content)
        return r.content

    # ── NCF ──────────────────────────────────────────────────────────────────
    def get_sequences(self) -> Dict[str, Any]:
        return self._req("GET", "/api/v1/ncf/sequences")

    def next_ncf(self, ecf_type: str) -> Dict[str, Any]:
        return self._req("GET", f"/api/v1/ncf/next/{ecf_type}")

    def configure_sequence(self, seq: Dict[str, Any]) -> Dict[str, Any]:
        return self._req("POST", "/api/v1/ncf/configure", seq)

    def deactivate_sequence(self, ecf_type: str) -> Dict[str, Any]:
        return self._req("DELETE", f"/api/v1/ncf/{ecf_type}")

    # ── RNC ──────────────────────────────────────────────────────────────────
    def lookup_rnc(self, rnc: str) -> Dict[str, Any]:
        return self._req("GET", f"/api/v1/rnc/{rnc}")

    def autocomplete_rnc(self, q: str) -> Dict[str, Any]:
        return self._req("GET", "/api/v1/rnc/autocomplete", params={"q": q})

    def bulk_rnc(self, rncs: List[str]) -> Dict[str, Any]:
        return self._req("POST", "/api/v1/rnc/bulk", {"rncs": rncs})

    # ── Reportes ──────────────────────────────────────────────────────────────
    def report_606(self, start: str, end: str) -> Dict[str, Any]:
        return self._report("606", start, end)

    def report_607(self, start: str, end: str) -> Dict[str, Any]:
        return self._report("607", start, end)

    def report_608(self, start: str, end: str) -> Dict[str, Any]:
        return self._report("608", start, end)

    def report_it1(self, start: str, end: str) -> Dict[str, Any]:
        return self._report("it1", start, end)

    def report_isr(self, start: str, end: str) -> Dict[str, Any]:
        return self._report("isr", start, end)

    def report_ir2(self, body: Dict[str, Any]) -> Dict[str, Any]:
        return self._req("POST", "/api/v1/reports/ir2", body)

    def report_ir1(self, body: Dict[str, Any]) -> Dict[str, Any]:
        return self._req("POST", "/api/v1/reports/ir1", body)

    def report_tss(self, periodo: str, empleados: List[Dict[str, Any]]) -> Dict[str, Any]:
        return self._req("POST", "/api/v1/reports/tss", {"periodo": periodo, "empleados": empleados})

    def _report(self, tipo: str, start: str, end: str) -> Dict[str, Any]:
        return self._req("POST", f"/api/v1/reports/{tipo}", {"startDate": start, "endDate": end})

    # ── Empresa ────────────────────────────────────────────────────────────────
    def get_config(self) -> Dict[str, Any]:
        return self._req("GET", "/api/v1/company/config")

    def update_config(self, cfg: Dict[str, Any]) -> Dict[str, Any]:
        return self._req("PUT", "/api/v1/company/config", cfg)

    def upload_certificate(self, p12_path: str, password: str) -> Dict[str, Any]:
        with open(p12_path, "rb") as fh:
            r = self._session.post(
                f"{self.base_url}/api/v1/company/certificate",
                params={"password": password},
                files={"file": ("cert.p12", fh, "application/x-pkcs12")},
                timeout=self.timeout,
            )
        return self._handle(r)

    # ── Núcleo HTTP ───────────────────────────────────────────────────────────
    def _req(self, method: str, path: str, body: Optional[Dict[str, Any]] = None,
             params: Optional[Dict[str, Any]] = None) -> Dict[str, Any]:
        params = {k: v for k, v in (params or {}).items() if v is not None and v != ""}
        r = self._session.request(
            method, f"{self.base_url}{path}",
            json=body if body is not None else None,
            params=params or None,
            timeout=self.timeout,
        )
        return self._handle(r)

    @staticmethod
    def _handle(r: requests.Response) -> Dict[str, Any]:
        try:
            data = r.json() if r.content else {}
        except ValueError:
            data = {}
        if r.status_code >= 400:
            msg = data.get("error") if isinstance(data, dict) else f"HTTP {r.status_code}"
            raise CaribeFiscalError(msg or f"HTTP {r.status_code}", r.status_code, data)
        return data
