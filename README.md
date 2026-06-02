# CaribeFiscal · SDK & Integración e-CF (DGII)

SDKs oficiales y documentación para integrar tu sistema con la API de
**Comprobantes Fiscales Electrónicos (e-CF)** de la DGII de República Dominicana.

Firma digital, transmisión a la DGII, recepción de acuses, reportes 606/607/608,
declaraciones (IT-1 / IR-2 / IR-1 / TSS) y validación de RNC — desde cualquier
lenguaje.

- 📖 **Manual completo:** [`MANUAL.md`](./MANUAL.md)
- 🔧 **Especificación OpenAPI 3.0:** [`openapi.yaml`](./openapi.yaml) (impórtala en Swagger/Stoplight)
- 📮 **Colección Postman:** [`postman_collection.json`](./postman_collection.json)
- 🧩 **SDKs:** PHP · Node/TypeScript · Python · Java · C#/.NET · Rust · cURL

---

## Tabla de contenidos
1. [Autenticación](#autenticación)
2. [Instalación rápida por lenguaje](#instalación-rápida-por-lenguaje)
3. [Emitir un comprobante](#emitir-un-comprobante)
4. [Operaciones comunes](#operaciones-comunes)
5. [Reportes y declaraciones](#reportes-y-declaraciones)
6. [Estados y manejo de errores](#estados-y-manejo-de-errores)
7. [Referencia de métodos del SDK](#referencia-de-métodos-del-sdk)

---

## Autenticación

Todas las peticiones usan el header **`X-API-Key`** con tu clave (`ck_live_…` en
producción, `ck_test_…` en pruebas). La obtienes una sola vez en el panel.

```
X-API-Key: ck_live_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

Base URL: `https://ecf.tudominio.do` (producción) · `http://localhost:8081` (local).

> Nunca expongas la API key en frontend; llama a la API desde tu backend.

---

## Instalación rápida por lenguaje

### PHP (7.4+ / 8.x)
```php
require 'CaribeFiscal.php';
$cf = new CaribeFiscal('https://ecf.tudominio.do', 'ck_live_xxx');
```

### Node.js / TypeScript (Node 18+)
```ts
import { CaribeFiscal } from './caribefiscal'
const cf = new CaribeFiscal('https://ecf.tudominio.do', 'ck_live_xxx')
```

### Python (3.8+) — `pip install requests`
```python
from caribefiscal import CaribeFiscal
cf = CaribeFiscal("https://ecf.tudominio.do", "ck_live_xxx")
```

### Java (JDK 11+)
```java
CaribeFiscal cf = new CaribeFiscal("https://ecf.tudominio.do", "ck_live_xxx");
```

### C# / .NET (6+)
```csharp
using CaribeFiscal;
var cf = new CaribeFiscalClient("https://ecf.tudominio.do", "ck_live_xxx");
```

### Rust — `reqwest` + `serde_json`
```rust
use caribefiscal::CaribeFiscal;
let cf = CaribeFiscal::new("https://ecf.tudominio.do", "ck_live_xxx");
```

---

## Emitir un comprobante

Una **Factura de Crédito Fiscal (tipo 31, B2B)** con ITBIS 18%.

### PHP
```php
$res = $cf->emitInvoice([
  'ecfType' => '31',
  'emisor' => [
    'rnc' => '130556677', 'razonSocial' => 'BARSHINE SRL',
    'direccion' => 'Av. Churchill 100', 'fechaEmision' => '2026-06-01',
  ],
  'comprador' => ['rnc' => '101023127', 'razonSocial' => 'CLIENTE SRL'],
  'items' => [[
    'numeroLinea' => 1, 'indicadorBienServicio' => '2',
    'nombre' => 'Consultoría', 'cantidad' => 1,
    'precioUnitario' => 10000, 'tasaITBIS' => 18,
  ]],
  'tipoIngresos' => '01', 'tipoPago' => '1', 'formaPago' => '01',
  'invoiceDate' => '2026-06-01',
]);
echo $res['eNCF'] . ' · ' . $res['status']; // E310000000001 · ACCEPTED
```

### Node / TypeScript
```ts
const res = await cf.emitInvoice({
  ecfType: '31',
  emisor: { rnc: '130556677', razonSocial: 'BARSHINE SRL', direccion: 'Av. Churchill 100', fechaEmision: '2026-06-01' },
  comprador: { rnc: '101023127', razonSocial: 'CLIENTE SRL' },
  items: [{ numeroLinea: 1, indicadorBienServicio: '2', nombre: 'Consultoría', cantidad: 1, precioUnitario: 10000, tasaITBIS: 18 }],
  tipoIngresos: '01', tipoPago: '1', formaPago: '01', invoiceDate: '2026-06-01',
})
console.log(res.eNCF, res.status, res.securityCode)
```

### Python
```python
res = cf.emit_invoice({
    "ecfType": "31",
    "emisor": {"rnc": "130556677", "razonSocial": "BARSHINE SRL",
               "direccion": "Av. Churchill 100", "fechaEmision": "2026-06-01"},
    "comprador": {"rnc": "101023127", "razonSocial": "CLIENTE SRL"},
    "items": [{"numeroLinea": 1, "indicadorBienServicio": "2", "nombre": "Consultoría",
               "cantidad": 1, "precioUnitario": 10000, "tasaITBIS": 18}],
    "tipoIngresos": "01", "tipoPago": "1", "formaPago": "01", "invoiceDate": "2026-06-01",
})
print(res["eNCF"], res["status"])
```

### C# / .NET
```csharp
var res = await cf.EmitInvoiceAsync(new {
    ecfType = "31",
    emisor = new { rnc = "130556677", razonSocial = "BARSHINE SRL", direccion = "Av. Churchill 100", fechaEmision = "2026-06-01" },
    comprador = new { rnc = "101023127", razonSocial = "CLIENTE SRL" },
    items = new[] { new { numeroLinea = 1, indicadorBienServicio = "2", nombre = "Consultoría", cantidad = 1, precioUnitario = 10000, tasaITBIS = 18 } },
    tipoIngresos = "01", tipoPago = "1", formaPago = "01", invoiceDate = "2026-06-01",
});
Console.WriteLine(res.GetProperty("eNCF").GetString());
```

### Rust
```rust
let res = cf.emit_invoice(&serde_json::json!({
    "ecfType": "31",
    "emisor": { "rnc": "130556677", "razonSocial": "BARSHINE SRL", "direccion": "Av. Churchill 100", "fechaEmision": "2026-06-01" },
    "comprador": { "rnc": "101023127", "razonSocial": "CLIENTE SRL" },
    "items": [{ "numeroLinea": 1, "indicadorBienServicio": "2", "nombre": "Consultoría", "cantidad": 1, "precioUnitario": 10000, "tasaITBIS": 18 }],
    "tipoIngresos": "01", "tipoPago": "1", "formaPago": "01", "invoiceDate": "2026-06-01"
}))?;
println!("{} {}", res["eNCF"], res["status"]);
```

### Java
El SDK de Java recibe/devuelve **JSON como `String`** (úsalo con Jackson/Gson):
```java
String body = """
{ "ecfType":"31",
  "emisor":{"rnc":"130556677","razonSocial":"BARSHINE SRL","direccion":"Av. Churchill 100","fechaEmision":"2026-06-01"},
  "comprador":{"rnc":"101023127","razonSocial":"CLIENTE SRL"},
  "items":[{"numeroLinea":1,"indicadorBienServicio":"2","nombre":"Consultoría","cantidad":1,"precioUnitario":10000,"tasaITBIS":18}],
  "tipoIngresos":"01","tipoPago":"1","formaPago":"01","invoiceDate":"2026-06-01" }""";
String res = cf.emitInvoice(body); // -> { "eNCF":"...", "status":"...", ... }
```

### cURL
```bash
curl -X POST https://ecf.tudominio.do/api/v1/ecf/submit \
  -H "X-API-Key: ck_live_xxx" -H "Content-Type: application/json" \
  -d '{ "ecfType":"31",
        "emisor":{"rnc":"130556677","razonSocial":"BARSHINE SRL","direccion":"Av. Churchill 100","fechaEmision":"2026-06-01"},
        "comprador":{"rnc":"101023127","razonSocial":"CLIENTE SRL"},
        "items":[{"numeroLinea":1,"indicadorBienServicio":"2","nombre":"Consultoría","cantidad":1,"precioUnitario":10000,"tasaITBIS":18}],
        "tipoIngresos":"01","tipoPago":"1","formaPago":"01","invoiceDate":"2026-06-01" }'
```

**Respuesta:**
```json
{ "success": true, "eNCF": "E310000000001", "trackId": "...",
  "status": "ACCEPTED", "invoiceId": "9b1f…", "securityCode": "A1B2C3" }
```

---

## Operaciones comunes

### Consultar y descargar el PDF (Node)
```ts
const inv = await cf.getInvoice('E310000000001')

// PDF (representación impresa con QR + código de seguridad)
import { writeFileSync } from 'fs'
const pdf = await cf.downloadPdf('E310000000001')   // ArrayBuffer
writeFileSync('comprobante.pdf', Buffer.from(pdf))
```

### Descargar el PDF (Python / PHP)
```python
pdf = cf.download_pdf("E310000000001")   # bytes
open("comprobante.pdf", "wb").write(pdf)
```
```php
file_put_contents('comprobante.pdf', $cf->downloadPdf('E310000000001'));
```

### Nota de crédito (tipo 34)
```ts
await cf.emitInvoice({
  ecfType: '34',
  emisor: { rnc: '130556677', razonSocial: 'BARSHINE SRL', direccion: 'Av. Churchill 100', fechaEmision: '2026-06-02' },
  items: [{ numeroLinea: 1, indicadorBienServicio: '2', nombre: 'Ajuste', cantidad: 1, precioUnitario: 1000, tasaITBIS: 18 }],
  tipoIngresos: '01', tipoPago: '1', formaPago: '01', invoiceDate: '2026-06-02',
  informacionReferencia: { ncfModificado: 'E310000000001', fechaNCFModificado: '2026-06-01', codigoModificacion: '1' },
})
```

### Anular, listar, cola
```ts
await cf.cancelInvoice('E310000000001', 'Error en monto')
const list = await cf.listInvoices({ status: 'ACCEPTED', page: 1, size: 20 })
const queue = await cf.getQueue()        // pendientes + estado del circuit breaker
```

### Secuencias NCF
```ts
await cf.configureSequence({ ecfType: '31', prefix: 'E31', maxSequence: 10000000, expiryDate: '2026-12-31' })
const seqs = await cf.getSequences()
const next = await cf.nextNcf('31')      // previsualiza el próximo eNCF
```

### Validar RNC
```python
print(cf.lookup_rnc("130556677"))        # { source, rnc, nombre, estado }
print(cf.autocomplete_rnc("barshine"))
```

### Documentos recibidos (rol receptor)
```ts
const inbound = await cf.listInbound()   // e-CF y aprobaciones que te emiten
```

---

## Reportes y declaraciones

```ts
// Formatos pipe oficiales (TXT en res.txt)
const r607 = await cf.report607('2026-06-01', '2026-06-30')  // ventas
const r606 = await cf.report606('2026-06-01', '2026-06-30')  // compras
const r608 = await cf.report608('2026-06-01', '2026-06-30')  // anulados

// IT-1 (ITBIS mensual) — incluye casillas + txt
const it1 = await cf.reportIt1('2026-06-01', '2026-06-30')

// IR-2 (ISR Sociedades, 27%) — cifras contables del ejercicio
const ir2 = await cf.reportIr2({
  periodo: '2025', ingresosBrutos: 5000000, costoVentas: 2000000,
  gastosOperacionales: 1000000, gastosNoDeducibles: 200000,
  anticipos: 100000, retenciones: 50000,
})

// IR-1 (ISR Personas Físicas, escala progresiva)
const ir1 = await cf.reportIr1({ periodo: '2025', ingresos: 700000, retenciones: 20000 })

// TSS (seguridad social) — recibe la nómina
const tss = await cf.reportTss('2026-06', [
  { cedula: '00100000001', nombre: 'Juan Pérez', salarioBruto: 50000 },
])
```
Guardar un reporte TXT:
```python
r = cf.report_607("2026-06-01", "2026-06-30")
open(r["filename"], "w").write(r["txt"])
```

---

## Estados y manejo de errores

### Estados del comprobante (`status`)
| Estado | Significado |
|--------|-------------|
| `ACCEPTED`  | Aceptado por la DGII |
| `SUBMITTED` | Enviado, esperando respuesta |
| `PENDING`   | DGII no disponible; en cola de reintento automático |
| `REJECTED`  | Rechazado |

> **Degradación elegante:** si la DGII está caída, el e-CF no falla — queda
> `PENDING` y un worker lo reenvía. Trata `PENDING` como "aceptado localmente".

### Errores
Todos los SDKs lanzan una excepción con `status` HTTP y el mensaje del servidor:

```ts
try {
  await cf.emitInvoice(payload)
} catch (e) {
  if (e instanceof CaribeFiscalError) console.error(e.status, e.message, e.body)
}
```
```python
from caribefiscal import CaribeFiscalError
try:
    cf.emit_invoice(payload)
except CaribeFiscalError as e:
    print(e.status, e)
```

| HTTP | Significado |
|------|-------------|
| 401 | API key faltante o inválida |
| 402 | Suscripción requerida (producción) |
| 422 | Error de validación (revisa `errors` por campo) |
| 429 | Límite de tasa (50 emisiones/min) — reintenta con backoff |
| 503 | Servicio/BD no disponible — reintenta con backoff |

---

## Referencia de métodos del SDK

Misma superficie en todos los lenguajes (camelCase en PHP/Node/Java/C#, snake_case en Python/Rust):

| Método | Descripción |
|--------|-------------|
| `emitInvoice(payload)` | Emite un e-CF → `{ eNCF, trackId, status, securityCode }` |
| `getStatus(trackId)` | Estado del comprobante en la DGII |
| `getInvoice(encf)` | Detalle del comprobante |
| `downloadPdf(encf)` | PDF (bytes) de la representación impresa |
| `cancelInvoice(encf, motivo)` | Anula el comprobante (ANECF) |
| `listInvoices(filters)` | Lista con filtros y paginación |
| `listInbound(filters)` | Documentos recibidos (rol receptor) |
| `getQueue()` | Cola de envío + circuit breaker |
| `getSequences()` · `nextNcf(type)` · `configureSequence(seq)` · `deactivateSequence(type)` | Secuencias NCF |
| `lookupRnc(rnc)` · `autocompleteRnc(q)` · `bulkRnc(rncs)` | Padrón RNC |
| `report606/607/608(start, end)` · `reportIt1` · `reportIsr` | Formatos fiscales |
| `reportIr2(body)` · `reportIr1(body)` · `reportTss(periodo, empleados)` | Declaraciones |
| `getConfig()` · `updateConfig(cfg)` | Configuración de la empresa |

Tipos de e-CF: `31` Crédito Fiscal · `32` Consumo · `33` Nota Débito · `34` Nota
Crédito · `41` Compras · `43` Gastos Menores · `44` Régimen Especial · `45`
Gubernamental · `46` Exportaciones · `47` Pagos al Exterior.

---

## Soporte
📧 ecf@omegaerp.do · Documentación interactiva en `/docs.html` de tu instancia.
