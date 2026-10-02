const display = (value: unknown): void => console.log(JSON.stringify(value))
const groups = [
  { title: "Today", tasks: ["Read", "Review"] },
  { title: "Waiting", tasks: [] },
  { title: "Tomorrow", tasks: ["Ship"] },
]
display(groups.flatMap((group) => group.tasks))

export {}
