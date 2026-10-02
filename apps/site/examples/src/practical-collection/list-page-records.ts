const labels = ["Read", "Review", "Ship"]
console.log(JSON.stringify(labels.slice(1, 3)))
for (const count of [-1, 0, 3, 4])
  console.log(JSON.stringify(labels.slice(0, Math.max(0, count))))
for (const count of [-1, 0, 3, 4])
  console.log(JSON.stringify(labels.slice(Math.max(0, count))))
console.log(JSON.stringify([].slice(0, 2)))
console.log(JSON.stringify([].slice(2)))
console.log(JSON.stringify(labels))
export {}
