export {}
type Settings = { name: string; retries: number }
function writeSettings(value: Settings): string {
  return JSON.stringify({ name: value.name, retries: value.retries })
}
console.log(writeSettings({ name: "Mio", retries: 2 }))
console.log(writeSettings({ name: 'Line\n"two"', retries: 0 }))
