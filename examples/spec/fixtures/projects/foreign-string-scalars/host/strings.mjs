export const valid = () => "A😀𠮷\0\ufeffe\u0301"
export const invalidHigh = async () => String.fromCharCode(0xd800)
export const invalidLow = () => String.fromCharCode(0xdc00)
export const nested = async () => [{ text: String.fromCharCode(0xd800) }]
export const callback = async (f) => f(String.fromCharCode(0xdfff))
export const raw = () => String.fromCharCode(0xd800)
export const inspectRaw = (value) => value.charCodeAt(0)
