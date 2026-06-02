// CaribeFiscal — SDK Java para la API e-CF de la DGII (República Dominicana).
//
// Java 11+ (usa java.net.http.HttpClient). Sin dependencias externas: el cuerpo
// de las peticiones se construye como String JSON; las respuestas se devuelven
// como texto JSON crudo (parsea con tu librería preferida: Jackson/Gson).
//
// Ejemplo:
//   CaribeFiscal cf = new CaribeFiscal("https://ecf.omegaerp.do", "ck_live_xxx");
//   String json = "{ \"ecfType\":\"31\", ... }";
//   String res  = cf.emitInvoice(json);   // -> { "eNCF": "...", ... }

import java.io.IOException;
import java.net.URI;
import java.net.URLEncoder;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpRequest.BodyPublishers;
import java.net.http.HttpResponse;
import java.net.http.HttpResponse.BodyHandlers;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.Map;
import java.util.StringJoiner;

public class CaribeFiscal {

    public static class CaribeFiscalException extends RuntimeException {
        public final int status;
        public final String body;
        public CaribeFiscalException(String message, int status, String body) {
            super(message);
            this.status = status;
            this.body = body;
        }
    }

    private final String baseUrl;
    private final String apiKey;
    private final HttpClient http;

    public CaribeFiscal(String baseUrl, String apiKey) {
        this.baseUrl = baseUrl.replaceAll("/+$", "");
        this.apiKey = apiKey;
        this.http = HttpClient.newBuilder().connectTimeout(Duration.ofSeconds(120)).build();
    }

    // ── e-CF ──────────────────────────────────────────────────────────────────
    public String emitInvoice(String jsonBody)            { return req("POST", "/api/v1/ecf/submit", jsonBody); }
    public String getStatus(String trackId)               { return req("GET", "/api/v1/ecf/status/" + enc(trackId), null); }
    public String getInvoice(String encf)                 { return req("GET", "/api/v1/ecf/" + enc(encf), null); }
    public String cancelInvoice(String encf, String motivo){ return req("POST", "/api/v1/ecf/cancel", obj("encf", encf, "motivo", motivo)); }
    public String getQueue()                              { return req("GET", "/api/v1/ecf/queue", null); }
    public String listInvoices(Map<String,String> filters){ return req("GET", "/api/v1/ecf/" + qs(filters), null); }
    public String listInbound(Map<String,String> filters) { return req("GET", "/api/v1/inbound" + qs(filters), null); }

    /** Devuelve los bytes del PDF. */
    public byte[] downloadPdf(String encf) {
        try {
            HttpRequest r = base("/api/v1/ecf/" + enc(encf) + "/pdf").GET().build();
            HttpResponse<byte[]> resp = http.send(r, BodyHandlers.ofByteArray());
            if (resp.statusCode() >= 400) throw new CaribeFiscalException("HTTP " + resp.statusCode(), resp.statusCode(), null);
            return resp.body();
        } catch (IOException | InterruptedException e) {
            throw new CaribeFiscalException("Error de red: " + e.getMessage(), 0, null);
        }
    }

    // ── NCF ───────────────────────────────────────────────────────────────────
    public String getSequences()                          { return req("GET", "/api/v1/ncf/sequences", null); }
    public String nextNcf(String type)                    { return req("GET", "/api/v1/ncf/next/" + enc(type), null); }
    public String configureSequence(String jsonBody)      { return req("POST", "/api/v1/ncf/configure", jsonBody); }
    public String deactivateSequence(String type)         { return req("DELETE", "/api/v1/ncf/" + enc(type), null); }

    // ── RNC ───────────────────────────────────────────────────────────────────
    public String lookupRnc(String rnc)                   { return req("GET", "/api/v1/rnc/" + enc(rnc), null); }
    public String autocompleteRnc(String q)               { return req("GET", "/api/v1/rnc/autocomplete" + qs(Map.of("q", q)), null); }
    public String bulkRnc(String jsonBody)                { return req("POST", "/api/v1/rnc/bulk", jsonBody); }

    // ── Reportes ──────────────────────────────────────────────────────────────
    public String report606(String s, String e)           { return report("606", s, e); }
    public String report607(String s, String e)           { return report("607", s, e); }
    public String report608(String s, String e)           { return report("608", s, e); }
    public String reportIt1(String s, String e)           { return report("it1", s, e); }
    public String reportIsr(String s, String e)           { return report("isr", s, e); }
    public String reportIr2(String jsonBody)              { return req("POST", "/api/v1/reports/ir2", jsonBody); }
    public String reportIr1(String jsonBody)              { return req("POST", "/api/v1/reports/ir1", jsonBody); }
    public String reportTss(String jsonBody)              { return req("POST", "/api/v1/reports/tss", jsonBody); }
    private String report(String tipo, String s, String e){ return req("POST", "/api/v1/reports/" + tipo, obj("startDate", s, "endDate", e)); }

    // ── Empresa ───────────────────────────────────────────────────────────────
    public String getConfig()                             { return req("GET", "/api/v1/company/config", null); }
    public String updateConfig(String jsonBody)           { return req("PUT", "/api/v1/company/config", jsonBody); }

    // ── Núcleo HTTP ─────────────────────────────────────────────────────────
    private HttpRequest.Builder base(String path) {
        return HttpRequest.newBuilder(URI.create(baseUrl + path))
                .timeout(Duration.ofSeconds(120))
                .header("X-API-Key", apiKey)
                .header("Accept", "application/json");
    }

    private String req(String method, String path, String jsonBody) {
        try {
            HttpRequest.Builder b = base(path);
            if (jsonBody != null) {
                b.header("Content-Type", "application/json");
                b.method(method, BodyPublishers.ofString(jsonBody, StandardCharsets.UTF_8));
            } else {
                b.method(method, BodyPublishers.noBody());
            }
            HttpResponse<String> resp = http.send(b.build(), BodyHandlers.ofString());
            if (resp.statusCode() >= 400) {
                throw new CaribeFiscalException("HTTP " + resp.statusCode(), resp.statusCode(), resp.body());
            }
            return resp.body();
        } catch (IOException | InterruptedException e) {
            throw new CaribeFiscalException("Error de red: " + e.getMessage(), 0, null);
        }
    }

    private static String enc(String v) { return URLEncoder.encode(v, StandardCharsets.UTF_8); }

    private static String qs(Map<String, String> params) {
        if (params == null || params.isEmpty()) return "";
        StringJoiner sj = new StringJoiner("&", "?", "");
        params.forEach((k, v) -> { if (v != null && !v.isEmpty()) sj.add(enc(k) + "=" + enc(v)); });
        return sj.toString();
    }

    /** Mini-builder de JSON para cuerpos simples clave/valor (strings). */
    private static String obj(String... kv) {
        StringBuilder sb = new StringBuilder("{");
        for (int i = 0; i + 1 < kv.length; i += 2) {
            if (i > 0) sb.append(',');
            sb.append('"').append(kv[i]).append("\":\"").append(kv[i + 1].replace("\"", "\\\"")).append('"');
        }
        return sb.append('}').toString();
    }
}
