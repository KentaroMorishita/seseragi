function prepare(title: string): string {
  return `Preview: ${title}`
}
const work = () => prepare("Search improvements")
console.log("Before execution")
console.log(work())
console.log(work())
export {}
