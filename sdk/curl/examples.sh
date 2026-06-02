#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# CaribeFiscal · API e-CF DGII — ejemplos cURL (REST genérico)
# Uso: export BASE_URL y API_KEY, luego ejecuta cada bloque.
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

BASE_URL="${BASE_URL:-https://ecf.omegaerp.do}"   # reemplaza por tu dominio
API_KEY="${API_KEY:-ck_test_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx}"

H_AUTH=(-H "X-API-Key: ${API_KEY}")
H_JSON=(-H "Content-Type: application/json")

echo "== Health =="
curl -s "${H_AUTH[@]}" "${BASE_URL}/api/v1/health"

echo; echo "== Configurar secuencia NCF tipo 31 =="
curl -s "${H_AUTH[@]}" "${H_JSON[@]}" -X POST "${BASE_URL}/api/v1/ncf/configure" -d '{
  "ecfType":"31","prefix":"E31","maxSequence":10000000,"expiryDate":"2026-12-31"
}'

echo; echo "== Emitir factura crédito fiscal (31) =="
curl -s "${H_AUTH[@]}" "${H_JSON[@]}" -X POST "${BASE_URL}/api/v1/ecf/submit" -d '{
  "ecfType":"31",
  "emisor":{"rnc":"130556677","razonSocial":"BARSHINE SRL","direccion":"Av. Churchill 100","fechaEmision":"2026-06-01"},
  "comprador":{"rnc":"101023127","razonSocial":"CLIENTE SRL"},
  "items":[{"numeroLinea":1,"indicadorBienServicio":"2","nombre":"Consultoría","cantidad":1,"precioUnitario":10000,"tasaITBIS":18}],
  "tipoIngresos":"01","tipoPago":"1","formaPago":"01","invoiceDate":"2026-06-01"
}'

echo; echo "== Consultar comprobante por eNCF =="
curl -s "${H_AUTH[@]}" "${BASE_URL}/api/v1/ecf/E310000000001"

echo; echo "== Descargar PDF =="
curl -s "${H_AUTH[@]}" "${BASE_URL}/api/v1/ecf/E310000000001/pdf" -o comprobante.pdf
echo "PDF guardado en comprobante.pdf"

echo; echo "== Anular comprobante =="
curl -s "${H_AUTH[@]}" "${H_JSON[@]}" -X POST "${BASE_URL}/api/v1/ecf/cancel" -d '{
  "encf":"E310000000001","motivo":"Error en monto"
}'

echo; echo "== Validar RNC =="
curl -s "${H_AUTH[@]}" "${BASE_URL}/api/v1/rnc/130556677"

echo; echo "== Reporte 607 (ventas) — TXT en campo .txt =="
curl -s "${H_AUTH[@]}" "${H_JSON[@]}" -X POST "${BASE_URL}/api/v1/reports/607" -d '{
  "startDate":"2026-06-01","endDate":"2026-06-30"
}'

echo; echo "== IT-1 ITBIS mensual =="
curl -s "${H_AUTH[@]}" "${H_JSON[@]}" -X POST "${BASE_URL}/api/v1/reports/it1" -d '{
  "startDate":"2026-06-01","endDate":"2026-06-30"
}'

echo; echo "== IR-2 ISR Sociedades =="
curl -s "${H_AUTH[@]}" "${H_JSON[@]}" -X POST "${BASE_URL}/api/v1/reports/ir2" -d '{
  "periodo":"2025","ingresosBrutos":5000000,"costoVentas":2000000,
  "gastosOperacionales":1000000,"gastosNoDeducibles":200000,"anticipos":100000,"retenciones":50000
}'

echo; echo "== IR-1 ISR Personas Físicas =="
curl -s "${H_AUTH[@]}" "${H_JSON[@]}" -X POST "${BASE_URL}/api/v1/reports/ir1" -d '{
  "periodo":"2025","ingresos":700000,"retenciones":20000
}'

echo; echo "== TSS (nómina) =="
curl -s "${H_AUTH[@]}" "${H_JSON[@]}" -X POST "${BASE_URL}/api/v1/reports/tss" -d '{
  "periodo":"2026-06","empleados":[{"cedula":"00100000001","nombre":"Juan Pérez","salarioBruto":50000}]
}'
