export class BorshWriter {
  private chunks: Uint8Array[] = []

  u8(v: number | bigint): this {
    this.chunks.push(Uint8Array.of(Number(v) & 0xff))
    return this
  }

  bool(v: boolean): this {
    return this.u8(v ? 1 : 0)
  }

  u32(v: number | bigint): this {
    const b = new Uint8Array(4)
    new DataView(b.buffer).setUint32(0, Number(v) >>> 0, true)
    this.chunks.push(b)
    return this
  }

  u64(v: number | bigint): this {
    const b = new Uint8Array(8)
    new DataView(b.buffer).setBigUint64(0, BigInt(v), true)
    this.chunks.push(b)
    return this
  }

  i64(v: number | bigint): this {
    const b = new Uint8Array(8)
    new DataView(b.buffer).setBigInt64(0, BigInt(v), true)
    this.chunks.push(b)
    return this
  }

  str(s: string): this {
    const bytes = new TextEncoder().encode(s)
    this.u32(bytes.length)
    this.chunks.push(bytes)
    return this
  }

  toBuffer(): Buffer {
    const total = this.chunks.reduce((n, c) => n + c.length, 0)
    const out = new Uint8Array(total)
    let off = 0
    for (const c of this.chunks) {
      out.set(c, off)
      off += c.length
    }
    return Buffer.from(out)
  }
}
