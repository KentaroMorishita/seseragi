# Homepage hero spacing correction

## Observed production behavior

On production commit `9c39e1c7aff4ce4ec61da5d5d70de0fb41064dc2`, the Japanese homepage at `/ja/` was inspected in a browser at 1180 × 757 with scroll position zero. The hero grid used `align-items: center` alongside a tall example column. The introduction began around y=541, its heading around y=575, and the first-run action around y=798, below the initial viewport.

The example article also inherited 48px of top padding, and its first direct heading inherited about 23px top margin and 38px top padding. This spacing is useful in long reference articles but creates an unnecessary initial gap in the homepage hero.

## Bounded correction

- Top-align the two hero grid children
- Remove the hero example article's initial top padding
- Remove top margin and padding only from that article's first direct heading
- Preserve content, branding, responsive breakpoints, other article spacing, and anchor scroll margins

Only `apps/site/styles/home.css` and the corresponding assertions in `apps/site/tests/entrance.test.ts` change product/test behavior.

## Checks after the correction

- Entrance suite: 2 tests passed, 87 assertions, including English and Japanese rendering
- Biome passed for both changed files
- Focused TypeScript checking passed for `entrance.test.ts`
- Git whitespace checking passed

The content baseline is local checkpoint 8 (`10b3f66071181972f9217f35408e976f4ab260c1`), whose independent full-site checks are recorded separately. The CSS correction uses the narrower checks above; those are not a new claim that the entire checkpoint-8 suite was rerun after this layout-only change.

Actual post-correction browser geometry and initial-viewport action visibility remain to be checked after publication. The prior narrow-window check was a desktop browser at 500 × 757, not a real mobile-device test.
