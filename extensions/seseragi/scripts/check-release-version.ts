import {
  checkReleaseTag,
  readReleaseContract,
} from "../../../scripts/release-contract"

const tag = process.env.GITHUB_REF_NAME || process.argv[2]
await checkReleaseTag(tag)
const { version } = await readReleaseContract()
console.log(`Release tag ${tag} matches Seseragi ${version}.`)
