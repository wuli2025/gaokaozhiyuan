<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "../tauri";

const status = ref<any>(null);
onMounted(async () => {
  try {
    status.value = await invoke<any>("sql_tool_status");
  } catch {
    status.value = null;
  }
});

const RULES = [
  { t: "位次优先", d: "分数每年随难度浮动，位次（全省排名）才是稳定标尺。我们一律以位次比对。" },
  { t: "选科双闸", d: "第一道闸看首选（物理 / 历史）分轨；第二道闸看再选科目是否满足专业组要求。" },
  { t: "冲稳保分档", d: "你的位次明显优于专业去年最低位次→可保底；接近→较稳妥；不足→需冲刺。" },
  { t: "平行志愿", d: "分数（位次）优先、遵循志愿、一轮投档。位次越高越先被检索，按你填的顺序投档。" },
  { t: "服从调剂", d: "进了院校组但专业满额时，服从调剂可避免退档；不服从有滑档风险。" },
  { t: "招生计划", d: "录取以当年招生计划为准，参考往年位次但需留出波动余量。" },
];
</script>

<template>
  <div class="page">
    <header class="ph">
      <h1>规则 · 数据透明</h1>
      <p>这里讲清我们怎么算——可报靠数据库查询，冲稳保靠位次比对，不是黑盒。</p>
    </header>

    <div class="db-bar" v-if="status?.ok">
      <span class="db-dot"></span>
      <span>数据已连通</span>
      <span class="db-stat" v-if="status.stats">院校 {{ status.stats.school }}</span>
      <span class="db-stat" v-if="status.stats">专业 {{ status.stats.major }}</span>
      <span class="db-stat" v-if="status.stats">录取位次 {{ status.stats.admission_min_rank?.toLocaleString() }}</span>
      <span class="db-stat" v-if="status.stats">招生计划 {{ status.stats.plan?.toLocaleString() }}</span>
    </div>
    <div class="db-bar warn" v-else>
      <span>当前为预览模式，未连接本地数据库</span>
    </div>

    <div class="rule-grid">
      <div v-for="(r, i) in RULES" :key="r.t" class="rule">
        <div class="r-no">{{ String(i + 1).padStart(2, "0") }}</div>
        <div class="r-body">
          <div class="r-t">{{ r.t }}</div>
          <div class="r-d">{{ r.d }}</div>
        </div>
      </div>
    </div>

    <div class="formula">
      <div class="f-title">录取概率怎么算</div>
      <p>设你的位次为 <code>R</code>、专业去年最低录取位次为 <code>M</code>，差值比例 <code>d = (M − R) / R</code>。</p>
      <p>概率 <code>p = 1 / (1 + e<sup>−d/0.16</sup>)</code>：你比往年录取线越靠前，<code>d</code> 越大，概率越高。</p>
      <p class="f-foot">概率 ≥ 78% 记为「保」，45%–78% 为「稳」，&lt; 45% 为「冲」。</p>
    </div>
  </div>
</template>

<style scoped>
.page { height: 100vh; overflow-y: auto; max-width: 820px; margin: 0 auto; padding: 36px 28px 60px; }
.ph h1 { font-family: var(--serif); font-size: 26px; margin: 0; color: var(--ink); }
.ph p { color: var(--muted); font-size: 13.5px; margin: 8px 0 22px; }
.db-bar { display: flex; align-items: center; gap: 12px; background: #fff; border: 1px solid var(--border-soft); border-radius: 12px; padding: 12px 18px; font-size: 12.5px; color: var(--text-2); margin-bottom: 22px; }
.db-bar.warn { color: var(--gold); }
.db-dot { width: 8px; height: 8px; border-radius: 50%; background: #1f9d6b; box-shadow: 0 0 0 3px rgba(31,157,107,.15); }
.db-stat { color: var(--muted); }
.db-stat::before { content: "·"; margin-right: 8px; color: var(--dim); }
.rule-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 12px; margin-bottom: 24px; }
.rule { display: flex; gap: 14px; background: #fff; border: 1px solid var(--border-soft); border-radius: 14px; padding: 16px 18px; box-shadow: var(--shadow-sm); }
.r-no { font-family: var(--mono); font-size: 20px; font-weight: 800; color: var(--primary); opacity: .35; }
.r-t { font-size: 14px; font-weight: 700; color: var(--ink); }
.r-d { font-size: 12.5px; color: var(--text-2); margin-top: 5px; line-height: 1.7; }
.formula { background: linear-gradient(160deg, #1d2530, #161c24); color: #d7e0ea; border-radius: 16px; padding: 24px 26px; }
.f-title { font-size: 15px; font-weight: 800; color: #fff; margin-bottom: 12px; }
.formula p { font-size: 13px; line-height: 1.9; margin: 6px 0; }
.formula code { background: rgba(255,255,255,.1); color: #ffd9a8; padding: 1px 7px; border-radius: 5px; font-family: var(--mono); font-size: 12px; }
.f-foot { color: #9fb0c2; margin-top: 12px; }
@media (max-width: 720px) { .rule-grid { grid-template-columns: 1fr; } }
</style>
