import { upper } from "../support.mjs";

console.log("host:load");

export function call(label) {
  console.log(`host:call:${label}`);
  return upper(label);
}

export const Nested = {
  async callAsync(label) {
    console.log(`host:call:${label}`);
    return upper(label);
  },
};
