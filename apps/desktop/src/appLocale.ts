export function detectAppLocale(): string | null {
  const intlLocale = Intl.DateTimeFormat().resolvedOptions().locale?.trim();
  if (intlLocale) {
    return intlLocale;
  }

  const navigatorLocale = globalThis.navigator?.language?.trim();
  return navigatorLocale || null;
}
