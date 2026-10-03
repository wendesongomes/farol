import { describe, expect, it } from "vitest";
import { en } from "./en";
import { ptBR } from "./pt-BR";
import { pickMessages } from "./index";

describe("pickMessages", () => {
  it("matches an exact language tag", () => {
    expect(pickMessages(["pt-BR"])).toBe(ptBR);
  });

  it("falls back to the primary language", () => {
    expect(pickMessages(["pt-PT"])).toBe(ptBR);
  });

  it("uses the first supported language in order of preference", () => {
    expect(pickMessages(["de-DE", "pt-BR", "en-US"])).toBe(ptBR);
  });

  it("falls back to English", () => {
    expect(pickMessages(["ja-JP"])).toBe(en);
    expect(pickMessages([])).toBe(en);
  });
});

describe("catalogs", () => {
  it("pluralizes the port count", () => {
    expect(en.portCount(1)).toBe("1 open port");
    expect(en.portCount(3)).toBe("3 open ports");
    expect(ptBR.portCount(1)).toBe("1 porta aberta");
  });
});
