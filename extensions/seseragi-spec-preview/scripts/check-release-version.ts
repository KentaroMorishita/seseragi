import manifest from "../package.json"

const tag = process.env.GITHUB_REF_NAME || process.argv[2]
const expected = `vscode-v${manifest.version}`
if (tag !== expected) {
  throw new Error(`release tag ${tag || "<missing>"} must equal ${expected}`)
}
console.log(`Release tag ${tag} matches extension ${manifest.version}.`)
