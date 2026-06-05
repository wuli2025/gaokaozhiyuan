import { defineStore } from "pinia";
import { ref } from "vue";

/* =============================================================================
   主题系统 · 切换 / 持久化 / 元数据
   -----------------------------------------------------------------------------
   令牌定义在 src/style.css 的 :root[data-theme="..."] 中。
   这里只负责：写 documentElement 的 data-theme、记忆选择、提供给设置页渲染。
   默认主题 = festive（鎏金朱砂 · 喜庆）。
   ========================================================================== */

export type ThemeKey = "festive" | "celadon" | "inkblue";

export interface ThemeMeta {
  key: ThemeKey;
  /** 主名（中文，雅称） */
  name: string;
  /** 副名（气质标签） */
  alias: string;
  /** 一句话主张 */
  tagline: string;
  /** 预览顶部渐变条 */
  bar: string;
  /** 预览纸面底色（含微光） */
  paper: string;
  /** 三枚色点：主色 / 辅色 / 金或亮色 */
  dots: [string, string, string];
  /** 是否为默认 / 推荐 */
  recommended?: boolean;
}

export const THEMES: ThemeMeta[] = [
  {
    key: "festive",
    name: "鎏金朱砂",
    alias: "喜庆 · 默认",
    tagline: "暖红描金，喜庆而沉稳——为一生一次的选择配一抹好彩头。",
    bar: "linear-gradient(120deg, #d23a2c 0%, #ff6b47 45%, #ffb14d 100%)",
    paper:
      "radial-gradient(120px 80px at 18% 0%, #fde6d8 0%, transparent 60%), #fffdfb",
    dots: ["#d23a2c", "#ff6b47", "#ffb14d"],
    recommended: true,
  },
  {
    key: "celadon",
    name: "青瓷雅致",
    alias: "雨过天青",
    tagline: "克制的冷调青绿，安稳耐看，长时间阅读不累眼。",
    bar: "linear-gradient(120deg, #1f6e52 0%, #4fa885 48%, #7cc6a3 100%)",
    paper:
      "radial-gradient(120px 80px at 18% 0%, #dceee4 0%, transparent 60%), #fffffe",
    dots: ["#1f5e46", "#4fa885", "#7cc6a3"],
  },
  {
    key: "inkblue",
    name: "墨蓝沉静",
    alias: "深蓝科技",
    tagline: "深蓝主调，专业可信，透着一点冷静的科技感。",
    bar: "linear-gradient(120deg, #163a5c 0%, #2f6fb0 48%, #5b9bd5 100%)",
    paper:
      "radial-gradient(120px 80px at 18% 0%, #dfe8f6 0%, transparent 60%), #ffffff",
    dots: ["#163a5c", "#2f6fb0", "#5b9bd5"],
  },
];

const STORAGE_KEY = "polaris.theme.v1";
export const DEFAULT_THEME: ThemeKey = "festive";

function isThemeKey(v: unknown): v is ThemeKey {
  return typeof v === "string" && THEMES.some((t) => t.key === v);
}

export function loadThemeKey(): ThemeKey {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (isThemeKey(raw)) return raw;
  } catch {
    /* storage 不可用时回落默认 */
  }
  return DEFAULT_THEME;
}

/** 把主题写到 <html data-theme>；animate=true 时附带一次性丝滑过渡 */
export function applyTheme(key: ThemeKey, animate = false) {
  const el = document.documentElement;
  if (animate) {
    el.classList.add("theme-anim");
    window.setTimeout(() => el.classList.remove("theme-anim"), 450);
  }
  el.setAttribute("data-theme", key);
}

/** 应用启动时调用（在 mount 前，避免主题闪烁） */
export function initTheme() {
  applyTheme(loadThemeKey(), false);
}

export const useThemeStore = defineStore("theme", () => {
  const current = ref<ThemeKey>(loadThemeKey());

  function set(key: ThemeKey) {
    if (!isThemeKey(key) || key === current.value) return;
    current.value = key;
    applyTheme(key, true);
    try {
      localStorage.setItem(STORAGE_KEY, key);
    } catch {
      /* 忽略持久化失败 */
    }
  }

  return { current, set, themes: THEMES };
});
