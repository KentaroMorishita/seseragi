export type RoundingMode =
  | Readonly<{ readonly tag: "HalfEven" }>
  | Readonly<{ readonly tag: "HalfUp" }>
  | Readonly<{ readonly tag: "TowardZero" }>
  | Readonly<{ readonly tag: "AwayFromZero" }>
  | Readonly<{ readonly tag: "Floor" }>
  | Readonly<{ readonly tag: "Ceiling" }>

export const HalfEven: RoundingMode = /* @__PURE__ */ Object.freeze({
  tag: "HalfEven",
})
export const HalfUp: RoundingMode = /* @__PURE__ */ Object.freeze({
  tag: "HalfUp",
})
export const TowardZero: RoundingMode = /* @__PURE__ */ Object.freeze({
  tag: "TowardZero",
})
export const AwayFromZero: RoundingMode = /* @__PURE__ */ Object.freeze({
  tag: "AwayFromZero",
})
export const Floor: RoundingMode = /* @__PURE__ */ Object.freeze({
  tag: "Floor",
})
export const Ceiling: RoundingMode = /* @__PURE__ */ Object.freeze({
  tag: "Ceiling",
})
