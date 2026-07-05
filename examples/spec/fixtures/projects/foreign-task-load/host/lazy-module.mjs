console.log("host:load");

export function call(label) {
  console.log(`host:call:${label}`);
  return label.toUpperCase();
}
