import type { Locale, PreparedDocBlock, PreparedPage } from "../content/model"

export type HighlightPart = { text: string; className: string }

export type SiteBlock = PreparedDocBlock & {
  anchorId: string
  signature: HighlightPart[]
  playgroundUrl: string
}

export type SidebarPage = {
  title: string
  route: string
  active: boolean
}

export type SidebarSection = {
  title: string
  active: boolean
  pages: SidebarPage[]
}

export type SidebarGroup = {
  title: string
  active: boolean
  sections: SidebarSection[]
}

export type SidebarArea = {
  id: string
  title: string
  description: string
  route: string
  active: boolean
  groups: SidebarGroup[]
}

export type SitePage = {
  route: string
  locale: Locale
  kind: string
  title: string
  summary: string
  alternateRoute: string
  breadcrumbs: SidebarPage[]
  sidebar: SidebarArea[]
  toc: SidebarPage[]
  previousTitle: string
  previousRoute: string
  nextTitle: string
  nextRoute: string
  blocks: SiteBlock[]
}

export type SiteInput = {
  schema: 1
  origin: string
  base: string
  playgroundUrl: string
  pages: SitePage[]
}

export type NavigationSource = {
  schema: 1
  areas: Array<{
    id: string
    title: Record<Locale, string>
    description: Record<Locale, string>
    groups: Array<{
      id: string
      title: Record<Locale, string>
      sections: Array<{
        id: string
        title: Record<Locale, string>
        pages: string[]
      }>
    }>
  }>
}

export type PreparedPageByLocale = Map<string, PreparedPage>
