export function syncThrow() {
  throw new Error("sync boom");
}

export async function promiseReject() {
  throw new Error("async boom");
}
