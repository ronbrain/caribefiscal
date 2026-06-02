# Manual de Integración · API e-CF CaribeFiscal (DGII)

Guía para integrar tu sistema con el microservicio de Comprobantes Fiscales
Electrónicos (e-CF) de la DGII de República Dominicana.

- **Base URL (producción):** `https://ecf.omegaerp.do` *(reemplaza por tu dominio)*
- **Base URL (local):** `http://localhost:8081`
- **Formato:** JSON (UTF-8). Los reportes 606/607/608 e IT-1 devuelven texto plano en el campo `txt`.
- **Especificación máquina:** [`openapi.yaml`](./openapi.yaml) · Colección: [`postman_collection.json`](./postman_collection.json)

---

## 1. Conceptos previos

Un **e-CF** es un comprobante fiscal en formato XML, firmado digitalmente y
transmitido a la DGII. Su número (**eNCF**) tiene 13 caracteres:
`E` + 2 dígitos de tipo + 10 dígitos de secuencia, ej. `E310000000001`.

Tipos de e-CF soportados:

| Tipo | Denominación |
|------|--------------|
| 31 | Factura de Crédito Fiscal |
| 32 | Factura de Consumo |
| 33 | Nota de Débito |
| 34 | Nota de Crédito |
| 41 | Compras |
| 43 | Gastos Menores |
| 44 | Régimen Especial |
| 45 | Gubernamental |
| 46 | Exportaciones |
| 47 | Pagos al Exterior |

El servicio se encarga de **reservar la secuencia, construir el XML, firmarlo,
calcular el código de seguridad, generar el QR y transmitirlo a la DGII**. Tú
solo envías los datos del negocio.

---

## 2. Autenticación

Todas las rutas bajo `/api/v1/*` requieren el header:

```
X-API-Key: ck_live_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

- `ck_live_…` → entorno de producción DGII (`ecf`)
- `ck_test_…` → entornos de prueba/certificación (`testecf` / `certecf`)

La API key se entrega **una sola vez** al crear la cuenta y se guarda hasheada.
Alternativamente, las apps web pueden usar `X-Session-Token` con el token que
devuelve `POST /auth/login`.

> **Seguridad:** nunca expongas la API key en código de frontend. Llama a la API
> desde tu backend. Si se compromete, rótala desde el panel.

---

## 3. Puesta en marcha (onboarding)

Antes de emitir necesitas, una sola vez:

1. **Subir el certificado digital** (`.p12`) emitido por una entidad autorizada:
   ```
   POST /api/v1/company/certificate?password=TU_CLAVE   (multipart: file=cert.p12)
   ```
2. **Configurar la secuencia NCF** que la DGII te autorizó:
   ```
   POST /api/v1/ncf/configure
   { "ecfType":"31", "prefix":"E31", "maxSequence":10000000, "expiryDate":"2026-12-31" }
   ```
3. **(Opcional) Completar datos de la empresa** (`PUT /api/v1/company/config`).

Verifica el estado en cualquier momento con `GET /api/v1/health` (incluye alertas
de secuencias por agotarse o vencer).

---

## 4. Emitir un comprobante

`POST /api/v1/ecf/submit`

### Ejemplo: Factura de Crédito Fiscal (tipo 31, B2B)

```json
{
  "ecfType": "31",
  "emisor": {
    "rnc": "130556677",
    "razonSocial": "BARSHINE SRL",
    "direccion": "Av. Winston Churchill 100, Santo Domingo",
    "fechaEmision": "2026-06-01"
  },
  "comprador": {
    "rnc": "101023127",
    "razonSocial": "CLIENTE EMPRESARIAL SRL"
  },
  "items": [
    {
      "numeroLinea": 1,
      "indicadorBienServicio": "2",
      "nombre": "Servicio de consultoría",
      "cantidad": 1,
      "precioUnitario": 10000,
      "tasaITBIS": 18
    }
  ],
  "tipoIngresos": "01",
  "tipoPago": "1",
  "formaPago": "01",
  "invoiceDate": "2026-06-01"
}
```

### Respuesta

```json
{
  "success": true,
  "eNCF": "E310000000001",
  "trackId": "...",
  "status": "ACCEPTED",
  "invoiceId": "9b1f...",
  "securityCode": "A1B2C3"
}
```

### Estados posibles (`status`)

| Estado | Significado |
|--------|-------------|
| `ACCEPTED`  | Aceptado por la DGII |
| `SUBMITTED` | Enviado, esperando respuesta |
| `PENDING`   | DGII no disponible; en cola de reintento automático |
| `REJECTED`  | Rechazado (revisa el detalle / logs) |

> **Degradación elegante:** si la DGII está caída, el comprobante NO falla:
> queda `PENDING` y un worker lo reenvía automáticamente. Tu integración debe
> tratar `PENDING` como "aceptado localmente, pendiente de confirmar".

### Notas de crédito/débito (tipo 33/34)

Incluye `informacionReferencia` apuntando al comprobante que modificas:

```json
{
  "ecfType": "34",
  "emisor": { "...": "..." },
  "items": [ { "...": "..." } ],
  "tipoIngresos": "01", "tipoPago": "1", "formaPago": "01",
  "invoiceDate": "2026-06-02",
  "informacionReferencia": {
    "ncfModificado": "E310000000001",
    "fechaNCFModificado": "2026-06-01",
    "codigoModificacion": "1"
  }
}
```

> También puedes usar el flujo de **correcciones** (`POST /api/v1/corrections`)
> para crear un borrador, aprobarlo y emitirlo (`/{id}/emit`).

---

## 5. Consultar, descargar y anular

| Acción | Endpoint |
|--------|----------|
| Estado en DGII | `GET /api/v1/ecf/status/{trackId}` |
| Detalle por eNCF | `GET /api/v1/ecf/{encf}` |
| PDF (representación impresa) | `GET /api/v1/ecf/{encf}/pdf` |
| Listado con filtros | `GET /api/v1/ecf/?status=&ecfType=&startDate=&endDate=&page=1&size=20` |
| Anular (ANECF) | `POST /api/v1/ecf/cancel` `{ "encf","motivo" }` |
| Cola + circuit breaker | `GET /api/v1/ecf/queue` |

---

## 6. Secuencias NCF

| Acción | Endpoint |
|--------|----------|
| Listar | `GET /api/v1/ncf/sequences` |
| Siguiente (preview) | `GET /api/v1/ncf/next/{type}` |
| Configurar | `POST /api/v1/ncf/configure` |
| Desactivar | `DELETE /api/v1/ncf/{type}` |
| Alertas | `GET /api/v1/ncf/alerts` |

---

## 7. Validación de RNC

| Acción | Endpoint |
|--------|----------|
| Consultar uno | `GET /api/v1/rnc/{rnc}` |
| Autocompletar | `GET /api/v1/rnc/autocomplete?q=...` |
| Validar varios | `POST /api/v1/rnc/bulk` `{ "rncs":[...] }` |

---

## 8. Reportes y declaraciones

| Formato | Endpoint | Entrada | Salida |
|---------|----------|---------|--------|
| 606 Compras | `POST /api/v1/reports/606` | `{startDate,endDate}` | `txt` (pipe DGII) |
| 607 Ventas | `POST /api/v1/reports/607` | `{startDate,endDate}` | `txt` |
| 608 Anulados | `POST /api/v1/reports/608` | `{startDate,endDate}` | `txt` |
| IT-1 ITBIS | `POST /api/v1/reports/it1` | `{startDate,endDate}` | `casillas` + `txt` |
| IR-2 ISR Sociedades | `POST /api/v1/reports/ir2` | cifras contables (ver abajo) | `casillas` + `txt` + `xml` |
| IR-1 ISR Personas Físicas | `POST /api/v1/reports/ir1` | cifras anuales | `casillas` + `txt` + `xml` |
| TSS Seguridad social | `POST /api/v1/reports/tss` | `{periodo, empleados}` | aportes + `xml` |
| ISR retención servicios | `POST /api/v1/reports/isr` | `{startDate,endDate}` | retención |

### IR-2 (ISR Sociedades)

El e-CF no contiene tu contabilidad completa, así que aportas las cifras del
ejercicio y el servicio aplica las reglas oficiales (**27%** sobre la renta neta
imponible).

```json
{
  "periodo": "2025",
  "ingresosBrutos": 5000000,
  "costoVentas": 2000000,
  "gastosOperacionales": 1000000,
  "gastosNoDeducibles": 200000,
  "anticipos": 100000,
  "retenciones": 50000
}
```

Cálculo: `Renta neta imponible = (5,000,000 − 3,000,000) + 200,000 = 2,200,000` →
`ISR = 27% = 594,000` → `A pagar = 594,000 − 150,000 = 444,000`.

### IR-1 (ISR Personas Físicas) — escala progresiva vigente

| Tramo (RD$) | Tasa |
|-------------|------|
| 0 – 416,220 | Exento |
| 416,220.01 – 624,329 | 15% del excedente de 416,220 |
| 624,329.01 – 867,123 | 31,216 + 20% del excedente de 624,329 |
| > 867,123 | 79,776 + 25% del excedente de 867,123 |

```json
{ "periodo": "2025", "ingresos": 700000, "gastos": 0, "retenciones": 20000 }
```

---

## 9. Rol receptor y registro de URLs en la DGII

La DGII **no usa un registro de webhooks** estilo Stripe. En su modelo distribuido,
**tú publicas las URLs de tus propios servicios web** en el directorio de la DGII.
Para certificarte como Emisor Electrónico **también debes poder recibir** e-CF y
aprobaciones comerciales. Este servicio ya expone esos endpoints en las rutas
exactas que exige la DGII:

Las URLs son **de la plataforma** (no alojas nada). Cada empresa tiene un **buzón
dedicado** identificado por su RNC en el path: `/{RNC}/fe/...`. La plataforma
enruta por ese RNC. (También existe la forma compartida `/fe/...` que enruta por
el RNC dentro del documento.)

| Servicio | Ruta (buzón por RNC) | Función |
|----------|----------------------|---------|
| Autenticación | `GET /{RNC}/fe/autenticacion/api/semilla` + `POST …/validacioncertificado` | Semilla → el emisor la firma → token Bearer |
| Recepción | `POST /{RNC}/fe/recepcion/api/ecf` | Recibe un e-CF y responde con el **ARECF firmado** |
| Aprobación comercial | `POST /{RNC}/fe/aprobacioncomercial/api/ecf` | Recibe una **ACECF** y actualiza el estado del comprobante |

### Cómo se registran las URLs en la DGII

1. **Durante la certificación** (portal de Certificación de FE): Paso 7 *"URL Servicios
   Prueba"* y Paso 12 *"URL Servicios Producción"*. El campo de host admite ruta, así
   que registras el dominio + tu segmento de RNC.
2. **En producción / cambios**: Oficina Virtual → Facturación Electrónica →
   **Mantenimiento de directorio**.

Registra estas URLs (reemplaza el host y usa tu RNC; el panel las muestra ya
armadas con botón Copiar en Certificación → "URL Servicios Prueba"):

```
Autenticación:        https://ecf.tudominio.do/130556677/fe/autenticacion/api/[semilla|validacioncertificado]
Recepción:            https://ecf.tudominio.do/130556677/fe/recepcion/api/ecf
Aprobación Comercial: https://ecf.tudominio.do/130556677/fe/aprobacioncomercial/api/ecf
```

### Flujo de autenticación del emisor (cliente que te envía un e-CF)

1. `GET /fe/autenticacion/api/semilla` → recibe un XML `<SemillaModel>`.
2. Firma esa semilla con su certificado digital (firma enveloped RSA-SHA256).
3. `POST /fe/autenticacion/api/validacioncertificado` con la semilla firmada →
   recibe `{ token, expedido, expira }`.
4. Usa `Authorization: Bearer <token>` para llamar a `/fe/recepcion/api/ecf` o
   `/fe/aprobacioncomercial/api/ecf`.

> El multi-tenant funciona en un host compartido: el receptor se identifica por el
> **RNC Comprador** dentro del e-CF entrante (y por el **RNC Emisor** en la ACECF),
> y el ARECF se firma con el certificado de esa empresa.

### Acuses entrantes ya procesados

`POST /webhooks/dgii` (alias interno) procesa ARECF/ACECF/ANECF que lleguen por esa
vía y actualiza el estado del comprobante. Consulta el historial con
`GET /webhooks/dgii/events`, y los documentos recibidos por tu empresa con
`GET /api/v1/inbound`.

> El resultado fiscal de **tus propios** e-CF no llega por webhook: recibes un
> `trackId` y haces *polling* a `GET /api/v1/ecf/status/{trackId}` o
> `GET /api/v1/ecf/{encf}`. Las URLs `/fe/*` solo las llaman otros cuando **te
> emiten** a ti.

---

## 10. Manejo de errores

Las respuestas de error tienen esta forma:

```json
{
  "success": false,
  "error": "Validation error",
  "errors": { "campo": ["mensaje"] }
}
```

| Código | Significado | Acción recomendada |
|--------|-------------|--------------------|
| 401 | Falta o es inválida la API key | Revisa el header `X-API-Key` |
| 403 | Sin acceso al recurso | El recurso pertenece a otra empresa |
| 404 | No encontrado | Verifica el eNCF / id |
| 409 | Conflicto (RNC/email existe) | — |
| 422 | Error de validación | Revisa `errors` por campo |
| 429 | Límite de tasa (50 emisiones/min) | Reintenta con backoff |
| 503 | Servicio/BD no disponible | Reintenta con backoff |

**Buenas prácticas:** usa timeouts de 120s, reintentos con backoff exponencial
ante 429/503, e idempotencia a nivel de tu sistema (no reenvíes el mismo
comprobante si ya recibiste un `eNCF`).

---

## 11. SDKs oficiales

Disponibles en [`/integration/sdk/`](./sdk):

| Lenguaje | Archivo |
|----------|---------|
| PHP | [`sdk/php/CaribeFiscal.php`](./sdk/php/CaribeFiscal.php) |
| Node.js / TypeScript | [`sdk/node/caribefiscal.ts`](./sdk/node/caribefiscal.ts) |
| Python | [`sdk/python/caribefiscal.py`](./sdk/python/caribefiscal.py) |
| Java | [`sdk/java/CaribeFiscal.java`](./sdk/java/CaribeFiscal.java) |
| C# / .NET (ASP.NET) | [`sdk/csharp/CaribeFiscal.cs`](./sdk/csharp/CaribeFiscal.cs) |
| Rust | [`sdk/rust/caribefiscal.rs`](./sdk/rust/caribefiscal.rs) |
| cURL / REST genérico | [`sdk/curl/examples.sh`](./sdk/curl/examples.sh) |

Todos exponen la misma API mínima:

```
client = new CaribeFiscal(baseUrl, apiKey)
client.emitInvoice(payload)      -> { eNCF, trackId, status, securityCode }
client.getInvoice(encf)
client.downloadPdf(encf)         -> bytes
client.cancelInvoice(encf, motivo)
client.listInvoices(filters)
client.configureSequence(seq)
client.nextNcf(type)
client.lookupRnc(rnc)
client.report607(start, end)     (y 606/608/it1/isr)
client.reportIr2(body) / reportIr1(body) / reportTss(periodo, empleados)
```
