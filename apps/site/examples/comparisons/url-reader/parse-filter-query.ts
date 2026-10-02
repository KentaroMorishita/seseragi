export {}
console.log(
  new URLSearchParams("?q=tea%20cake&literal=%2b&path=%2f&empty").toString()
)
console.log(
  new URLSearchParams("q=tea+cake&literal=%2B&path=%2F&empty=").toString()
)
