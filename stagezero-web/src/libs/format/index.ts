export function displayUrl(url: string | null | undefined): string {
  if (!url) {
    return ''
  }
  return url.replace(/^https?:\/\/(www\.)?/i, '').replace(/\/$/, '')
}

export function safeUrl(url: string | null | undefined): string | null {
  return url && /^https?:\/\//i.test(url) ? url : null
}

export function ycCompanyUrl(slug: string): string {
  return `https://www.ycombinator.com/companies/${encodeURIComponent(slug)}`
}

export function humanize(value: string): string {
  const spaced = value.replace(/_/g, ' ').toLowerCase()
  return spaced.charAt(0).toUpperCase() + spaced.slice(1)
}
