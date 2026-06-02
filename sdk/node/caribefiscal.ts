/**
 * CaribeFiscal — SDK Node.js / TypeScript para la API e-CF de la DGII.
 *
 * Sin dependencias: usa `fetch` nativo (Node 18+). Para Node <18 instala
 * `undici` o `node-fetch` y asígnalo a globalThis.fetch.
 *
 * Ejemplo:
 *   import { CaribeFiscal } from './caribefiscal'
 *   const cf = new CaribeFiscal('https://ecf.omegaerp.do', 'ck_live_xxx')
 *   const res = await cf.emitInvoice({ ... })
 *   console.log(res.eNCF)
 */

export interface EcfItem {
  numeroLinea: number
  indicadorBienServicio: '1' | '2'   // 1=bien, 2=servicio
  nombre: string
  cantidad: number
  precioUnitario: number
  descuento?: number
  recargo?: number
  tasaITBIS?: 0 | 16 | 18
}

export interface SubmitEcf {
  ecfType: '31'|'32'|'33'|'34'|'41'|'43'|'44'|'45'|'46'|'47'
  emisor: {
    rnc: string; razonSocial: string; direccion: string; fechaEmision: string
    nombreComercial?: string; telefono?: string; email?: string
  }
  comprador?: { rnc?: string; cedula?: string; razonSocial: string; direccion?: string; email?: string }
  items: EcfItem[]
  tipoIngresos: '01'|'02'|'03'|'04'|'05'|'06'
  tipoPago: '1'|'2'|'3'
  formaPago: '01'|'02'|'03'|'04'|'05'|'06'|'07'|'08'|'09'
  invoiceDate: string
  plazoCredito?: string
  fechaVencimientoSecuencia?: string
  informacionReferencia?: {
    ncfModificado: string; fechaNCFModificado: string
    codigoModificacion: '1'|'2'|'3'|'4'|'5'
    rncOtroContribuyente?: string; razonModificacion?: string; indicadorNotaCredito?: '0'|'1'
  }
}

export interface SubmitResult {
  success: boolean; eNCF: string; trackId: string
  status: 'PENDING'|'SUBMITTED'|'ACCEPTED'|'REJECTED'
  invoiceId: string; securityCode: string
}

export class CaribeFiscalError extends Error {
  constructor(message: string, public status: number, public body?: unknown) {
    super(message)
    this.name = 'CaribeFiscalError'
  }
}

export class CaribeFiscal {
  private baseUrl: string
  constructor(baseUrl: string, private apiKey: string, private timeoutMs = 120_000) {
    this.baseUrl = baseUrl.replace(/\/$/, '')
  }

  // ── e-CF ──────────────────────────────────────────────────────────────────
  emitInvoice(payload: SubmitEcf)              { return this.req<SubmitResult>('POST', '/api/v1/ecf/submit', payload) }
  getStatus(trackId: string)                   { return this.req('GET', `/api/v1/ecf/status/${encodeURIComponent(trackId)}`) }
  getInvoice(encf: string)                     { return this.req('GET', `/api/v1/ecf/${encodeURIComponent(encf)}`) }
  cancelInvoice(encf: string, motivo: string)  { return this.req('POST', '/api/v1/ecf/cancel', { encf, motivo }) }
  getQueue()                                   { return this.req('GET', '/api/v1/ecf/queue') }
  listInvoices(filters: Record<string, unknown> = {}) { return this.req('GET', '/api/v1/ecf/' + this.qs(filters)) }
  listInbound(filters: Record<string, unknown> = {})  { return this.req('GET', '/api/v1/inbound' + this.qs(filters)) }

  /** Devuelve el PDF como ArrayBuffer (usa Buffer.from(...) para guardarlo). */
  async downloadPdf(encf: string): Promise<ArrayBuffer> {
    const res = await this.raw('GET', `/api/v1/ecf/${encodeURIComponent(encf)}/pdf`)
    if (!res.ok) throw new CaribeFiscalError(`HTTP ${res.status}`, res.status)
    return res.arrayBuffer()
  }

  // ── NCF ───────────────────────────────────────────────────────────────────
  getSequences()                               { return this.req('GET', '/api/v1/ncf/sequences') }
  nextNcf(type: string)                        { return this.req('GET', `/api/v1/ncf/next/${encodeURIComponent(type)}`) }
  configureSequence(seq: Record<string, unknown>) { return this.req('POST', '/api/v1/ncf/configure', seq) }
  deactivateSequence(type: string)             { return this.req('DELETE', `/api/v1/ncf/${encodeURIComponent(type)}`) }

  // ── RNC ───────────────────────────────────────────────────────────────────
  lookupRnc(rnc: string)                       { return this.req('GET', `/api/v1/rnc/${encodeURIComponent(rnc)}`) }
  autocompleteRnc(q: string)                   { return this.req('GET', '/api/v1/rnc/autocomplete' + this.qs({ q })) }
  bulkRnc(rncs: string[])                      { return this.req('POST', '/api/v1/rnc/bulk', { rncs }) }

  // ── Reportes ────────────────────────────────────────────────────────────────
  report606(startDate: string, endDate: string) { return this.req('POST', '/api/v1/reports/606', { startDate, endDate }) }
  report607(startDate: string, endDate: string) { return this.req('POST', '/api/v1/reports/607', { startDate, endDate }) }
  report608(startDate: string, endDate: string) { return this.req('POST', '/api/v1/reports/608', { startDate, endDate }) }
  reportIt1(startDate: string, endDate: string) { return this.req('POST', '/api/v1/reports/it1', { startDate, endDate }) }
  reportIsr(startDate: string, endDate: string) { return this.req('POST', '/api/v1/reports/isr', { startDate, endDate }) }
  reportIr2(body: Record<string, unknown>)      { return this.req('POST', '/api/v1/reports/ir2', body) }
  reportIr1(body: Record<string, unknown>)      { return this.req('POST', '/api/v1/reports/ir1', body) }
  reportTss(periodo: string, empleados: unknown[]) { return this.req('POST', '/api/v1/reports/tss', { periodo, empleados }) }

  // ── Empresa ─────────────────────────────────────────────────────────────────
  getConfig()                                  { return this.req('GET', '/api/v1/company/config') }
  updateConfig(cfg: Record<string, unknown>)   { return this.req('PUT', '/api/v1/company/config', cfg) }

  // ── Núcleo HTTP ───────────────────────────────────────────────────────────
  private qs(params: Record<string, unknown>): string {
    const entries = Object.entries(params).filter(([, v]) => v != null && v !== '')
    if (!entries.length) return ''
    return '?' + entries.map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(String(v))}`).join('&')
  }

  private raw(method: string, path: string, body?: unknown): Promise<Response> {
    const ctrl = new AbortController()
    const timer = setTimeout(() => ctrl.abort(), this.timeoutMs)
    const headers: Record<string, string> = { 'X-API-Key': this.apiKey, Accept: 'application/json' }
    if (body !== undefined) headers['Content-Type'] = 'application/json'
    return fetch(this.baseUrl + path, {
      method, headers,
      body: body !== undefined ? JSON.stringify(body) : undefined,
      signal: ctrl.signal,
    }).finally(() => clearTimeout(timer))
  }

  private async req<T = any>(method: string, path: string, body?: unknown): Promise<T> {
    const res = await this.raw(method, path, body)
    const text = await res.text()
    const data = text ? JSON.parse(text) : {}
    if (!res.ok) {
      throw new CaribeFiscalError(data?.error ?? `HTTP ${res.status}`, res.status, data)
    }
    return data as T
  }
}
