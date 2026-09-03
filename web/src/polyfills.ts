import { Buffer } from 'buffer'

if (typeof globalThis.Buffer === 'undefined') {
  ;(globalThis as { Buffer: typeof Buffer }).Buffer = Buffer
}
