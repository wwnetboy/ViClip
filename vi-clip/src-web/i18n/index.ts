import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import zhCN from "./zh-CN.json";
import zhHK from "./zh-HK.json";
import zhTW from "./zh-TW.json";
import enUS from "./en-US.json";
import jaJP from "./ja-JP.json";
import deDE from "./de-DE.json";
import frFR from "./fr-FR.json";
import esES from "./es-ES.json";
import itIT from "./it-IT.json";
import ptBR from "./pt-BR.json";
import ruRU from "./ru-RU.json";
import koKR from "./ko-KR.json";
import thTH from "./th-TH.json";
import viVN from "./vi-VN.json";
import idID from "./id-ID.json";
import msMY from "./ms-MY.json";
import hiIN from "./hi-IN.json";

i18n.use(initReactI18next).init({
  resources: {
    "zh-CN": { translation: zhCN },
    "zh-HK": { translation: zhHK },
    "zh-TW": { translation: zhTW },
    "en-US": { translation: enUS },
    "ja-JP": { translation: jaJP },
    "de-DE": { translation: deDE },
    "fr-FR": { translation: frFR },
    "es-ES": { translation: esES },
    "it-IT": { translation: itIT },
    "pt-BR": { translation: ptBR },
    "ru-RU": { translation: ruRU },
    "ko-KR": { translation: koKR },
    "th-TH": { translation: thTH },
    "vi-VN": { translation: viVN },
    "id-ID": { translation: idID },
    "ms-MY": { translation: msMY },
    "hi-IN": { translation: hiIN },
  },
  lng: "zh-CN",
  fallbackLng: "zh-CN",
  interpolation: { escapeValue: false },
});

export default i18n;
