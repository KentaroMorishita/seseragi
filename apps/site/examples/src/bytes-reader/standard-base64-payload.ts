export {}
for (const label of ["", "f", "fo", "foo", "foob", "fooba", "foobar"])
  console.log(new TextEncoder().encode(label).toBase64())
