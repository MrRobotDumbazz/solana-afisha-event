import { ref } from 'vue'
import { Transaction, ComputeBudgetProgram } from '@solana/web3.js'
import { connection } from '../api'

export function useTransactions() {
  const pending = ref(false)
  const error = ref(null)

  async function send(wallet, instructions, { computeUnits = 300000 } = {}) {
    if (!wallet.publicKey.value) throw new Error('Кошелёк не подключён')
    error.value = null
    pending.value = true
    try {
      const tx = new Transaction()
      if (computeUnits) {
        tx.add(ComputeBudgetProgram.setComputeUnitLimit({ units: computeUnits }))
      }
      for (const ix of instructions) tx.add(ix)
      const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash('confirmed')
      tx.recentBlockhash = blockhash
      tx.feePayer = wallet.publicKey.value
      const signature = await wallet.sendTransaction(tx, connection)
      await connection.confirmTransaction({ signature, blockhash, lastValidBlockHeight }, 'confirmed')
      return signature
    } catch (e) {
      error.value = e?.message || String(e)
      throw e
    } finally {
      pending.value = false
    }
  }

  return { pending, error, send }
}
