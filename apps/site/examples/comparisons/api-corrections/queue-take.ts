export class QueueClosed extends Error {}
export interface AsyncQueue<T> {
  take(): Promise<T>
}
export async function takeText(queue: AsyncQueue<number>): Promise<string> {
  try {
    return String(await queue.take())
  } catch (error) {
    if (error instanceof QueueClosed) return "closed"
    throw error
  }
}
