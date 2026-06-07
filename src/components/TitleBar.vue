<script setup lang="ts">
// 与软件融为一体的自定义标题栏：透出主题暖光、左侧品牌、右侧窗口控制。
import { ref, onMounted } from "vue";
import { isTauri } from "../tauri";
import BrandLogo from "./BrandLogo.vue";

const maximized = ref(false);
let win: any = null;

onMounted(async () => {
  if (!isTauri) return;
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    win = getCurrentWindow();
    maximized.value = await win.isMaximized();
    await win.onResized(async () => {
      try { maximized.value = await win.isMaximized(); } catch { /* noop */ }
    });
  } catch {
    win = null;
  }
});

async function minimize() { try { await win?.minimize(); } catch { /* noop */ } }
async function toggleMax() {
  try { await win?.toggleMaximize(); maximized.value = await win.isMaximized(); } catch { /* noop */ }
}
async function close() { try { await win?.close(); } catch { /* noop */ } }
</script>

<template>
  <div class="titlebar" data-tauri-drag-region>
    <div class="tb-brand" data-tauri-drag-region>
      <BrandLogo :size="22" :radius="6" />
      <span class="tb-name">高智愿</span>
      <span class="tb-sub">智能填报 · 冲稳保</span>
    </div>
    <div class="tb-ctrls">
      <button class="tb-btn" title="最小化" @click="minimize">
        <svg viewBox="0 0 12 12"><rect x="2" y="5.4" width="8" height="1.2" /></svg>
      </button>
      <button class="tb-btn" :title="maximized ? '还原' : '最大化'" @click="toggleMax">
        <svg v-if="!maximized" viewBox="0 0 12 12"><rect x="2.4" y="2.4" width="7.2" height="7.2" fill="none" stroke-width="1.2" /></svg>
        <svg v-else viewBox="0 0 12 12"><rect x="3.4" y="2.2" width="6" height="6" fill="none" stroke-width="1.1" /><rect x="2.2" y="3.4" width="6" height="6" fill="none" stroke-width="1.1" /></svg>
      </button>
      <button class="tb-btn close" title="关闭" @click="close">
        <svg viewBox="0 0 12 12"><path d="M2.6 2.6 L9.4 9.4 M9.4 2.6 L2.6 9.4" stroke-width="1.3" /></svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  height: 38px;
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 6px 0 14px;
  /* 透明 → 透出 #app 的暖光画布，与软件融为一体 */
  background: transparent;
  user-select: none;
  -webkit-user-select: none;
}
.tb-brand { display: flex; align-items: center; gap: 9px; height: 100%; }
.tb-name { font-family: var(--serif); font-size: 14.5px; font-weight: 800; color: var(--ink); letter-spacing: .5px; }
.tb-sub { font-size: 11px; color: var(--muted); letter-spacing: .3px; }
.tb-ctrls { display: flex; gap: 2px; -webkit-app-region: no-drag; }
.tb-btn {
  width: 38px; height: 30px; border: none; background: transparent; border-radius: 7px;
  display: flex; align-items: center; justify-content: center; color: var(--text-2); transition: .14s;
}
.tb-btn svg { width: 12px; height: 12px; stroke: currentColor; fill: currentColor; }
.tb-btn svg rect[fill="none"], .tb-btn svg path { fill: none; stroke: currentColor; }
.tb-btn:hover { background: var(--selection-bg); color: var(--ink); }
.tb-btn.close:hover { background: #e0483b; color: #fff; }
</style>
