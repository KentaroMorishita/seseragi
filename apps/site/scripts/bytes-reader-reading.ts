type Reading = { en: string; ja: string }
type Identity = {
  identity: string
  module: string
  namespace: string
  kind: string
}
const readings: Record<string, Reading> = {
  "std/bytes::ByteError": {
    en: "ByteError is the error type for an integer that cannot fit in a byte. A failed conversion returns Left containing ByteOutOfRange with that integer. Match ByteOutOfRange value to read it; fromInts does not return partial Bytes.",
    ja: "ByteErrorは、バイトに収まらない整数を示すエラーの型です。変換が失敗すると、その整数を持つByteOutOfRangeがLeftに入って返ります。ByteOutOfRange valueをmatchに書くと、その整数を読み取れます。fromIntsは途中までのBytesを返しません。",
  },
  "std/text::Utf8DecodeError": {
    en: "Utf8DecodeError is the error type for invalid UTF-8. decodeUtf8 returns Left containing InvalidUtf8 with an offset field. Match InvalidUtf8 { offset } to read the zero-based byte position where the first invalid sequence starts.",
    ja: "Utf8DecodeErrorは、不正なUTF-8を示すエラーの型です。decodeUtf8が失敗すると、offsetフィールドを持つInvalidUtf8がLeftに入って返ります。InvalidUtf8 { offset }をmatchに書くと、最初の不正な並びが始まるバイト位置を読み取れます。先頭の位置は0です。",
  },
  "std/bytes/hex::HexDecodeError": {
    en: "HexDecodeError has two public alternatives. OddHexLength carries the input length in UTF-8 bytes; InvalidHexDigit { offset } carries the zero-based byte position of an invalid digit. Use match to read the corresponding payload.",
    ja: "HexDecodeErrorには公開された二つの形があります。OddHexLengthは入力のUTF-8バイト数を持ち、InvalidHexDigit { offset }は不正な文字のバイト位置を持ちます。先頭の位置は0です。matchで対応する中身を読み取れます。",
  },
  "std/bytes/base64::Base64DecodeError": {
    en: "Base64DecodeError has four public alternatives. InvalidBase64Length carries the input byte length. InvalidBase64Digit, InvalidBase64Padding and NonCanonicalBase64Bits each carry an offset field. Match the alternative to read its information; offsets count bytes from zero.",
    ja: "Base64DecodeErrorには公開された四つの形があります。InvalidBase64Lengthは入力のバイト数を持ちます。InvalidBase64Digit、InvalidBase64Padding、NonCanonicalBase64Bitsはoffsetフィールドを持ちます。matchでそれぞれの情報を読み取れます。offsetは先頭を0とするバイト位置です。",
  },
}
export function bytesReaderReading(item: Identity, fallback: Reading): Reading {
  if (item.namespace !== "type" || item.kind !== "opaque-type") return fallback
  if (!item.identity.startsWith(`${item.module}::`)) return fallback
  return readings[item.identity] ?? fallback
}
