// CaribeFiscal — SDK C# / .NET (ASP.NET) para la API e-CF de la DGII.
//
// .NET 6+ (usa HttpClient + System.Text.Json). Sin dependencias externas.
//
// Ejemplo:
//   var cf = new CaribeFiscal("https://ecf.omegaerp.do", "ck_live_xxx");
//   var res = await cf.EmitInvoiceAsync(payload);   // payload = objeto/anónimo o Dictionary
//   Console.WriteLine(res.GetProperty("eNCF").GetString());
//
// En ASP.NET regístralo como singleton:
//   builder.Services.AddSingleton(new CaribeFiscal(cfg["Ecf:BaseUrl"], cfg["Ecf:ApiKey"]));

using System;
using System.Collections.Generic;
using System.Net.Http;
using System.Net.Http.Json;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using System.Web;

namespace CaribeFiscal
{
    public class CaribeFiscalException : Exception
    {
        public int Status { get; }
        public string? Body { get; }
        public CaribeFiscalException(string message, int status = 0, string? body = null) : base(message)
        {
            Status = status;
            Body = body;
        }
    }

    public class CaribeFiscalClient
    {
        private readonly HttpClient _http;
        private readonly string _baseUrl;

        public CaribeFiscalClient(string baseUrl, string apiKey, HttpClient? http = null)
        {
            _baseUrl = baseUrl.TrimEnd('/');
            _http = http ?? new HttpClient();
            _http.Timeout = TimeSpan.FromSeconds(120);
            _http.DefaultRequestHeaders.Add("X-API-Key", apiKey);
        }

        // ── e-CF ─────────────────────────────────────────────────────────────
        public Task<JsonElement> EmitInvoiceAsync(object payload)        => ReqAsync(HttpMethod.Post, "/api/v1/ecf/submit", payload);
        public Task<JsonElement> GetStatusAsync(string trackId)          => ReqAsync(HttpMethod.Get, $"/api/v1/ecf/status/{Uri.EscapeDataString(trackId)}");
        public Task<JsonElement> GetInvoiceAsync(string encf)            => ReqAsync(HttpMethod.Get, $"/api/v1/ecf/{Uri.EscapeDataString(encf)}");
        public Task<JsonElement> CancelInvoiceAsync(string encf, string motivo) => ReqAsync(HttpMethod.Post, "/api/v1/ecf/cancel", new { encf, motivo });
        public Task<JsonElement> GetQueueAsync()                         => ReqAsync(HttpMethod.Get, "/api/v1/ecf/queue");
        public Task<JsonElement> ListInvoicesAsync(IDictionary<string, string>? filters = null) => ReqAsync(HttpMethod.Get, "/api/v1/ecf/" + Qs(filters));
        public Task<JsonElement> ListInboundAsync(IDictionary<string, string>? filters = null)  => ReqAsync(HttpMethod.Get, "/api/v1/inbound" + Qs(filters));

        public async Task<byte[]> DownloadPdfAsync(string encf)
        {
            using var resp = await _http.GetAsync($"{_baseUrl}/api/v1/ecf/{Uri.EscapeDataString(encf)}/pdf");
            if (!resp.IsSuccessStatusCode)
                throw new CaribeFiscalException($"HTTP {(int)resp.StatusCode}", (int)resp.StatusCode);
            return await resp.Content.ReadAsByteArrayAsync();
        }

        // ── NCF ──────────────────────────────────────────────────────────────
        public Task<JsonElement> GetSequencesAsync()                     => ReqAsync(HttpMethod.Get, "/api/v1/ncf/sequences");
        public Task<JsonElement> NextNcfAsync(string type)              => ReqAsync(HttpMethod.Get, $"/api/v1/ncf/next/{Uri.EscapeDataString(type)}");
        public Task<JsonElement> ConfigureSequenceAsync(object seq)      => ReqAsync(HttpMethod.Post, "/api/v1/ncf/configure", seq);
        public Task<JsonElement> DeactivateSequenceAsync(string type)    => ReqAsync(HttpMethod.Delete, $"/api/v1/ncf/{Uri.EscapeDataString(type)}");

        // ── RNC ──────────────────────────────────────────────────────────────
        public Task<JsonElement> LookupRncAsync(string rnc)             => ReqAsync(HttpMethod.Get, $"/api/v1/rnc/{Uri.EscapeDataString(rnc)}");
        public Task<JsonElement> AutocompleteRncAsync(string q)         => ReqAsync(HttpMethod.Get, "/api/v1/rnc/autocomplete" + Qs(new Dictionary<string, string> { ["q"] = q }));
        public Task<JsonElement> BulkRncAsync(IEnumerable<string> rncs) => ReqAsync(HttpMethod.Post, "/api/v1/rnc/bulk", new { rncs });

        // ── Reportes ───────────────────────────────────────────────────────────
        public Task<JsonElement> Report606Async(string s, string e)    => ReportAsync("606", s, e);
        public Task<JsonElement> Report607Async(string s, string e)    => ReportAsync("607", s, e);
        public Task<JsonElement> Report608Async(string s, string e)    => ReportAsync("608", s, e);
        public Task<JsonElement> ReportIt1Async(string s, string e)    => ReportAsync("it1", s, e);
        public Task<JsonElement> ReportIsrAsync(string s, string e)    => ReportAsync("isr", s, e);
        public Task<JsonElement> ReportIr2Async(object body)            => ReqAsync(HttpMethod.Post, "/api/v1/reports/ir2", body);
        public Task<JsonElement> ReportIr1Async(object body)            => ReqAsync(HttpMethod.Post, "/api/v1/reports/ir1", body);
        public Task<JsonElement> ReportTssAsync(string periodo, object empleados) => ReqAsync(HttpMethod.Post, "/api/v1/reports/tss", new { periodo, empleados });
        private Task<JsonElement> ReportAsync(string tipo, string start, string end) => ReqAsync(HttpMethod.Post, $"/api/v1/reports/{tipo}", new { startDate = start, endDate = end });

        // ── Empresa ──────────────────────────────────────────────────────────
        public Task<JsonElement> GetConfigAsync()                       => ReqAsync(HttpMethod.Get, "/api/v1/company/config");
        public Task<JsonElement> UpdateConfigAsync(object cfg)          => ReqAsync(HttpMethod.Put, "/api/v1/company/config", cfg);

        // ── Núcleo HTTP ────────────────────────────────────────────────────────
        private static string Qs(IDictionary<string, string>? p)
        {
            if (p == null || p.Count == 0) return "";
            var qb = HttpUtility.ParseQueryString(string.Empty);
            foreach (var kv in p)
                if (!string.IsNullOrEmpty(kv.Value)) qb[kv.Key] = kv.Value;
            var s = qb.ToString();
            return string.IsNullOrEmpty(s) ? "" : "?" + s;
        }

        private async Task<JsonElement> ReqAsync(HttpMethod method, string path, object? body = null)
        {
            using var req = new HttpRequestMessage(method, _baseUrl + path);
            if (body != null)
                req.Content = new StringContent(JsonSerializer.Serialize(body), Encoding.UTF8, "application/json");

            using var resp = await _http.SendAsync(req);
            var text = await resp.Content.ReadAsStringAsync();
            JsonElement data = default;
            if (!string.IsNullOrEmpty(text))
            {
                try { data = JsonSerializer.Deserialize<JsonElement>(text); } catch { /* respuesta no-JSON */ }
            }

            if (!resp.IsSuccessStatusCode)
            {
                string msg = $"HTTP {(int)resp.StatusCode}";
                if (data.ValueKind == JsonValueKind.Object && data.TryGetProperty("error", out var e))
                    msg = e.GetString() ?? msg;
                throw new CaribeFiscalException(msg, (int)resp.StatusCode, text);
            }
            return data;
        }
    }
}
