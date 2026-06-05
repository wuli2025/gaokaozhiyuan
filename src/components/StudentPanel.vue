<script setup lang="ts">
import { computed, ref } from "vue";
import { useProfileStore, ASPIRATION_DIMS } from "../stores/profile";
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

const avatar = computed(() => "考");

// 行内编辑
const editingKey = ref<string | null>(null);
const draft = ref("");
function startEdit(key: string) {
  editingKey.value = key;
  draft.value = profile.aspiration[key] ?? "";
}
function saveEdit(key: string) {
  profile.setAspiration(key, draft.value.trim());
  editingKey.value = null;
}

function goFill() { app.setView("match"); }
function goIdol() { app.setView("idol"); }
function reset() { if (confirm("确定清空当前档案（含志向画像）？")) profile.reset(); }

// 系统提示：基于已填维度做轻量启发式判断（规则，非 LLM 算命）
const sysHints = computed(() => {
  const a = profile.aspiration;
  const hits: { type: "green" | "gold" | "red"; text: string }[] = [];
  const ability = (a.subjectAbility ?? "");
  const interest = (a.interest ?? "");
  if (/物理|数学/.test(ability)) hits.push({ type: "green", text: "数理底子好 → 适配 工科/电子信息/电气" });
  if (/英语弱|英语差|英语偏弱/.test(ability)) hits.push({ type: "red", text: "英语偏弱 → 慎选 强英语类/翻译" });
  if (/考公|编制|稳定/.test(a.family ?? "")) hits.push({ type: "gold", text: "家庭倾向编制 → 关注 师范/法学(考公)/农学" });
  if (interest && interest.length > 4) hits.push({ type: "green", text: "已填兴趣证据 → 真适配判断更准" });
  else hits.push({ type: "gold", text: "兴趣证据偏少 → 建议补“做过/查过什么”" });
  return hits;
});
</script>

<template>
  <div class="page">
    <header class="ph">
      <div class="eyebrow">★ 我的志向画像 · LLMWiki</div>
      <h1>我的档案</h1>
      <p>位次是硬条件，志向画像是软条件。这一页越丰富，冲稳保和偶像对话就越懂你。</p>
    </header>

    <!-- 成绩卡 -->
    <div v-if="profile.ready" class="card profile">
      <div class="pf-avatar">{{ avatar }}</div>
      <div class="pf-body">
        <div class="pf-row">
          <span class="pf-score">{{ profile.score ?? "—" }}<i>分</i></span>
          <span class="pf-rank">位次 ≈ {{ profile.rank?.toLocaleString() }}</span>
        </div>
        <div class="pf-tags"><span v-for="t in tags" :key="t" class="pf-tag">{{ t }}</span></div>
      </div>
      <div class="pf-actions">
        <button class="btn primary" @click="goFill">去智能填报 →</button>
        <button class="btn ghost" @click="reset">清空档案</button>
      </div>
    </div>

    <div v-else class="card empty">
      <div class="em-icon">📝</div>
      <h3>还没有建立档案</h3>
      <p>先填省份、分数与选科，换算位次后再来完善志向画像。</p>
      <button class="btn primary big" @click="goFill">立即建立档案</button>
    </div>

    <!-- 志向画像（有档案才显示） -->
    <template v-if="profile.ready">
      <div class="sec-head">
        <h3>志向画像</h3>
        <span class="done">已填 {{ profile.aspirationFilled }} / {{ ASPIRATION_DIMS.length }} 维 · 点任意行编辑</span>
      </div>
      <div class="wiki">
        <div v-for="d in ASPIRATION_DIMS" :key="d.key" class="wrow">
          <div class="wk">{{ d.label }}</div>
          <div class="wv" @click="startEdit(d.key)">
            <template v-if="editingKey === d.key">
              <textarea
                v-model="draft"
                rows="2"
                :placeholder="d.hint"
                @blur="saveEdit(d.key)"
                @keydown.enter.exact.prevent="saveEdit(d.key)"
                ref="ta"
                autofocus
              ></textarea>
            </template>
            <template v-else>
              <span v-if="profile.aspiration[d.key]" class="filled">{{ profile.aspiration[d.key] }}</span>
              <span v-else class="ph-hint">{{ d.hint }} ＋点击填写</span>
            </template>
          </div>
        </div>
      </div>

      <div class="syshint">
        <div class="sh-title">系统提示 <small>（规则引擎，非 AI 算命，可追溯）</small></div>
        <div class="sh-row">
          <span v-for="(h, i) in sysHints" :key="i" class="sh-tag" :class="h.type">{{ h.text }}</span>
        </div>
        <button class="idol-cta" @click="goIdol">想不清方向？去和偶像聊聊 →</button>
      </div>
    </template>

    <div class="card tips">
      <h3>位次法 · 为什么看位次而不是分数</h3>
      <ul>
        <li>每年试题难度不同，<b>分数会浮动</b>，但全省排名（位次）相对稳定。</li>
        <li>我们用本省真实「分数↔位次」对应关系，把分数换算成全省位次。</li>
        <li>再用位次对比各院校专业去年的<b>最低录取位次</b>，给出冲 / 稳 / 保。</li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.page { height: 100vh; overflow-y: auto; max-width: 760px; margin: 0 auto; padding: 30px 28px 60px; }
.eyebrow { font-family: var(--mono); font-size: 11px; color: var(--gold-deep); font-weight: 700; letter-spacing: .2em; }
.ph h1 { font-family: var(--serif); font-size: 27px; margin: 6px 0 0; color: var(--ink); }
.ph p { color: var(--text-2); font-size: 13.5px; margin: 8px 0 22px; }
.card { background: var(--panel); border: 1px solid var(--border); border-radius: 18px; padding: 22px 26px; box-shadow: var(--shadow); margin-bottom: 18px; }
.profile { display: flex; gap: 18px; align-items: center; }
.pf-avatar { width: 56px; height: 56px; border-radius: 16px; background: linear-gradient(135deg, #c8372d, #e0644f); color: #fff; display: flex; align-items: center; justify-content: center; font-size: 24px; font-weight: 800; font-family: var(--serif); flex: none; }
.pf-body { flex: 1; }
.pf-row { display: flex; align-items: baseline; gap: 16px; }
.pf-score { font-size: 32px; font-weight: 800; color: var(--ink); font-family: var(--serif); }
.pf-score i { font-size: 14px; color: var(--muted); font-style: normal; margin-left: 2px; }
.pf-rank { background: var(--blue-soft); color: var(--blue-deep); padding: 3px 12px; border-radius: 999px; font-size: 13px; font-weight: 700; }
.pf-tags { display: flex; gap: 7px; margin-top: 10px; flex-wrap: wrap; }
.pf-tag { background: var(--bg-soft); color: var(--text-2); padding: 3px 11px; border-radius: 7px; font-size: 12px; }
.pf-actions { display: flex; flex-direction: column; gap: 8px; }
.btn { border: none; border-radius: 10px; padding: 10px 18px; font-size: 13px; font-weight: 700; cursor: pointer; }
.btn.primary { background: var(--grad); color: #fff; }
.btn.primary.big { padding: 13px 28px; font-size: 14.5px; margin-top: 16px; }
.btn.ghost { background: var(--panel); border: 1px solid var(--border); color: var(--muted); }
.empty { text-align: center; padding: 46px 26px; }
.em-icon { font-size: 40px; }
.empty h3 { margin: 14px 0 6px; color: var(--ink); }
.empty p { color: var(--muted); font-size: 13px; margin: 0; }

.sec-head { display: flex; align-items: baseline; justify-content: space-between; margin: 6px 2px 12px; }
.sec-head h3 { font-family: var(--serif); font-size: 18px; color: var(--ink); margin: 0; }
.done { font-size: 12px; color: var(--muted); }
.wiki { background: var(--panel); border: 1px solid var(--border); border-radius: 14px; overflow: hidden; box-shadow: var(--shadow-sm); margin-bottom: 16px; }
.wrow { display: grid; grid-template-columns: 130px 1fr; border-bottom: 1px solid var(--border-soft); }
.wrow:last-child { border-bottom: none; }
.wk { background: var(--gold-soft); color: var(--gold-deep); font-weight: 700; font-size: 13px; padding: 12px 14px; display: flex; align-items: center; }
.wv { padding: 10px 14px; font-size: 13px; color: var(--text-2); cursor: text; min-height: 44px; display: flex; align-items: center; }
.wv .filled { white-space: pre-wrap; }
.wv .ph-hint { color: var(--dim); }
.wv textarea { width: 100%; border: 1px solid var(--primary); border-radius: 8px; padding: 7px 9px; font-size: 13px; resize: vertical; outline: none; font-family: var(--sans); line-height: 1.6; }

.syshint { background: linear-gradient(135deg, #fffaf4, #fdf1e7); border: 1px solid var(--border); border-radius: 14px; padding: 16px 18px; margin-bottom: 18px; }
.sh-title { font-size: 13.5px; font-weight: 800; color: var(--ink); margin-bottom: 10px; }
.sh-title small { color: var(--muted); font-weight: 400; }
.sh-row { display: flex; flex-wrap: wrap; gap: 8px; }
.sh-tag { font-size: 12px; padding: 4px 11px; border-radius: 999px; border: 1px solid transparent; }
.sh-tag.green { background: var(--green-soft); color: var(--green); border-color: #c4e2cf; }
.sh-tag.gold { background: var(--gold-soft); color: var(--gold-deep); border-color: #ecd5a3; }
.sh-tag.red { background: #fde6e3; color: var(--red-deep); border-color: #f2c1ba; }
.idol-cta { margin-top: 14px; background: none; border: none; color: var(--blue); font-size: 13px; font-weight: 700; padding: 0; cursor: pointer; }

.tips h3 { font-size: 15px; color: var(--ink); margin: 0 0 12px; }
.tips ul { margin: 0; padding-left: 18px; color: var(--text-2); font-size: 13.5px; line-height: 2; }
.tips b { color: var(--blue); }
</style>
