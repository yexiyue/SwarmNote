import { i18n } from "@lingui/core";

export const locales = { zh: "中文", en: "English" } as const;
export type Locale = keyof typeof locales;

const SOURCE_LOCALE: Locale = "zh";

/**
 * Activate a locale. For the source locale (zh), activates synchronously
 * without loading a catalog. For other locales, dynamically imports the .po file.
 */
export async function activateLocale(locale: Locale) {
  if (i18n.locale === locale) return;

  if (locale === SOURCE_LOCALE) {
    // Source locale uses original strings directly — no catalog needed
    i18n.load(locale, {});
    i18n.activate(locale);
    return;
  }

  const { messages } = await import(`./locales/${locale}/messages.po`);
  i18n.load(locale, messages);
  i18n.activate(locale);
}

/**
 * Synchronously initialize the source locale so the app can render immediately.
 */
export function initI18n() {
  i18n.load(SOURCE_LOCALE, {});
  i18n.activate(SOURCE_LOCALE);
}

export function detectLocale(): Locale {
  const lang = navigator.language;
  if (lang.startsWith("zh")) return "zh";
  return "en";
}
