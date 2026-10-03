import { en, type Messages } from "./en";
import { ptBR } from "./pt-BR";

// Keys are BCP 47 language tags. A tag matches exactly first, then by its
// primary language ("pt-PT" falls back to "pt-BR"), then English.
const catalogs: Record<string, Messages> = {
  en,
  "pt-BR": ptBR,
};

/** The supported language tag that best matches the user's preferences. */
export function pickLanguage(languages: readonly string[]): string {
  for (const lang of languages) {
    if (catalogs[lang]) return lang;
    const primary = lang.split("-")[0].toLowerCase();
    const match = Object.keys(catalogs).find((tag) => tag.split("-")[0].toLowerCase() === primary);
    if (match) return match;
  }
  return "en";
}

export function pickMessages(languages: readonly string[]): Messages {
  return catalogs[pickLanguage(languages)];
}

export const language: string = pickLanguage(navigator.languages ?? [navigator.language]);
export const messages: Messages = catalogs[language];
export type { Messages };
