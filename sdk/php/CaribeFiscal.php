<?php
/**
 * CaribeFiscal — SDK PHP para la API e-CF de la DGII (República Dominicana).
 *
 * Requiere la extensión cURL (estándar en PHP). Compatible con PHP 7.4+ / 8.x.
 *
 * Ejemplo:
 *   require 'CaribeFiscal.php';
 *   $cf = new CaribeFiscal('https://ecf.omegaerp.do', 'ck_live_xxx');
 *   $res = $cf->emitInvoice([...]);
 *   echo $res['eNCF'];
 */
class CaribeFiscalException extends \Exception
{
    /** @var int */    public $status;
    /** @var mixed */  public $body;
    public function __construct(string $message, int $status = 0, $body = null)
    {
        parent::__construct($message, $status);
        $this->status = $status;
        $this->body   = $body;
    }
}

class CaribeFiscal
{
    private string $baseUrl;
    private string $apiKey;
    private int    $timeout;

    public function __construct(string $baseUrl, string $apiKey, int $timeout = 120)
    {
        $this->baseUrl = rtrim($baseUrl, '/');
        $this->apiKey  = $apiKey;
        $this->timeout = $timeout;
    }

    // ── e-CF ────────────────────────────────────────────────────────────────
    public function emitInvoice(array $payload): array          { return $this->request('POST', '/api/v1/ecf/submit', $payload); }
    public function getStatus(string $trackId): array           { return $this->request('GET', "/api/v1/ecf/status/" . rawurlencode($trackId)); }
    public function getInvoice(string $encf): array             { return $this->request('GET', "/api/v1/ecf/" . rawurlencode($encf)); }
    public function cancelInvoice(string $encf, string $motivo): array { return $this->request('POST', '/api/v1/ecf/cancel', ['encf' => $encf, 'motivo' => $motivo]); }
    public function getQueue(): array                           { return $this->request('GET', '/api/v1/ecf/queue'); }
    public function listInvoices(array $filters = []): array    { return $this->request('GET', '/api/v1/ecf/' . $this->qs($filters)); }
    public function listInbound(array $filters = []): array     { return $this->request('GET', '/api/v1/inbound' . $this->qs($filters)); }

    /** Devuelve los bytes binarios del PDF. */
    public function downloadPdf(string $encf): string
    {
        return $this->request('GET', "/api/v1/ecf/" . rawurlencode($encf) . "/pdf", null, true);
    }

    // ── NCF ─────────────────────────────────────────────────────────────────
    public function getSequences(): array                       { return $this->request('GET', '/api/v1/ncf/sequences'); }
    public function nextNcf(string $type): array                { return $this->request('GET', "/api/v1/ncf/next/" . rawurlencode($type)); }
    public function configureSequence(array $seq): array        { return $this->request('POST', '/api/v1/ncf/configure', $seq); }
    public function deactivateSequence(string $type): array     { return $this->request('DELETE', "/api/v1/ncf/" . rawurlencode($type)); }

    // ── RNC ─────────────────────────────────────────────────────────────────
    public function lookupRnc(string $rnc): array              { return $this->request('GET', "/api/v1/rnc/" . rawurlencode($rnc)); }
    public function autocompleteRnc(string $q): array          { return $this->request('GET', '/api/v1/rnc/autocomplete' . $this->qs(['q' => $q])); }
    public function bulkRnc(array $rncs): array                { return $this->request('POST', '/api/v1/rnc/bulk', ['rncs' => $rncs]); }

    // ── Reportes ──────────────────────────────────────────────────────────────
    public function report606(string $start, string $end): array { return $this->report('606', $start, $end); }
    public function report607(string $start, string $end): array { return $this->report('607', $start, $end); }
    public function report608(string $start, string $end): array { return $this->report('608', $start, $end); }
    public function reportIt1(string $start, string $end): array { return $this->report('it1', $start, $end); }
    public function reportIsr(string $start, string $end): array { return $this->report('isr', $start, $end); }
    public function reportIr2(array $body): array               { return $this->request('POST', '/api/v1/reports/ir2', $body); }
    public function reportIr1(array $body): array               { return $this->request('POST', '/api/v1/reports/ir1', $body); }
    public function reportTss(string $periodo, array $empleados): array { return $this->request('POST', '/api/v1/reports/tss', ['periodo' => $periodo, 'empleados' => $empleados]); }

    private function report(string $tipo, string $start, string $end): array
    {
        return $this->request('POST', "/api/v1/reports/$tipo", ['startDate' => $start, 'endDate' => $end]);
    }

    // ── Empresa ───────────────────────────────────────────────────────────────
    public function getConfig(): array                         { return $this->request('GET', '/api/v1/company/config'); }
    public function updateConfig(array $cfg): array            { return $this->request('PUT', '/api/v1/company/config', $cfg); }

    /** Sube el certificado .p12. */
    public function uploadCertificate(string $p12Path, string $password): array
    {
        $url = $this->baseUrl . '/api/v1/company/certificate?password=' . rawurlencode($password);
        $ch  = curl_init($url);
        curl_setopt_array($ch, [
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_TIMEOUT        => $this->timeout,
            CURLOPT_POST           => true,
            CURLOPT_HTTPHEADER     => ['X-API-Key: ' . $this->apiKey],
            CURLOPT_POSTFIELDS     => ['file' => new \CURLFile($p12Path, 'application/x-pkcs12', basename($p12Path))],
        ]);
        return $this->exec($ch, false);
    }

    // ── Núcleo HTTP ─────────────────────────────────────────────────────────
    private function qs(array $params): string
    {
        $params = array_filter($params, fn($v) => $v !== null && $v !== '');
        return $params ? '?' . http_build_query($params) : '';
    }

    private function request(string $method, string $path, $body = null, bool $raw = false)
    {
        $ch = curl_init($this->baseUrl . $path);
        $headers = ['X-API-Key: ' . $this->apiKey, 'Accept: application/json'];
        $opts = [
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_TIMEOUT        => $this->timeout,
            CURLOPT_CUSTOMREQUEST  => $method,
        ];
        if ($body !== null) {
            $opts[CURLOPT_POSTFIELDS] = json_encode($body);
            $headers[] = 'Content-Type: application/json';
        }
        $opts[CURLOPT_HTTPHEADER] = $headers;
        curl_setopt_array($ch, $opts);
        return $this->exec($ch, $raw);
    }

    private function exec($ch, bool $raw)
    {
        $response = curl_exec($ch);
        if ($response === false) {
            $err = curl_error($ch);
            curl_close($ch);
            throw new CaribeFiscalException("Error de red: $err");
        }
        $status = curl_getinfo($ch, CURLINFO_HTTP_CODE);
        curl_close($ch);

        if ($raw) {
            if ($status >= 400) throw new CaribeFiscalException("HTTP $status", $status, $response);
            return $response;
        }

        $data = json_decode($response, true);
        if ($status >= 400) {
            $msg = is_array($data) && isset($data['error']) ? $data['error'] : "HTTP $status";
            throw new CaribeFiscalException($msg, $status, $data);
        }
        return $data ?? [];
    }
}
