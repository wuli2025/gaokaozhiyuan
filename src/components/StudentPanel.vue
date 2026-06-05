<script setup lang="ts">
import { computed } from "vue";
import { useProfileStore } from "../stores/profile";
import { useAppStore } from "../stores/app";

const profile = useProfileStore();
const app = useAppStore();

const tags = computed(() => {
  const out: string[] = [];
  if (profile.province) out.push(profile.province);
  if (profile.track) out.push(profile.track + "类");
  if (profile.reselect.length) out.push("再选 " + profile.reselect.join("、"));
  return out;
});

function goFill() {
  app.setView("match");
}
function reset() {
  if (confirm("确定清空当前档案？")) profile.reset();
}
</script>

<template>
  <div class="page">
    <header class="ph">
      <h1>我的档案</h1>
      <p>你的位次、选科与偏好，是所有匹配的输入源。一年一版，本地保存。</p>
    </header>

    <div v-if="profile.ready" class="card profile">
      <div class="pf-avatar">考</div>
      <div class="pf-body">
        <div class="pf-row">
          <span class="pf-score">{{ profile.score ?? "—" }}<i>分</i></span>
          <span class="pf-rank">位次 ≈ {{ profile.rank?.toLocaleString() }}</span>
        </div>
        <div class="pf-tags">
          <span v-for="t in tags" :key="t" class="pf-tag">{{ t }}</span>
        </div>
      </div>
      <div class="pf-actions">
        <button class="btn primary" @click="goFill">去智能填报 →</button>
        <button class="btn ghost" @click="reset">清空档案</button>
      </div>
    </div>

    <div v-else class="card empty">
      <div class="em-icon">📝</div>
      <h3>还没有建立档案</h3>
      <p>填写省份、分数与选科，系统会用真实录取数据为你换算位次并匹配院校。</p>
      <button class="btn primary big" @click="goFill">立即建立档案</button>
    </div>

    <div class="card tips">
      <h3>位次法 · 为什么看位次而不是分数</h3>
      <ul>
        <li>每年试题难度不同，<b>分数会浮动</b>，但全省排名（位次）相对稳定。</li>
        <li>我们用本省真实的「分数 ↔ 位次」对应关系，把你的分数换算成全省位次。</li>
        <li>再用你的位次对比各院校专业去年的<b>最低录取位次</b>，给出冲 / 稳 / 保。</li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.page { height: 100vh; overflow-y: auto; max-width: 760px; margin: 0 auto; padding: 36px 28px 60px; }
.ph h1 { font-family: var(--serif); font-size: 26px; margin: 0; color: var(--ink); }
.ph p { color: var(--muted); font-size: 13.5px; margin: 8px 0 24px; }
.card { background: #fff; border: 1px solid var(--border-soft); border-radius: 18px; padding: 24px 26px; box-shadow: var(--shadow); margin-bottom: 18px; }
.profile { display: flex; gap: 18px; align-items: center; }
.pf-avatar { width: 56px; height: 56px; border-radius: 16px; background: linear-gradient(135deg, #c0392b, #e0584a); color: #fff; display: flex; align-items: center; justify-content: center; font-size: 24px; font-weight: 800; font-family: var(--serif); flex: none; }
.pf-body { flex: 1; }
.pf-row { display: flex; align-items: baseline; gap: 16px; }
.pf-score { font-size: 32px; font-weight: 800; color: var(--ink); font-family: var(--serif); }
.pf-score i { font-size: 14px; color: var(--muted); font-style: normal; margin-left: 2px; }
.pf-rank { background: var(--primary-soft); color: var(--primary-deep); padding: 3px 12px; border-radius: 999px; font-size: 13px; font-weight: 700; }
.pf-tags { display: flex; gap: 7px; margin-top: 10px; flex-wrap: wrap; }
.pf-tag { background: var(--bg-soft); color: var(--text-2); padding: 3px 11px; border-radius: 7px; font-size: 12px; }
.pf-actions { display: flex; flex-direction: column; gap: 8px; }
.btn { border: none; border-radius: 10px; padding: 10px 18px; font-size: 13px; font-weight: 700; cursor: pointer; }
.btn.primary { background: var(--primary); color: #fff; }
.btn.primary.big { padding: 13px 28px; font-size: 14.5px; margin-top: 16px; }
.btn.ghost { background: #fff; border: 1px solid var(--border); color: var(--muted); }
.empty { text-align: center; padding: 48px 26px; }
.em-icon { font-size: 40px; }
.empty h3 { margin: 14px 0 6px; color: var(--ink); }
.empty p { color: var(--muted); font-size: 13px; margin: 0; }
.tips h3 { font-size: 15px; color: var(--ink); margin: 0 0 12px; }
.tips ul { margin: 0; padding-left: 18px; color: var(--text-2); font-size: 13.5px; line-height: 2; }
.tips b { color: var(--vermilion); }
</style>
