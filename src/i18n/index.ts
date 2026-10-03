import { en, type Messages } from "./en";
import { ptBR } from "./pt-BR";

// Keys are BCP 47 language tags. A tag matches exactly first, then by its
// primary language ("pt-PT" falls back to "pt-BR"), then English.
const catalogs: Record<string, Messages> = {
  en,
  "pt-BR": ptBR,
};

export function pickMessages(languages: readonly string[]): Messages {
  for (const lang of languages) {
    if (catalogs[lang]) return catalogs[lang];
    const primary = lang.split("-")[0].toLowerCase();
    const match = Object.keys(catalogs).find((tag) => tag.split("-")[0].toLowerCase() === primary);
    if (match) return catalogs[match];
  }
  return en;
}

export const messages: Messages = pickMessages(navigator.languages ?? [navigator.language]);
export type { Messages };
