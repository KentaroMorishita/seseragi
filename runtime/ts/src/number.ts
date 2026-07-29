export type RoundingMode =
  | Readonly<{ readonly tag: "HalfEven" }>
  | Readonly<{ readonly tag: "HalfUp" }>
  | Readonly<{ readonly tag: "TowardZero" }>
  | Readonly<{ readonly tag: "AwayFromZero" }>
  | Readonly<{ readonly tag: "Floor" }>
  | Readonly<{ readonly tag: "Ceiling" }>

export const HalfEven: RoundingMode = Object.freeze({ tag: "HalfEven" })
export const HalfUp: RoundingMode = Object.freeze({ tag: "HalfUp" })
export const TowardZero: RoundingMode = Object.freeze({ tag: "TowardZero" })
export const AwayFromZero: RoundingMode = Object.freeze({
  tag: "AwayFromZero",
})
export const Floor: RoundingMode = Object.freeze({ tag: "Floor" })
export const Ceiling: RoundingMode = Object.freeze({ tag: "Ceiling" })
