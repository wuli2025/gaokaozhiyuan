<script setup lang="ts">
import { computed, ref, onMounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useProfileStore, ASPIRATION_DIMS } from "../stores/profile";
import { useAppStore } from "../stores/app";
import { usePersonalWikiStore } from "../stores/personalWiki";
import { useFileDrop } from "../composables/useFileDrop";
import { isTauri } from "../tauri";

const profile = useProfileStore();
const app = useAppStore();
const pwiki = usePersonalWikiStore();

// 进入档案页：拉一次个人资料清单 + 已有报告
onMounted(() => {
  pwiki.refreshMaterials();
  if (!pwiki.reportLoaded) pwiki.loadReport();
});

// 拖拽上传（仅当前视图是档案页时响应）
const { isOver: dropOver } = useFileDrop({
  active: () => app.view === "student",
  onDrop: (paths) => pwiki.upload(paths),
});

// 按钮选择文件上传
async function pickFiles() {
  if (!isTauri) return;
  const picked = await open({
    multiple: true,
    title: "选择要上传的个人资料（成绩单 / 体检表 / 个人陈述 等）",
  });
  if (!picked) return;
  const paths = Array.isArray(picked) ? picked : [picked];
  await pwiki.upload(paths);
}

function fmtSize(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

async function genReport() {
  await pwiki.generateReport();
}

const riskClass = (lvl: string) =>
  lvl === "high" ? "red" : lvl === "mid" ? "gold" : "green";
const levelClass = (lvl: string) =>
  lvl === "强" ? "green" : lvl === "弱" ? "red" : "gold";

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

    <!-- 个人 wiki：上传个人资料 + 一键整理报告 -->
    <div class="sec-head">
      <h3>考试个人 wiki</h3>
      <span class="done">成绩单 / 体检表 / 个人陈述等 · 对话时自动作为你的专属上下文</span>
    </div>

    <div
      class="card drop"
      :class="{ over: dropOver }"
      @click="pickFiles"
    >
      <div class="drop-icon">📎</div>
      <div class="drop-main">
        <b>拖拽文件到这里，或点击选择上传</b>
        <small>支持 PDF / Word / Excel / PPT / 图片 / 纯文本，自动转入你的个人资料库</small>
      </div>
      <button class="btn primary" @click.stop="pickFiles" :disabled="pwiki.uploading">
        {{ pwiki.uploading ? "上传中…" : "选择文件" }}
      </button>
    </div>

    <div v-if="pwiki.materials.length" class="card files">
      <div v-for="f in pwiki.materials" :key="f.relPath" class="frow">
        <span class="fname">📄 {{ f.name }}</span>
        <span class="fsize">{{ fmtSize(f.size) }}</span>
        <button class="fdel" title="删除" @click="pwiki.remove(f.relPath)">✕</button>
      </div>
    </div>
    <div v-else class="hint-empty">还没有上传个人资料 —— 上传后整理报告会更准。</div>

    <div class="report-cta">
      <button class="btn gen" @click="genReport" :disabled="pwiki.generating">
        {{ pwiki.generating ? "正在整理…" : "✦ 一键整理 · 生成结构化考试报告" }}
      </button>
      <span v-if="pwiki.generating" class="gen-progress">{{ pwiki.progress }}</span>
      <span v-if="pwiki.error" class="gen-error">{{ pwiki.error }}</span>
    </div>

    <!-- 结构化报告卡 -->
    <div v-if="pwiki.report" class="card report">
      <div class="rp-head">
        <div class="rp-badge">考试报告</div>
        <h3 v-if="pwiki.report.headline">{{ pwiki.report.headline }}</h3>
      </div>

      <div v-if="pwiki.report.score_profile" class="rp-score">
        <span v-if="pwiki.report.score_profile.score != null" class="rp-num">
          {{ pwiki.report.score_profile.score }}<i>分</i>
        </span>
        <span v-if="pwiki.report.score_profile.rank != null" class="rp-rank">
          位次 ≈ {{ pwiki.report.score_profile.rank?.toLocaleString() }}
        </span>
        <span v-if="pwiki.report.score_profile.province" class="rp-tag">{{ pwiki.report.score_profile.province }}</span>
        <span v-if="pwiki.report.score_profile.track" class="rp-tag">{{ pwiki.report.score_profile.track }}</span>
        <span v-if="pwiki.report.score_profile.subjects" class="rp-tag">{{ pwiki.report.score_profile.subjects }}</span>
      </div>

      <div v-if="pwiki.report.subjects?.length" class="rp-block">
        <div class="rp-bt">学科强弱</div>
        <div class="rp-chips">
          <span v-for="s in pwiki.report.subjects" :key="s.name" class="rp-chip" :class="levelClass(s.level)">
            {{ s.name }} · {{ s.level }}<em v-if="s.note"> · {{ s.note }}</em>
          </span>
        </div>
      </div>

      <div v-if="pwiki.report.strengths?.length" class="rp-block">
        <div class="rp-bt">优势</div>
        <ul class="rp-list"><li v-for="(s, i) in pwiki.report.strengths" :key="i">{{ s }}</li></ul>
      </div>

      <div v-if="pwiki.report.risks?.length" class="rp-block">
        <div class="rp-bt">风险与注意</div>
        <div class="rp-chips">
          <span v-for="(r, i) in pwiki.report.risks" :key="i" class="rp-chip" :class="riskClass(r.level)">{{ r.text }}</span>
        </div>
      </div>

      <div v-if="pwiki.report.directions?.length" class="rp-block">
        <div class="rp-bt">建议关注方向</div>
        <div class="rp-chips">
          <span v-for="(d, i) in pwiki.report.directions" :key="i" class="rp-chip blue">{{ d }}</span>
        </div>
      </div>

      <div v-if="pwiki.report.gaps?.length" class="rp-block gaps">
        <div class="rp-bt">⚠ 资料不足 / 待补充</div>
        <ul class="rp-list"><li v-for="(g, i) in pwiki.report.gaps" :key="i">{{ g }}</li></ul>
      </div>

      <div v-if="pwiki.report.sources?.length" class="rp-sources">
        来源：<span v-for="(s, i) in pwiki.report.sources" :key="i">{{ s }}<template v-if="i < pwiki.report.sources.length - 1"> · </template></span>
      </div>
    </div>

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

/* 个人 wiki 上传区 */
.drop { display: flex; align-items: center; gap: 16px; cursor: pointer; border-style: dashed; transition: border-color .15s, background .15s; }
.drop.over { border-color: var(--primary); background: var(--gold-soft); }
.drop-icon { font-size: 26px; flex: none; }
.drop-main { flex: 1; }
.drop-main b { display: block; color: var(--ink); font-size: 14px; }
.drop-main small { color: var(--muted); font-size: 12px; }

.files { padding: 8px 14px; margin-bottom: 10px; }
.frow { display: flex; align-items: center; gap: 10px; padding: 8px 4px; border-bottom: 1px solid var(--border-soft); }
.frow:last-child { border-bottom: none; }
.fname { flex: 1; font-size: 13px; color: var(--text-1); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.fsize { font-size: 12px; color: var(--muted); flex: none; }
.fdel { border: none; background: none; color: var(--dim); cursor: pointer; font-size: 13px; padding: 2px 6px; border-radius: 6px; }
.fdel:hover { background: #fde6e3; color: var(--red-deep); }
.hint-empty { color: var(--muted); font-size: 12.5px; margin: 0 2px 14px; }

.report-cta { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; margin-bottom: 18px; }
.btn.gen { background: var(--grad); color: #fff; padding: 12px 22px; font-size: 14px; border: none; border-radius: 12px; cursor: pointer; font-weight: 700; }
.btn.gen:disabled { opacity: .65; cursor: default; }
.gen-progress { font-size: 12.5px; color: var(--blue-deep); }
.gen-error { font-size: 12.5px; color: var(--red-deep); }

/* 报告卡 */
.report { padding: 22px 26px; }
.rp-head { display: flex; align-items: center; gap: 12px; margin-bottom: 14px; }
.rp-badge { background: var(--grad); color: #fff; font-size: 11px; font-weight: 800; padding: 4px 12px; border-radius: 999px; flex: none; letter-spacing: .05em; }
.rp-head h3 { font-family: var(--serif); font-size: 18px; color: var(--ink); margin: 0; }
.rp-score { display: flex; align-items: baseline; gap: 12px; flex-wrap: wrap; padding-bottom: 16px; margin-bottom: 14px; border-bottom: 1px solid var(--border-soft); }
.rp-num { font-size: 30px; font-weight: 800; color: var(--ink); font-family: var(--serif); }
.rp-num i { font-size: 13px; color: var(--muted); font-style: normal; margin-left: 2px; }
.rp-rank { background: var(--blue-soft); color: var(--blue-deep); padding: 3px 12px; border-radius: 999px; font-size: 13px; font-weight: 700; }
.rp-tag { background: var(--bg-soft); color: var(--text-2); padding: 3px 11px; border-radius: 7px; font-size: 12px; }
.rp-block { margin-bottom: 14px; }
.rp-bt { font-size: 13px; font-weight: 800; color: var(--ink); margin-bottom: 8px; }
.rp-chips { display: flex; flex-wrap: wrap; gap: 8px; }
.rp-chip { font-size: 12px; padding: 4px 11px; border-radius: 999px; border: 1px solid transparent; }
.rp-chip em { font-style: normal; opacity: .8; }
.rp-chip.green { background: var(--green-soft); color: var(--green); border-color: #c4e2cf; }
.rp-chip.gold { background: var(--gold-soft); color: var(--gold-deep); border-color: #ecd5a3; }
.rp-chip.red { background: #fde6e3; color: var(--red-deep); border-color: #f2c1ba; }
.rp-chip.blue { background: var(--blue-soft); color: var(--blue-deep); border-color: #bcd5f5; }
.rp-list { margin: 0; padding-left: 18px; color: var(--text-2); font-size: 13px; line-height: 1.9; }
.rp-block.gaps .rp-bt { color: var(--gold-deep); }
.rp-sources { font-size: 11.5px; color: var(--muted); margin-top: 14px; padding-top: 12px; border-top: 1px solid var(--border-soft); font-family: var(--mono); }
</style>
