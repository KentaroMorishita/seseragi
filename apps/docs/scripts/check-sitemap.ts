import { resolve } from "node:path"
import { validateRepositorySitemap } from "./sitemap"

const report = validateRepositorySitemap(resolve(import.meta.dir, "../../.."))
console.log(
  `Docs sitemap: ${report.specSections} sections across ` +
    `${report.specFiles} spec chapters -> ${report.routes} canonical routes`
)
