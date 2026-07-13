export function requiredElement<T extends HTMLElement>(
  selector: string,
  elementType: { new (): T }
): T {
  const element = document.querySelector(selector)
  if (!(element instanceof elementType)) {
    throw new Error(`missing playground element: ${selector}`)
  }
  return element
}
