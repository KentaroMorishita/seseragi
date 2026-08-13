const repositoryBrand = `<p align="center">
  <img src="../../assets/brand/source/seseragi-icon.svg" alt="Seseragi symbol" width="180">
</p>

`

export function packagedReadme(source: string): string {
  const normalized = source.replaceAll("\r\n", "\n")
  if (!normalized.includes(repositoryBrand)) {
    throw new Error("extension README is missing the repository brand block")
  }
  return normalized.replace(repositoryBrand, "")
}
