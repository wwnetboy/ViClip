export type EngineType = "google" | "ai" | "baidu" | "youdao" | "tencent" | "volctrans";

export interface LangDef {
  code: string;
  name: string;
  badge: string;
  /** Omitted engines = not supported. Value = engine-specific code. */
  engines: Partial<Record<EngineType, string>>;
}

export const ALL_LANGUAGES: LangDef[] = [
  { code: "zh",   name: "中文",               badge: "ZH",  engines: { google:"zh", ai:"zh", baidu:"zh",  youdao:"zh-CHS", tencent:"zh",    volctrans:"zh" } },
  { code: "zh-TW",name: "繁體中文",           badge: "TW",  engines: { google:"zh-TW", ai:"zh-TW", baidu:"cht", youdao:"zh-CHT", tencent:"zh-TW", volctrans:"zh-Hant" } },
  { code: "en",   name: "English",            badge: "EN",  engines: { google:"en", ai:"en", baidu:"en",   youdao:"en",     tencent:"en",    volctrans:"en" } },
  { code: "ja",   name: "日本語",             badge: "JA",  engines: { google:"ja", ai:"ja", baidu:"jp",   youdao:"ja",     tencent:"ja",    volctrans:"ja" } },
  { code: "ko",   name: "한국어",             badge: "KO",  engines: { google:"ko", ai:"ko", baidu:"kor",  youdao:"ko",     tencent:"ko",    volctrans:"ko" } },
  { code: "fr",   name: "Français",           badge: "FR",  engines: { google:"fr", ai:"fr", baidu:"fra",  youdao:"fr",     tencent:"fr",    volctrans:"fr" } },
  { code: "de",   name: "Deutsch",            badge: "DE",  engines: { google:"de", ai:"de", baidu:"de",   youdao:"de",     tencent:"de",    volctrans:"de" } },
  { code: "es",   name: "Español",            badge: "ES",  engines: { google:"es", ai:"es", baidu:"spa",  youdao:"es",     tencent:"es",    volctrans:"es" } },
  { code: "ru",   name: "Русский",            badge: "RU",  engines: { google:"ru", ai:"ru", baidu:"ru",   youdao:"ru",     tencent:"ru",    volctrans:"ru" } },
  { code: "pt",   name: "Português",          badge: "PT",  engines: { google:"pt", ai:"pt", baidu:"pt",   youdao:"pt",     tencent:"pt",    volctrans:"pt" } },
  { code: "it",   name: "Italiano",           badge: "IT",  engines: { google:"it", ai:"it", baidu:"it",   youdao:"it",     tencent:"it",    volctrans:"it" } },
  { code: "ar",   name: "العربية",            badge: "AR",  engines: { google:"ar", ai:"ar", baidu:"ara",  youdao:"ar",     tencent:"ar",    volctrans:"ar" } },
  { code: "th",   name: "ไทย",                badge: "TH",  engines: { google:"th", ai:"th", baidu:"th",   youdao:"th",     tencent:"th",    volctrans:"th" } },
  { code: "vi",   name: "Tiếng Việt",         badge: "VI",  engines: { google:"vi", ai:"vi", baidu:"vie",  youdao:"vi",     tencent:"vi",    volctrans:"vi" } },
  { code: "id",   name: "Bahasa Indonesia",   badge: "ID",  engines: { google:"id", ai:"id",                youdao:"id",                      volctrans:"id" } },
  { code: "ms",   name: "Bahasa Melayu",      badge: "MS",  engines: { google:"ms", ai:"ms",                youdao:"ms",                      volctrans:"ms" } },
  { code: "hi",   name: "हिन्दी",             badge: "HI",  engines: { google:"hi", ai:"hi",                youdao:"hi",                      volctrans:"hi" } },
  { code: "tr",   name: "Türkçe",             badge: "TR",  engines: { google:"tr", ai:"tr",                youdao:"tr",     tencent:"tr",    volctrans:"tr" } },
  { code: "nl",   name: "Nederlands",         badge: "NL",  engines: { google:"nl", ai:"nl", baidu:"nl",   youdao:"nl",                                    volctrans:"nl" } },
  { code: "pl",   name: "Polski",             badge: "PL",  engines: { google:"pl", ai:"pl", baidu:"pl",   youdao:"pl",                                    volctrans:"pl" } },
  { code: "sv",   name: "Svenska",            badge: "SV",  engines: { google:"sv", ai:"sv", baidu:"swe",  youdao:"sv",                                    volctrans:"sv" } },
  { code: "da",   name: "Dansk",              badge: "DA",  engines: { google:"da", ai:"da", baidu:"dan",  youdao:"da",                                    volctrans:"da" } },
  { code: "fi",   name: "Suomi",              badge: "FI",  engines: { google:"fi", ai:"fi", baidu:"fin",  youdao:"fi",                                    volctrans:"fi" } },
  { code: "cs",   name: "Čeština",            badge: "CS",  engines: { google:"cs", ai:"cs", baidu:"cs",   youdao:"cs",                                    volctrans:"cs" } },
  { code: "ro",   name: "Română",             badge: "RO",  engines: { google:"ro", ai:"ro", baidu:"rom",  youdao:"ro",                                    volctrans:"ro" } },
  { code: "hu",   name: "Magyar",             badge: "HU",  engines: { google:"hu", ai:"hu", baidu:"hu",   youdao:"hu",                                    volctrans:"hu" } },
  { code: "el",   name: "Ελληνικά",           badge: "EL",  engines: { google:"el", ai:"el", baidu:"el",   youdao:"el",                                    volctrans:"el" } },
  { code: "he",   name: "עברית",              badge: "HE",  engines: { google:"he", ai:"he",                youdao:"he",                                    volctrans:"he" } },
  { code: "uk",   name: "Українська",         badge: "UK",  engines: { google:"uk", ai:"uk",                youdao:"uk",                                    volctrans:"uk" } },
  { code: "no",   name: "Norsk",              badge: "NO",  engines: { google:"no", ai:"no",                youdao:"no",                                    volctrans:"no" } },
  { code: "bg",   name: "Български",          badge: "BG",  engines: { google:"bg", ai:"bg", baidu:"bul",  youdao:"bg",                                    volctrans:"bg" } },
  { code: "hr",   name: "Hrvatski",           badge: "HR",  engines: { google:"hr", ai:"hr",                youdao:"hr",                                    volctrans:"hr" } },
  { code: "sr",   name: "Српски",             badge: "SR",  engines: { google:"sr", ai:"sr",                youdao:"sr",                                    volctrans:"sr" } },
  { code: "sk",   name: "Slovenčina",         badge: "SK",  engines: { google:"sk", ai:"sk",                youdao:"sk",                                    volctrans:"sk" } },
  { code: "tl",   name: "Filipino",           badge: "TL",  engines: { google:"tl", ai:"tl",                youdao:"tl",                                    volctrans:"tl" } },
];

/** Normalize a detected language code (from any engine) to our standard code for i18n lookup. */
export function normalizeLangCode(code: string): string {
  if (ALL_LANGUAGES.some((l) => l.code === code)) return code;
  const found = ALL_LANGUAGES.find((l) =>
    Object.values(l.engines).some((v) => v === code)
  );
  return found ? found.code : code;
}

export function getLanguagesForEngine(engine: string): { code: string; name: string }[] {
  return ALL_LANGUAGES
    .filter((l) => engine in l.engines)
    .map((l) => ({ code: l.code, name: l.name }));
}
