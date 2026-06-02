# Changelog

Todas las novedades de este proyecto se documentan en este archivo.

El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/)
y el proyecto usa [Versionado Semántico](https://semver.org/lang/es/).

## [Sin publicar]

## [1.0.0] — 2026-06-02

### Añadido
- **Especificación OpenAPI 3.0** (`openapi.yaml`) de la API e-CF: emisión,
  consulta, PDF, anulación, secuencias NCF, RNC, reportes, empresa, rol receptor
  y webhooks.
- **Manual de integración** (`MANUAL.md`): autenticación, onboarding, emisión,
  notas de crédito/débito, reportes y registro de URLs en la DGII.
- **Colección Postman** (`postman_collection.json`) lista para importar.
- **SDKs** en 7 plataformas con la misma superficie de métodos:
  PHP, Node.js/TypeScript, Python, Java, C#/.NET, Rust y ejemplos cURL.
- Operaciones soportadas: `emitInvoice`, `getStatus`, `getInvoice`,
  `downloadPdf`, `cancelInvoice`, `listInvoices`, `listInbound`, `getQueue`,
  secuencias NCF (`getSequences`, `nextNcf`, `configureSequence`,
  `deactivateSequence`), RNC (`lookupRnc`, `autocompleteRnc`, `bulkRnc`),
  reportes (606/607/608, IT-1, ISR), declaraciones (`reportIr2`, `reportIr1`,
  `reportTss`) y configuración de empresa.
- **README** con ejemplos de uso por lenguaje, estados, manejo de errores y
  referencia de métodos. Badges y licencia.
- **Licencia MIT**.

[Sin publicar]: https://github.com/ronbrain/caribefiscal/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/ronbrain/caribefiscal/releases/tag/v1.0.0
