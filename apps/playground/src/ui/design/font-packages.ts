// 自前で配信するフォント。fonts.css の @import と、ビルドで出すライセンス文の一覧です。
export type FontPackage = Readonly<{
  packageName: string
  family: string
  licenseFile: string
}>

export const fontPackages: readonly FontPackage[] = [
  {
    packageName: "@fontsource-variable/geist",
    family: "Geist Variable",
    licenseFile: "fonts/LICENSE-Geist.txt",
  },
  {
    packageName: "@fontsource-variable/geist-mono",
    family: "Geist Mono Variable",
    licenseFile: "fonts/LICENSE-GeistMono.txt",
  },
  {
    packageName: "@fontsource-variable/noto-sans-jp",
    family: "Noto Sans JP Variable",
    licenseFile: "fonts/LICENSE-NotoSansJP.txt",
  },
]
