# Integración · API e-CF CaribeFiscal (DGII)

Todo lo necesario para integrar tu sistema con el microservicio de Comprobantes
Fiscales Electrónicos (e-CF) de la DGII de República Dominicana.

## Contenido

| Recurso | Para qué sirve |
|---------|----------------|
| [`MANUAL.md`](./MANUAL.md) | **Manual de integración** completo (auth, onboarding, emisión, reportes, errores). Empieza aquí. |
| [`openapi.yaml`](./openapi.yaml) | Especificación **OpenAPI 3.0** — impórtala en Swagger UI, Stoplight, o genera clientes. |
| [`postman_collection.json`](./postman_collection.json) | Colección **Postman** lista para probar todos los endpoints. |
| [`sdk/`](./sdk) | **SDKs** en PHP, Node/TypeScript, Python, Java, C#/.NET y Rust, más ejemplos cURL. |

## Inicio rápido

1. Obtén tu **API key** (`ck_live_…` o `ck_test_…`) desde el panel.
2. Importa la colección Postman y define las variables `baseUrl` y `apiKey`.
3. Sube tu certificado y configura la secuencia NCF (ver §3 del manual).
4. Emite tu primer comprobante con `POST /api/v1/ecf/submit`.

## SDKs

| Lenguaje | Archivo | Dependencias |
|----------|---------|--------------|
| PHP | [`sdk/php/CaribeFiscal.php`](./sdk/php/CaribeFiscal.php) | ext-curl (estándar) |
| Node / TypeScript | [`sdk/node/caribefiscal.ts`](./sdk/node/caribefiscal.ts) | ninguna (fetch nativo, Node 18+) |
| Python | [`sdk/python/caribefiscal.py`](./sdk/python/caribefiscal.py) | `requests` |
| Java | [`sdk/java/CaribeFiscal.java`](./sdk/java/CaribeFiscal.java) | ninguna (java.net.http, JDK 11+) |
| C# / .NET | [`sdk/csharp/CaribeFiscal.cs`](./sdk/csharp/CaribeFiscal.cs) | ninguna (HttpClient, .NET 6+) |
| Rust | [`sdk/rust/caribefiscal.rs`](./sdk/rust/caribefiscal.rs) | `reqwest`, `serde_json` |
| cURL / REST | [`sdk/curl/examples.sh`](./sdk/curl/examples.sh) | bash + curl |

Todos los SDKs exponen la misma superficie: `emitInvoice`, `getInvoice`,
`downloadPdf`, `cancelInvoice`, `listInvoices`, `configureSequence`, `nextNcf`,
`lookupRnc`, `report606/607/608/it1/isr`, `reportIr2`, `reportIr1`, `reportTss`,
`getConfig`/`updateConfig`.

## Soporte

ecf@omegaerp.do
