<script setup lang="ts">
import { ref, reactive, onMounted, computed } from "vue";
import { gk, type GkSchoolDetail, type GkGroupMajors } from "../tauri";

const props = defineProps<{
  name: string;
  province: string;
  track: string;
  userRank: number;
}>();
const emit = defineEmits<{ (e: "close"): void }>();

const data = ref<GkSchoolDetail | null>(null);
const loading = ref(true);
const selectedYear = ref<number>(0);

onMounted(async () => {
  try {
    data.value = await gk.schoolDetail(props.name, props.province, props.track);
    selectedYear.value = data.value?.years?.[0] ?? 0;
  } catch {
    data.value = null;
  } finally {
    loading.value = false;
  }
});

const info = computed(() => data.value?.info);
const years = computed(() => data.value?.years ?? []);

// 当前选中年份的专业录取行(已按位次升序)
const yearMajors = computed(() =>
  (data.value?.majors ?? []).filter((m) => m.year === selectedYear.value)
);

// 组级下钻: 广东"院校专业组"投档是一组一条线, 点开看组内专业(取最近招生计划构成)
type GroupState = { open: boolean; loading: boolean; data: GkGroupMajors | null };
const groupState = reactive<Record<string, GroupState>>({});
function gkey(m: { group_code: string | null; year: number }): string {
  return `${m.year}|${m.group_code ?? ""}`;
}
async function toggleGroup(m: { group_code: string | null; year: number }) {
  if (!m.group_code) return; // 只有带专业组码的(组级投档行)能下钻
  const k = gkey(m);
  const st = groupState[k] ?? (groupState[k] = { open: false, loading: false, data: null });
  st.open = !st.open;
  if (st.open && !st.data && !st.loading) {
    st.loading = true;
    try {
      st.data = await gk.groupMajors(props.name, m.group_code, m.year);
    } catch {
      st.data = { majors: [], plan_year: null };
    } finally {
      st.loading = false;
    }
  }
}

// 各年选科串写法不一(2023只有"首选物理", 2022/24是"首选物理，再选不限"),
// 对比时只按"首选X"归一, 让同一专业能跨年对上
function firstChoice(sg: string | null): string {
  if (!sg) return "";
  return sg.split(/[，,]/)[0].trim();
}

// 专业名归一: 去掉（中外合作）/（办学地点…）等括注尾巴, 让同一专业跨年对上
function normMajor(name: string): string {
  return name.replace(/[（(].*$/, "").trim();
}

// 近三年同专业位次对比: 按 专业+首选科 聚合, 取每年最低位次, 仅保留出现≥2年的专业
type Cmp = { name: string; sg: string | null; byYear: Record<number, { rank: number | null; score: number | null }> };
const compare = computed<Cmp[]>(() => {
  const ys = years.value;
  if (ys.length < 2) return [];
  const map = new Map<string, Cmp>();
  for (const m of data.value?.majors ?? []) {
    const nm = normMajor(m.major);
    const key = nm + "|" + firstChoice(m.subject_group);
    let row = map.get(key);
    if (!row) { row = { name: nm, sg: firstChoice(m.subject_group), byYear: {} }; map.set(key, row); }
    const cur = row.byYear[m.year];
    // 同年同名多个专业组时取最低位次(最易进的那条), 口径统一
    if (!cur || (m.min_rank != null && (cur.rank == null || m.min_rank < cur.rank)))
      row.byYear[m.year] = { rank: m.min_rank, score: m.min_score };
  }
  return [...map.values()]
    .filter((r) => ys.filter((y) => r.byYear[y]?.rank != null).length >= 2)
    .sort((a, b) => {
      const ra = a.byYear[ys[0]]?.rank ?? a.byYear[ys[1]]?.rank ?? 9e9;
      const rb = b.byYear[ys[0]]?.rank ?? b.byYear[ys[1]]?.rank ?? 9e9;
      return ra - rb;
    });
});

// 选中年份的数据来源/口径说明(2025=省考试院官方组级投档线; 2017-24=历年志愿数据整理)
const srcNote = computed(() => {
  const y = selectedYear.value;
  if (y >= 2025) return `${y} 为${props.province}省考试院官方投档线（院校专业组级·投档最低分${hasRank.value ? "/位次" : "，部分省位次由一分一段估算"}）`;
  return `${y} 为历年志愿数据整理（专业级最低分/位次），仅供参考`;
});
// 当前年份是否真有位次(广东等省官方带位次; 江苏等仅分)
const hasRank = computed(() => yearMajors.value.some((m) => m.min_rank != null));
const tags = computed(() => {
  const i = info.value;
  if (!i) return [];
  const out: string[] = [];
  if (i.is985) out.push("985");
  if (i.is211) out.push("211");
  if (i.double_first?.includes("双一流")) out.push("双一流");
  if (i.is_c9) out.push("C9");
  if (i.is_central) out.push("中央部属");
  return out;
});

function prob(minRank: number | null): { p: number; tier: string; color: string } {
  if (!minRank || !props.userRank) return { p: 0, tier: "—", color: "#999" };
  const diff = (minRank - props.userRank) / props.userRank;
  const p = 1 / (1 + Math.exp(-diff / 0.16));
  const tier = p >= 0.78 ? "保" : p >= 0.45 ? "稳" : "冲";
  const color = tier === "冲" ? "#e0483b" : tier === "稳" ? "#e8973a" : "#1f9d6b";
  return { p: Math.round(p * 100), tier, color };
}
</script>

<template>
  <div class="modal-mask" @click.self="emit('close')">
    <div class="modal">
      <button class="x" @click="emit('close')">✕</button>

      <div v-if="loading" class="loading">加载院校信息…</div>

      <template v-else-if="info">
        <header class="sd-head">
          <div class="sd-avatar">{{ info.name.slice(0, 1) }}</div>
          <div class="sd-titles">
            <h2>{{ info.name }}</h2>
            <div class="sd-tags">
              <span v-for="t in tags" :key="t" class="sd-tag">{{ t }}</span>
              <span v-if="info.school_type" class="sd-tag plain">{{ info.school_type }}</span>
            </div>
            <div class="sd-meta">
              <span v-if="info.city">📍 {{ info.city }}</span>
              <span v-if="info.dept">🏛 {{ info.dept }}</span>
            </div>
          </div>
        </header>

        <div class="sd-rankbar" v-if="userRank">
          <span>你的位次 <b>{{ userRank.toLocaleString() }}</b></span>
          <span class="sd-rankbar-r">{{ province }} · {{ track }}类 · {{ selectedYear || "—" }} 录取数据</span>
        </div>

        <!-- 年份切换 -->
        <div class="sd-years" v-if="years.length">
          <button
            v-for="y in years"
            :key="y"
            class="sd-year"
            :class="{ on: y === selectedYear }"
            @click="selectedYear = y"
          >{{ y }}</button>
          <span class="sd-years-hint" v-if="years.length > 1">近 {{ years.length }} 年可比</span>
        </div>

        <div class="sd-section-title">
          {{ selectedYear }} 专业录取分 · 位次
          <span class="sd-count">{{ yearMajors.length }} 个专业</span>
        </div>

        <div class="sd-majors">
          <div v-if="!yearMajors.length" class="sd-empty">本省该科类暂无该校 {{ selectedYear }} 年专业录取数据</div>
          <div v-for="(m, i) in yearMajors" :key="i" class="sd-major-wrap">
            <div
              class="sd-major"
              :class="{ clickable: !!m.group_code }"
              @click="toggleGroup(m)"
            >
              <div class="sm-tier" :style="{ background: prob(m.min_rank).color }">{{ prob(m.min_rank).tier }}</div>
              <div class="sm-main">
                <div class="sm-name">
                  <span v-if="m.group_code" class="sm-caret" :class="{ open: groupState[gkey(m)]?.open }">▸</span>
                  {{ m.major }}
                </div>
                <div class="sm-sg" v-if="m.subject_group">{{ m.subject_group }}<span v-if="m.group_code" class="sm-hint"> · 点开看组内专业</span></div>
              </div>
              <div class="sm-nums">
                <div class="sm-prob" :style="{ color: prob(m.min_rank).color }">{{ prob(m.min_rank).p }}%</div>
                <div class="sm-rank">
                  <span v-if="m.min_score"><b>{{ m.min_score }}</b>分</span>
                  <span><b>{{ m.min_rank?.toLocaleString() }}</b>位</span>
                </div>
              </div>
            </div>
            <!-- 组内专业构成下钻 -->
            <div v-if="m.group_code && groupState[gkey(m)]?.open" class="sd-sub">
              <div v-if="groupState[gkey(m)]?.loading" class="sub-loading">载入组内专业…</div>
              <template v-else>
                <div class="sub-head" v-if="groupState[gkey(m)]?.data?.plan_year">
                  本组共 {{ groupState[gkey(m)]?.data?.majors.length || 0 }} 个专业 · 同组投档同一条线，进组后按分数分配专业
                  <span class="sub-src">构成参考 {{ groupState[gkey(m)]?.data?.plan_year }} 年招生计划</span>
                </div>
                <div class="sub-empty" v-if="!groupState[gkey(m)]?.data?.majors.length">暂无该组专业构成数据</div>
                <div v-for="(sm, j) in groupState[gkey(m)]?.data?.majors" :key="j" class="sub-major">
                  <span class="sub-name">{{ sm.major }}</span>
                  <span class="sub-plan" v-if="sm.plan_count != null">计划 {{ sm.plan_count }} 人</span>
                </div>
              </template>
            </div>
          </div>
        </div>
        <p class="src-note" v-if="yearMajors.length">ℹ {{ srcNote }}</p>

        <!-- 近三年同专业位次对比 -->
        <template v-if="compare.length">
          <div class="sd-section-title">近三年位次走势 <span class="sd-count">同专业逐年对比 · 位次越小越难</span></div>
          <div class="cmp">
            <div class="cmp-head">
              <span class="cmp-name">专业</span>
              <span v-for="y in years" :key="y" class="cmp-cell">{{ y }}</span>
            </div>
            <div v-for="(r, i) in compare" :key="i" class="cmp-row">
              <span class="cmp-name" :title="r.name">{{ r.name }}</span>
              <span v-for="y in years" :key="y" class="cmp-cell">
                <template v-if="r.byYear[y]?.rank != null">
                  <b>{{ r.byYear[y]!.rank!.toLocaleString() }}</b>
                  <em v-if="r.byYear[y]?.score">{{ r.byYear[y]!.score }}分</em>
                </template>
                <span v-else class="cmp-na">—</span>
              </span>
            </div>
          </div>
          <p class="cmp-foot">注：2025 为省考试院官方组级投档线，2017–2024 为历年志愿数据整理，仅供参考；同名专业/组若含多条，此处取当年最低位次。</p>
        </template>
      </template>

      <div v-else class="loading">未找到该院校信息</div>
    </div>
  </div>
</template>

<style scoped>
.modal-mask { position: fixed; inset: 0; z-index: 60; background: rgba(20,20,25,.4); display: flex; align-items: center; justify-content: center; padding: 5vh 20px; }
.modal { position: relative; width: 640px; max-width: 94vw; max-height: 88vh; overflow-y: auto; background: #fff; border-radius: 20px; box-shadow: var(--shadow-lg); padding: 28px 30px; }
.x { position: absolute; top: 18px; right: 18px; width: 30px; height: 30px; border: none; background: var(--bg-soft); border-radius: 8px; color: var(--muted); font-size: 14px; }
.x:hover { background: var(--border); color: var(--text); }
.loading { padding: 80px 0; text-align: center; color: var(--muted); }

.sd-head { display: flex; gap: 16px; align-items: center; }
.sd-avatar { flex: none; width: 60px; height: 60px; border-radius: 16px; background: linear-gradient(135deg, #2c4661, #3a5a7d); color: #fff; display: flex; align-items: center; justify-content: center; font-size: 28px; font-weight: 800; font-family: var(--serif); }
.sd-titles h2 { margin: 0; font-size: 22px; font-family: var(--serif); color: var(--ink); }
.sd-tags { display: flex; gap: 6px; flex-wrap: wrap; margin-top: 6px; }
.sd-tag { font-size: 10.5px; font-weight: 700; padding: 2px 8px; border-radius: 6px; background: #fbe9e7; color: #c0392b; }
.sd-tag.plain { background: var(--bg-soft); color: var(--text-2); }
.sd-meta { display: flex; gap: 14px; margin-top: 7px; font-size: 12.5px; color: var(--muted); }

.sd-rankbar { display: flex; justify-content: space-between; align-items: center; margin: 20px 0 4px; padding: 11px 16px; background: var(--primary-soft); border-radius: 11px; font-size: 13px; color: var(--primary-deep); }
.sd-rankbar b { font-size: 15px; }
.sd-rankbar-r { font-size: 11.5px; color: var(--primary); opacity: .8; }

.sd-section-title { display: flex; align-items: center; justify-content: space-between; font-size: 14px; font-weight: 800; color: var(--ink); margin: 22px 0 12px; }
.sd-count { font-size: 12px; color: var(--muted); font-weight: 400; }

.sd-years { display: flex; align-items: center; gap: 8px; margin: 14px 0 2px; }
.sd-year { border: 1px solid var(--border); background: #fff; color: var(--text-2); font-size: 13px; font-weight: 700; padding: 5px 16px; border-radius: 9px; cursor: pointer; transition: .15s; }
.sd-year:hover { border-color: var(--primary); color: var(--primary); }
.sd-year.on { background: var(--primary); border-color: var(--primary); color: #fff; }
.sd-years-hint { font-size: 11px; color: var(--muted); margin-left: auto; }

.cmp { border: 1px solid var(--border-soft); border-radius: 11px; overflow: hidden; }
.cmp-head, .cmp-row { display: flex; align-items: center; }
.cmp-head { background: var(--bg-soft); font-size: 11.5px; font-weight: 700; color: var(--text-2); }
.cmp-row { border-top: 1px solid var(--border-soft); font-size: 12px; }
.cmp-row:hover { background: var(--primary-soft); }
.cmp-name { flex: 1.5; min-width: 0; padding: 9px 14px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-weight: 600; color: var(--text); }
.cmp-head .cmp-name { font-weight: 700; }
.cmp-cell { flex: 1; padding: 9px 8px; text-align: center; display: flex; flex-direction: column; gap: 1px; }
.cmp-cell b { font-size: 12.5px; color: var(--text); font-weight: 700; }
.cmp-cell em { font-size: 10px; color: var(--muted); font-style: normal; }
.cmp-na { color: var(--border); }
.cmp-foot { font-size: 10.5px; color: var(--muted); margin: 8px 2px 0; line-height: 1.5; }
.src-note { font-size: 11px; color: var(--muted); margin: 10px 2px 0; line-height: 1.5; }

.sd-majors { display: flex; flex-direction: column; gap: 8px; }
.sd-empty { text-align: center; color: var(--muted); padding: 30px 0; font-size: 13px; }
.sd-major-wrap { display: flex; flex-direction: column; }
.sd-major { display: flex; align-items: stretch; border: 1px solid var(--border-soft); border-radius: 11px; overflow: hidden; }
.sd-major.clickable { cursor: pointer; transition: border-color .15s, box-shadow .15s; }
.sd-major.clickable:hover { border-color: var(--primary); box-shadow: 0 2px 8px rgba(0,0,0,.05); }
.sm-caret { display: inline-block; font-size: 10px; color: var(--primary); transition: transform .15s; margin-right: 2px; }
.sm-caret.open { transform: rotate(90deg); }
.sm-hint { color: var(--primary); opacity: .65; }
.sd-sub { margin: 2px 0 4px; padding: 8px 12px 8px 40px; background: var(--bg-soft); border-radius: 0 0 11px 11px; border: 1px solid var(--border-soft); border-top: none; }
.sub-loading, .sub-empty { font-size: 12px; color: var(--muted); padding: 4px 0; }
.sub-head { font-size: 11px; color: var(--text-2); margin-bottom: 6px; display: flex; flex-wrap: wrap; gap: 6px; align-items: center; }
.sub-src { font-size: 10px; color: var(--muted); background: var(--border-soft); padding: 1px 7px; border-radius: 6px; }
.sub-major { display: flex; justify-content: space-between; align-items: center; padding: 5px 0; border-top: 1px dashed var(--border-soft); font-size: 12.5px; }
.sub-name { color: var(--text); }
.sub-plan { color: var(--muted); font-size: 11px; flex: none; }
.sm-tier { flex: none; width: 32px; display: flex; align-items: center; justify-content: center; color: #fff; font-weight: 800; font-size: 14px; }
.sm-main { flex: 1; padding: 10px 14px; min-width: 0; }
.sm-name { font-size: 13.5px; font-weight: 700; color: var(--text); }
.sm-sg { font-size: 11px; color: var(--dim); margin-top: 3px; }
.sm-nums { flex: none; display: flex; align-items: center; gap: 14px; padding: 0 16px; }
.sm-prob { font-size: 17px; font-weight: 800; }
.sm-rank { font-size: 11px; color: var(--muted); display: flex; flex-direction: column; gap: 2px; text-align: right; }
.sm-rank b { color: var(--text); font-size: 12.5px; }
</style>
