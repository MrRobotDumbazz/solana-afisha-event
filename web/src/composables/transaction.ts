import { ref, type Ref } from 'vue'
import { Transaction, ComputeBudgetProgram, type PublicKey, type TransactionInstruction } from '@solana/web3.js'
import type { WalletStore } from '@solana/wallet-adapter-vue'
import { connection } from '../api'

interface WalletSender {
  publicKey: Ref<PublicKey | null>
  sendTransaction: WalletStore['sendTransaction']
}

export function useTransactions() {
  const pending = ref(false)
  const error = ref<string | null>(null)

  async function send(
    wallet: WalletSender,
    instructions: TransactionInstruction[],
    { computeUnits = 300000 }: { computeUnits?: number } = {},
  ): Promise<string> {
    const publicKey: Ref<PublicKey | null> = wallet.publicKey
    if (!publicKey.value) {
      throw new Error('Кошелёк не подключён')
    }
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
      tx.feePayer = publicKey.value
      const signature = await wallet.sendTransaction(tx, connection)
      await connection.confirmTransaction({ signature, blockhash, lastValidBlockHeight }, 'confirmed')
      return signature
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    } finally {
      pending.value = false
    }
  }

  return { pending, error, send }
}
