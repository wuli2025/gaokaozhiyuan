<script setup lang="ts">
import { useAppStore } from "../stores/app";
const app = useAppStore();
const NETS = [
  { c: "🟦", t: "专业网", d: "学什么 / 选哪个" },
  { c: "🟩", t: "行业网", d: "就业 / 在哪" },
  { c: "🟥", t: "真话网", d: "有什么坑" },
  { c: "🟨", t: "规则网", d: "位次 / 选科 / 体检" },
  { c: "🟪", t: "学生网", d: "我娃的档案" },
  { c: "⬜", t: "案例网", d: "别人怎么选" },
];
</script>

<template>
  <div class="page">
    <header class="ph">
      <h1>知识网络</h1>
      <p>专业、行业、真话、规则、学生、案例 —— 6 张网交叉相连，匹配时沿链取证。</p>
    </header>
    <div class="net-grid">
      <div v-for="n in NETS" :key="n.t" class="net">
        <span class="n-c">{{ n.c }}</span>
        <div class="n-t">{{ n.t }}</div>
        <div class="n-d">{{ n.d }}</div>
      </div>
    </div>
    <button class="graph-btn" @click="app.setView('graph')">打开知识图谱 →</button>
  </div>
</template>

<style scoped>
.page { height: 100vh; overflow-y: auto; max-width: 760px; margin: 0 auto; padding: 36px 28px 60px; }
.ph h1 { font-family: var(--serif); font-size: 26px; margin: 0; color: var(--ink); }
.ph p { color: var(--muted); font-size: 13.5px; margin: 8px 0 24px; }
.net-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; margin-bottom: 24px; }
.net { background: #fff; border: 1px solid var(--border-soft); border-radius: 16px; padding: 22px 18px; text-align: center; box-shadow: var(--shadow-sm); transition: .15s; }
.net:hover { transform: translateY(-3px); box-shadow: var(--shadow-lg); }
.n-c { font-size: 26px; }
.n-t { font-size: 15px; font-weight: 800; color: var(--ink); margin-top: 8px; }
.n-d { font-size: 12px; color: var(--muted); margin-top: 4px; }
.graph-btn { background: var(--ink); color: #fff; border: none; border-radius: 12px; padding: 13px 26px; font-size: 14px; font-weight: 700; }
.graph-btn:hover { background: var(--primary); }
@media (max-width: 640px) { .net-grid { grid-template-columns: 1fr 1fr; } }
</style>
