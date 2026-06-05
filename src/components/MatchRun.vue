<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { gk, type GkMatchResult, type GkProvince } from "../tauri";
import { useProfileStore } from "../stores/profile";
import SchoolDetailModal from "./SchoolDetailModal.vue";

const profile = useProfileStore();

// ─────────────── 数据状态 ───────────────
const provinces = ref<GkProvince[]>([]);
const result = ref<GkMatchResult | null>(null);
const loading = ref(false);
const errMsg = ref("");

// 输入面板（首次或点"修改成绩"时显示）
const editing = ref(false);
const draftScore = ref<number | null>(profile.score);

// 筛选
const fRegions = ref<string[]>([]);
const fLevels = ref<string[]>([]);
const fTypes = ref<string[]>([]);
const fTier = ref<"" | "冲" | "稳" | "保">("");
const keyword = ref("");
const sort = ref<"prob" | "rank">("prob");
const page = ref(0);
const pageSize = 40;

// 弹层
const showRegionPicker = ref(false);
const detailSchool = ref<string | null>(null);

const RESELECT = ["化学", "生物", "地理", "思想政治"];

onMounted(async () => {
  try {
    provinces.value = await gk.provinces();
  } catch {
    /* ignore */
  }
  if (profile.ready) {
    runMatch();
  } else {
    editing.value = true;
  }
});

// ─────────────── 匹配 ───────────────
async function runMatch(resetPage = true) {
  if (!profile.ready) {
    editing.value = true;
    return;
  }
  if (resetPage) page.value = 0;
  loading.value = true;
  errMsg.value = "";
  try {
    result.value = await gk.match({
      province: profile.province,
      track: profile.track,
      subjects: profile.reselect,
      rank: profile.rank!,
      regions: fRegions.value,
      levels: fLevels.value,
      types: fTypes.value,
      tiers: fTier.value ? [fTier.value] : [],
      keyword: keyword.value.trim(),
      sort: sort.value,
      page: page.value,
      page_size: pageSize,
    });
  } catch (e: any) {
    errMsg.value = typeof e === "string" ? e : e?.message ?? "匹配失败";
    result.value = null;
  } finally {
    loading.value = false;
  }
}

// 筛选变化即重查（轻量防抖）
let t: number | undefined;
watch([fRegions, fLevels, fTypes, fTier, sort], () => runMatch(true), { deep: true });
watch(keyword, () => {
  clearTimeout(t);
  t = window.setTimeout(() => runMatch(true), 350);
});

// ─────────────── 输入面板逻辑 ───────────────
async function confirmProfile() {
  if (!profile.province) {
    profile.error = "请选择省份";
    return;
  }
  if (draftScore.value == null || draftScore.value <= 0) {
    profile.error = "请填写预估总分";
    return;
  }
  profile.score = draftScore.value;
  const ok = await profile.computeRank();
  if (ok) {
    editing.value = false;
    await runMatch();
  }
}

// ─────────────── 看板派生 ───────────────
const stats = computed(() => result.value?.stats);
const donut = computed(() => {
  const s = stats.value;
  if (!s) return [];
  const total = Math.max(1, s.charge + s.steady + s.safe);
  const segs = [
    { key: "保", label: "可保底", n: s.safe, color: "#1f9d6b" },
    { key: "稳", label: "较稳妥", n: s.steady, color: "#e8973a" },
    { key: "冲", label: "可冲刺", n: s.charge, color: "#e0483b" },
  ];
  let offset = 0;
  const C = 2 * Math.PI * 80;
  return segs.map((seg) => {
    const frac = seg.n / total;
    const dash = frac * C;
    const o = offset;
    offset += dash;
    return { ...seg, dash, gap: C - dash, dashoffset: -o, pct: Math.round(frac * 100) };
  });
});

function tierColor(t: string) {
  return t === "冲" ? "#e0483b" : t === "稳" ? "#e8973a" : "#1f9d6b";
}
function probText(p: number) {
  return Math.round(p * 100) + "%";
}

// 地区预设
const REGION_PRESETS: Record<string, string[]> = {
  江浙沪: ["江苏", "浙江", "上海"],
  京津冀: ["北京", "天津", "河北"],
  珠三角: ["广东"],
  成渝: ["四川", "重庆"],
  华中: ["湖北", "湖南", "河南"],
  东北: ["辽宁", "吉林", "黑龙江"],
};
function applyPreset(name: string) {
  const set = new Set(fRegions.value);
  const all = REGION_PRESETS[name];
  const allIn = all.every((p) => set.has(p));
  if (allIn) all.forEach((p) => set.delete(p));
  else all.forEach((p) => set.add(p));
  fRegions.value = [...set];
}
function toggleArr(arr: { value: string[] }, v: string) {
  const i = arr.value.indexOf(v);
  if (i >= 0) arr.value.splice(i, 1);
  else arr.value.push(v);
}
function clearFilters() {
  fRegions.value = [];
  fLevels.value = [];
  fTypes.value = [];
  fTier.value = "";
  keyword.value = "";
}
const activeFilterCount = computed(
  () => fRegions.value.length + fLevels.value.length + fTypes.value.length + (fTier.value ? 1 : 0)
);

const levelOrder = ["985", "211", "双一流", "普通本科"];
const sortedLevels = computed(() => {
  const f = result.value?.facets.level ?? [];
  return [...f].sort((a, b) => levelOrder.indexOf(a.key) - levelOrder.indexOf(b.key));
});

function nextPage() {
  if (!result.value) return;
  if ((page.value + 1) * pageSize >= (result.value.stats.total || 0)) return;
  page.value++;
  runMatch(false);
}
function prevPage() {
  if (page.value <= 0) return;
  page.value--;
  runMatch(false);
}
const pageInfo = computed(() => {
  if (!result.value) return "";
  const tot = result.value.stats.total;
  const from = page.value * pageSize + 1;
  const to = Math.min(tot, (page.value + 1) * pageSize);
  return tot === 0 ? "暂无匹配结果" : `${from}–${to} / 共 ${tot} 个`;
});
</script>

<template>
  <div class="match-root">
    <!-- ══════════ 输入面板 ══════════ -->
    <div v-if="editing" class="input-stage">
      <div class="input-card">
        <div class="ic-brand"><span class="ai">智能</span>填报</div>
        <p class="ic-sub">输入分数与选科，按真实位次为你匹配可报院校专业</p>

        <label class="ic-label">考生省份</label>
        <div class="prov-grid">
          <button
            v-for="p in provinces"
            :key="p.province"
            class="prov-chip"
            :class="{ on: profile.province === p.province }"
            @click="profile.province = p.province"
          >{{ p.province }}</button>
        </div>

        <label class="ic-label">首选科目</label>
        <div class="seg">
          <button :class="{ on: profile.track === '物理' }" @click="profile.track = '物理'">物理</button>
          <button :class="{ on: profile.track === '历史' }" @click="profile.track = '历史'">历史</button>
        </div>

        <label class="ic-label">再选科目 <span class="dim">（最多 2 科，可不选）</span></label>
        <div class="seg multi">
          <button
            v-for="s in RESELECT"
            :key="s"
            :class="{ on: profile.reselect.includes(s) }"
            @click="profile.toggleReselect(s)"
          >{{ s }}</button>
        </div>

        <label class="ic-label">预估总分</label>
        <div class="score-input">
          <input type="number" v-model.number="draftScore" placeholder="如 530" @keyup.enter="confirmProfile" />
          <span class="unit">分</span>
        </div>

        <p v-if="profile.error" class="err">{{ profile.error }}</p>

        <button class="cta" :disabled="profile.computing" @click="confirmProfile">
          {{ profile.computing ? "换算位次中…" : "开始智能匹配" }}
        </button>
        <p class="ic-foot">位次由本省真实录取数据插值换算，越靠近真实一分一段越准</p>
      </div>
    </div>

    <!-- ══════════ 结果工作台 ══════════ -->
    <div v-else class="board" :class="{ busy: loading }">
      <header class="hero pk-hero">
        <div class="hero-eyebrow">★ POLARIS · 智能填报 · 位次法 · 冲 / 稳 / 保</div>
        <div class="hero-row">
          <h1 class="hero-title">智能<span class="lite">填报</span></h1>
          <button class="edit-btn" @click="editing = true; draftScore = profile.score">修改成绩</button>
        </div>
        <div class="hero-meta">
          <span class="pk-chip">{{ profile.province }}</span>
          <span class="pk-chip">{{ profile.track }}{{ profile.reselect.length ? " · " + profile.reselect.join(" ") : "" }}</span>
          <span class="pk-chip" v-if="profile.score"><b>{{ profile.score }}</b> 分</span>
          <span class="pk-chip rank">位次 ≈ <b>{{ profile.rank?.toLocaleString() }}</b></span>
        </div>
      </header>

      <section class="dashboard" v-if="stats">
        <div class="donut-wrap">
          <svg viewBox="0 0 200 200" class="donut">
            <circle cx="100" cy="100" r="80" fill="none" stroke="#f0eee9" stroke-width="22" />
            <circle
              v-for="seg in donut"
              :key="seg.key"
              cx="100" cy="100" r="80" fill="none"
              :stroke="seg.color" stroke-width="22"
              :stroke-dasharray="`${seg.dash} ${seg.gap}`"
              :stroke-dashoffset="seg.dashoffset"
              transform="rotate(-90 100 100)"
            />
            <text x="100" y="94" text-anchor="middle" class="donut-num">{{ stats.total.toLocaleString() }}</text>
            <text x="100" y="118" text-anchor="middle" class="donut-cap">适合我的院校专业</text>
          </svg>
          <div class="donut-legend">
            <button
              v-for="seg in donut"
              :key="seg.key"
              class="leg"
              :class="{ on: fTier === seg.key }"
              @click="fTier = fTier === seg.key ? '' : (seg.key as any)"
            >
              <span class="leg-dot" :style="{ background: seg.color }"></span>
              <span class="leg-n" :style="{ color: seg.color }">{{ seg.n }}</span>
              <span class="leg-l">{{ seg.label }}</span>
            </button>
          </div>
        </div>

        <div class="stat-grid">
          <div class="sg-cell"><div class="sg-n">{{ stats.c985 }}</div><div class="sg-l">985 院校</div></div>
          <div class="sg-cell"><div class="sg-n">{{ stats.c211 }}</div><div class="sg-l">211 院校</div></div>
          <div class="sg-cell"><div class="sg-n">{{ stats.double_first }}</div><div class="sg-l">双一流</div></div>
          <div class="sg-cell"><div class="sg-n">{{ stats.charge }}</div><div class="sg-l">可冲刺</div></div>
          <div class="sg-cell"><div class="sg-n">{{ stats.steady }}</div><div class="sg-l">较稳妥</div></div>
          <div class="sg-cell"><div class="sg-n">{{ stats.safe }}</div><div class="sg-l">可保底</div></div>
        </div>
      </section>

      <section class="filterbar">
        <div class="fb-row">
          <button class="fb-region" :class="{ active: fRegions.length }" @click="showRegionPicker = !showRegionPicker">
            <span>📍 地区</span>
            <span v-if="fRegions.length" class="badge">{{ fRegions.length }}</span>
            <span class="caret">▾</span>
          </button>

          <div class="fb-tabs">
            <button :class="{ on: !fTier }" @click="fTier = ''">全部</button>
            <button class="t-chong" :class="{ on: fTier === '冲' }" @click="fTier = fTier === '冲' ? '' : '冲'">冲</button>
            <button class="t-wen" :class="{ on: fTier === '稳' }" @click="fTier = fTier === '稳' ? '' : '稳'">稳</button>
            <button class="t-bao" :class="{ on: fTier === '保' }" @click="fTier = fTier === '保' ? '' : '保'">保</button>
          </div>

          <div class="fb-search"><input v-model="keyword" placeholder="搜院校 / 专业" /></div>
          <select class="fb-sort" v-model="sort">
            <option value="prob">按录取概率</option>
            <option value="rank">按最低位次</option>
          </select>
        </div>

        <div class="fb-facets" v-if="result">
          <div class="facet-line">
            <span class="facet-tag">层次</span>
            <button
              v-for="l in sortedLevels"
              :key="l.key"
              class="facet-chip"
              :class="{ on: fLevels.includes(l.key) }"
              @click="toggleArr(fLevels as any, l.key)"
            >{{ l.key }} <i>{{ l.count }}</i></button>
          </div>
          <div class="facet-line">
            <span class="facet-tag">类型</span>
            <button
              v-for="ty in result.facets.type.slice(0, 10)"
              :key="ty.key"
              class="facet-chip"
              :class="{ on: fTypes.includes(ty.key) }"
              @click="toggleArr(fTypes as any, ty.key)"
            >{{ ty.key }} <i>{{ ty.count }}</i></button>
          </div>
          <button v-if="activeFilterCount" class="clear-all" @click="clearFilters">清空筛选 ✕</button>
        </div>

        <div v-if="showRegionPicker" class="region-pop" @click.self="showRegionPicker = false">
          <div class="region-panel">
            <div class="rp-head">
              <b>选择院校所在地</b>
              <button @click="showRegionPicker = false">完成</button>
            </div>
            <div class="rp-sec">热门区域</div>
            <div class="rp-grid">
              <button
                v-for="(arr, name) in REGION_PRESETS"
                :key="name"
                class="rp-chip preset"
                :class="{ on: arr.every((p) => fRegions.includes(p)) }"
                @click="applyPreset(name as string)"
              >{{ name }}</button>
            </div>
            <div class="rp-sec">全部地区</div>
            <div class="rp-grid">
              <button
                v-for="r in result?.facets.region"
                :key="r.key"
                class="rp-chip"
                :class="{ on: fRegions.includes(r.key) }"
                @click="toggleArr(fRegions as any, r.key)"
              >{{ r.key }} <i>{{ r.count }}</i></button>
            </div>
            <div class="rp-foot"><button class="rp-reset" @click="fRegions = []">重置</button></div>
          </div>
        </div>
      </section>

      <section class="results">
        <div class="res-head">
          <span>{{ pageInfo }}</span>
          <span v-if="loading" class="loading-dot">匹配中…</span>
        </div>

        <p v-if="errMsg" class="err">{{ errMsg }}</p>

        <div v-if="result && result.rows.length" class="card-list">
          <article
            v-for="(c, i) in result.rows"
            :key="i"
            class="cand"
            @click="detailSchool = c.school"
          >
            <div class="cand-tier" :style="{ background: tierColor(c.tier) }">{{ c.tier }}</div>
            <div class="cand-main">
              <div class="cand-top">
                <span class="cand-school">{{ c.school }}</span>
                <span v-if="c.is985" class="badge985">985</span>
                <span v-else-if="c.is211" class="badge211">211</span>
                <span v-if="c.double_first" class="badgedf">双一流</span>
                <span class="cand-region">{{ c.region }} · {{ c.school_type }}</span>
              </div>
              <div class="cand-major">{{ c.major }}</div>
              <div class="cand-sg">{{ c.subject_group }}</div>
            </div>
            <div class="cand-data">
              <div class="cd-prob" :style="{ color: tierColor(c.tier) }">
                <span class="cd-prob-n">{{ probText(c.prob) }}</span>
                <span class="cd-prob-l">录取概率</span>
              </div>
              <div class="cd-rank">
                <div><b>{{ c.min_rank?.toLocaleString() }}</b> 位次</div>
                <div v-if="c.min_score"><b>{{ c.min_score }}</b> 分 · 2024</div>
              </div>
            </div>
          </article>
        </div>

        <div v-else-if="!loading" class="empty">没有符合条件的院校专业，试试放宽筛选或调整分数。</div>

        <div class="pager" v-if="result && result.stats.total > pageSize">
          <button :disabled="page === 0" @click="prevPage">上一页</button>
          <span>第 {{ page + 1 }} 页</span>
          <button :disabled="(page + 1) * pageSize >= result.stats.total" @click="nextPage">下一页</button>
        </div>
      </section>
    </div>

    <SchoolDetailModal
      v-if="detailSchool"
      :name="detailSchool"
      :province="profile.province"
      :track="profile.track"
      :user-rank="profile.rank ?? 0"
      @close="detailSchool = null"
    />
  </div>
</template>

<style scoped>
.match-root { height: 100vh; overflow-y: auto; background: transparent; }
.ai { color: var(--primary); }

/* ─── 输入面板 ─── */
.input-stage {
  min-height: 100vh; display: flex; align-items: center; justify-content: center; padding: 40px 20px;
}
.input-card { position: relative; width: 100%; max-width: 480px; background: var(--panel); border: 1px solid var(--border-soft); border-radius: 22px; padding: 38px 34px 34px; box-shadow: var(--shadow-lg); overflow: hidden; }
.input-card::before { content: ""; position: absolute; top: 0; left: 0; right: 0; height: 4px; background: var(--grad); }
.ic-brand { font-family: var(--serif); font-size: 30px; font-weight: 800; letter-spacing: 1px; color: var(--ink); }
.ic-sub { color: var(--muted); font-size: 13px; margin: 8px 0 22px; }
.ic-label { display: block; font-size: 12.5px; font-weight: 700; color: var(--text-2); margin: 18px 0 9px; }
.ic-label .dim { color: var(--dim); font-weight: 400; }
.prov-grid { display: grid; grid-template-columns: repeat(5, 1fr); gap: 7px; }
.prov-chip { padding: 8px 4px; border: 1px solid var(--border); background: #fff; border-radius: 9px; font-size: 12.5px; color: var(--text-2); transition: .15s; }
.prov-chip:hover { border-color: var(--primary); }
.prov-chip.on { background: var(--primary); color: #fff; border-color: var(--primary); font-weight: 700; }
.seg { display: flex; gap: 9px; }
.seg.multi { flex-wrap: wrap; }
.seg button { flex: 1; min-width: 70px; padding: 11px 0; border: 1px solid var(--border); background: #fff; border-radius: 10px; font-size: 14px; color: var(--text-2); font-weight: 600; transition: .15s; }
.seg button.on { background: var(--vermilion-soft); color: var(--vermilion); border-color: #f0bcb4; }
.score-input { display: flex; align-items: center; border: 1.5px solid var(--border); border-radius: 12px; padding: 4px 16px; transition: .15s; }
.score-input:focus-within { border-color: var(--primary); box-shadow: 0 0 0 3px var(--primary-soft); }
.score-input input { flex: 1; border: none; outline: none; font-size: 26px; font-weight: 800; color: var(--ink); padding: 8px 0; background: transparent; }
.score-input .unit { color: var(--muted); font-size: 15px; }
.cta { width: 100%; margin-top: 22px; padding: 14px; border: none; border-radius: 13px; background: var(--grad); color: #fff; font-size: 15.5px; font-weight: 800; letter-spacing: 2px; box-shadow: 0 10px 24px -8px var(--glow); transition: .18s; }
.cta:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 14px 30px -8px var(--glow); filter: brightness(1.04); }
.cta:disabled { opacity: .6; }
.ic-foot { text-align: center; color: var(--dim); font-size: 11.5px; margin-top: 14px; }
.err { color: var(--vermilion); font-size: 12.5px; margin: 10px 0 0; }

/* ─── 工作台 ─── */
.board { max-width: 1080px; margin: 0 auto; padding: 22px 24px 60px; }
.board.busy { opacity: .82; }

/* ─── 鎏金 hero band（首屏门面，.pk-hero 提供深色暖调+流光+网格） ─── */
.hero { margin-bottom: 18px; }
.hero-eyebrow { font-family: var(--mono); font-size: 10.5px; font-weight: 700; letter-spacing: .28em; text-transform: uppercase; color: var(--amber); margin-bottom: 12px; }
.hero-row { display: flex; align-items: center; justify-content: space-between; gap: 16px; }
.hero-title { margin: 0; font-family: var(--serif); font-size: 30px; font-weight: 800; letter-spacing: 1px; line-height: 1.2;
  background: linear-gradient(100deg, #fff 0%, #ffd9a8 42%, #ff9b6e 78%, #ffcf8e 100%);
  -webkit-background-clip: text; background-clip: text; color: transparent;
  filter: drop-shadow(0 2px 14px rgba(255, 120, 70, .28)); }
.hero-title .lite { -webkit-text-fill-color: var(--amber); color: var(--amber); }
.hero-meta { display: flex; align-items: center; gap: 9px; margin-top: 16px; flex-wrap: wrap; }
.hero-meta .pk-chip.rank { background: rgba(255, 177, 77, .16); border-color: rgba(255, 177, 77, .35); color: #ffe6cf; }
.edit-btn { flex: none; background: rgba(255, 255, 255, .08); border: 1px solid rgba(255, 214, 180, .22); border-radius: 999px; padding: 8px 18px; font-size: 12.5px; color: #f3dcc9; font-weight: 600; backdrop-filter: blur(8px); transition: .16s; }
.edit-btn:hover { background: rgba(255, 255, 255, .16); color: #fff; border-color: var(--amber); }

.dashboard { display: grid; grid-template-columns: 360px 1fr; gap: 22px; align-items: center; background: #fff; border: 1px solid var(--border-soft); border-radius: 20px; padding: 26px 28px; box-shadow: var(--shadow); margin-bottom: 18px; }
.donut-wrap { display: flex; align-items: center; gap: 18px; }
.donut { width: 168px; height: 168px; flex: none; }
.donut-num { font-size: 38px; font-weight: 800; fill: var(--ink); font-family: var(--serif); }
.donut-cap { font-size: 10.5px; fill: var(--muted); }
.donut-legend { display: flex; flex-direction: column; gap: 8px; }
.leg { display: flex; align-items: center; gap: 8px; background: transparent; border: 1px solid transparent; border-radius: 9px; padding: 5px 9px; transition: .15s; }
.leg:hover { background: var(--bg-soft); }
.leg.on { background: var(--bg-soft); border-color: var(--border); }
.leg-dot { width: 9px; height: 9px; border-radius: 3px; }
.leg-n { font-size: 18px; font-weight: 800; }
.leg-l { font-size: 12.5px; color: var(--text-2); }

.stat-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1px; background: var(--border-soft); border-radius: 14px; overflow: hidden; }
.sg-cell { background: var(--panel); padding: 16px 10px; text-align: center; transition: background .15s; }
.sg-cell:hover { background: var(--bg-soft); }
.sg-n { font-size: 25px; font-weight: 800; font-family: var(--serif); line-height: 1;
  background: var(--grad); -webkit-background-clip: text; background-clip: text; color: transparent; }
.sg-l { font-size: 11.5px; color: var(--muted); margin-top: 6px; }

/* ─── 筛选 ─── */
.filterbar { position: relative; margin-bottom: 16px; }
.fb-row { display: flex; gap: 10px; align-items: center; flex-wrap: wrap; }
.fb-region { display: flex; align-items: center; gap: 6px; background: #fff; border: 1px solid var(--border); border-radius: 11px; padding: 9px 14px; font-size: 13px; font-weight: 600; color: var(--text-2); }
.fb-region.active { border-color: var(--primary); color: var(--primary); }
.fb-region .badge { background: var(--vermilion); color: #fff; font-size: 10px; border-radius: 999px; padding: 0 6px; }
.fb-region .caret { color: var(--dim); font-size: 10px; }
.fb-tabs { display: flex; background: #fff; border: 1px solid var(--border); border-radius: 11px; overflow: hidden; }
.fb-tabs button { padding: 9px 18px; border: none; background: #fff; font-size: 13.5px; font-weight: 700; color: var(--text-2); border-right: 1px solid var(--border-soft); }
.fb-tabs button:last-child { border-right: none; }
.fb-tabs button.on { background: var(--ink); color: #fff; }
.fb-tabs .t-chong.on { background: #e0483b; }
.fb-tabs .t-wen.on { background: #e8973a; }
.fb-tabs .t-bao.on { background: #1f9d6b; }
.fb-search { flex: 1; min-width: 160px; }
.fb-search input { width: 100%; border: 1px solid var(--border); border-radius: 11px; padding: 9px 14px; font-size: 13px; outline: none; background: #fff; }
.fb-search input:focus { border-color: var(--primary); }
.fb-sort { border: 1px solid var(--border); border-radius: 11px; padding: 9px 12px; font-size: 13px; background: #fff; color: var(--text-2); }

.fb-facets { display: flex; flex-direction: column; gap: 7px; margin-top: 12px; }
.facet-line { display: flex; align-items: center; gap: 7px; flex-wrap: wrap; }
.facet-tag { font-size: 12px; color: var(--muted); font-weight: 700; width: 32px; flex: none; }
.facet-chip { background: #fff; border: 1px solid var(--border); border-radius: 999px; padding: 4px 11px; font-size: 12px; color: var(--text-2); transition: .12s; }
.facet-chip i { color: var(--dim); font-style: normal; font-size: 11px; }
.facet-chip.on { background: var(--primary); color: #fff; border-color: var(--primary); }
.facet-chip.on i { color: rgba(255,255,255,.7); }
.clear-all { align-self: flex-start; background: none; border: none; color: var(--vermilion); font-size: 12px; font-weight: 600; padding: 2px 0; }

.region-pop { position: fixed; inset: 0; z-index: 50; background: rgba(20,20,25,.28); display: flex; align-items: flex-start; justify-content: center; padding-top: 8vh; }
.region-panel { width: 540px; max-width: 92vw; max-height: 76vh; overflow-y: auto; background: #fff; border-radius: 18px; padding: 20px 22px; box-shadow: var(--shadow-lg); }
.rp-head { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
.rp-head b { font-size: 15px; }
.rp-head button { background: var(--primary); color: #fff; border: none; border-radius: 9px; padding: 6px 16px; font-weight: 600; font-size: 13px; }
.rp-sec { font-size: 12.5px; font-weight: 700; color: var(--muted); margin: 14px 0 8px; }
.rp-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
.rp-chip { padding: 9px 4px; border: 1px solid var(--border); background: #fff; border-radius: 9px; font-size: 12.5px; color: var(--text-2); }
.rp-chip i { color: var(--dim); font-style: normal; font-size: 10.5px; }
.rp-chip.on { background: var(--vermilion-soft); border-color: #f0bcb4; color: var(--vermilion); font-weight: 700; }
.rp-chip.preset.on { background: var(--primary); color: #fff; border-color: var(--primary); }
.rp-foot { margin-top: 16px; text-align: right; }
.rp-reset { background: #fff; border: 1px solid var(--border); border-radius: 9px; padding: 7px 18px; color: var(--text-2); }

/* ─── 列表 ─── */
.res-head { display: flex; justify-content: space-between; align-items: center; font-size: 12.5px; color: var(--muted); margin-bottom: 10px; padding: 0 2px; }
.loading-dot { color: var(--primary); }
.card-list { display: flex; flex-direction: column; gap: 10px; }
.cand { display: flex; align-items: stretch; background: #fff; border: 1px solid var(--border-soft); border-radius: 14px; overflow: hidden; cursor: pointer; transition: .15s; box-shadow: var(--shadow-sm); }
.cand:hover { transform: translateY(-2px); box-shadow: var(--shadow-lg); border-color: var(--border); }
.cand-tier { flex: none; width: 40px; display: flex; align-items: center; justify-content: center; color: #fff; font-size: 16px; font-weight: 800; font-family: var(--serif); }
.cand-main { flex: 1; padding: 14px 18px; min-width: 0; }
.cand-top { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.cand-school { font-size: 15.5px; font-weight: 800; color: var(--ink); }
.badge985, .badge211, .badgedf { font-size: 10px; font-weight: 700; padding: 1px 6px; border-radius: 5px; }
.badge985 { background: #fbe9e7; color: #c0392b; }
.badge211 { background: #e9f0fb; color: #2c4661; }
.badgedf { background: #f5efe2; color: #a78c4f; }
.cand-region { font-size: 12px; color: var(--muted); margin-left: auto; }
.cand-major { font-size: 13.5px; color: var(--text-2); margin-top: 7px; font-weight: 600; }
.cand-sg { font-size: 11.5px; color: var(--dim); margin-top: 3px; }
.cand-data { flex: none; width: 150px; display: flex; align-items: center; gap: 14px; padding: 0 18px; border-left: 1px solid var(--border-soft); }
.cd-prob { display: flex; flex-direction: column; align-items: center; }
.cd-prob-n { font-size: 20px; font-weight: 800; line-height: 1; }
.cd-prob-l { font-size: 10px; color: var(--muted); margin-top: 3px; }
.cd-rank { font-size: 11.5px; color: var(--muted); line-height: 1.7; }
.cd-rank b { color: var(--text); font-size: 13px; }

.empty { text-align: center; color: var(--muted); padding: 60px 20px; font-size: 13.5px; }
.pager { display: flex; align-items: center; justify-content: center; gap: 16px; margin-top: 22px; font-size: 13px; color: var(--text-2); }
.pager button { background: #fff; border: 1px solid var(--border); border-radius: 9px; padding: 8px 18px; }
.pager button:disabled { opacity: .4; }

@media (max-width: 860px) { .dashboard { grid-template-columns: 1fr; } }
</style>
