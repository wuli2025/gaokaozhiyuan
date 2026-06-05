<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { gk, type GkSchoolDetail } from "../tauri";

const props = defineProps<{
  name: string;
  province: string;
  track: string;
  userRank: number;
}>();
const emit = defineEmits<{ (e: "close"): void }>();

const data = ref<GkSchoolDetail | null>(null);
const loading = ref(true);

onMounted(async () => {
  try {
    data.value = await gk.schoolDetail(props.name, props.province, props.track);
  } catch {
    data.value = null;
  } finally {
    loading.value = false;
  }
});

const info = computed(() => data.value?.info);
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
          <span class="sd-rankbar-r">{{ province }} · {{ track }}类 · 2024 录取数据</span>
        </div>

        <div class="sd-section-title">
          专业录取分 · 位次
          <span class="sd-count">{{ data?.majors.length || 0 }} 个专业</span>
        </div>

        <div class="sd-majors">
          <div v-if="!data?.majors.length" class="sd-empty">本省该科类暂无该校专业录取数据</div>
          <div
            v-for="(m, i) in data?.majors"
            :key="i"
            class="sd-major"
          >
            <div class="sm-tier" :style="{ background: prob(m.min_rank).color }">{{ prob(m.min_rank).tier }}</div>
            <div class="sm-main">
              <div class="sm-name">{{ m.major }}</div>
              <div class="sm-sg" v-if="m.subject_group">{{ m.subject_group }}</div>
            </div>
            <div class="sm-nums">
              <div class="sm-prob" :style="{ color: prob(m.min_rank).color }">{{ prob(m.min_rank).p }}%</div>
              <div class="sm-rank">
                <span v-if="m.min_score"><b>{{ m.min_score }}</b>分</span>
                <span><b>{{ m.min_rank?.toLocaleString() }}</b>位</span>
              </div>
            </div>
          </div>
        </div>
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

.sd-majors { display: flex; flex-direction: column; gap: 8px; }
.sd-empty { text-align: center; color: var(--muted); padding: 30px 0; font-size: 13px; }
.sd-major { display: flex; align-items: stretch; border: 1px solid var(--border-soft); border-radius: 11px; overflow: hidden; }
.sm-tier { flex: none; width: 32px; display: flex; align-items: center; justify-content: center; color: #fff; font-weight: 800; font-size: 14px; }
.sm-main { flex: 1; padding: 10px 14px; min-width: 0; }
.sm-name { font-size: 13.5px; font-weight: 700; color: var(--text); }
.sm-sg { font-size: 11px; color: var(--dim); margin-top: 3px; }
.sm-nums { flex: none; display: flex; align-items: center; gap: 14px; padding: 0 16px; }
.sm-prob { font-size: 17px; font-weight: 800; }
.sm-rank { font-size: 11px; color: var(--muted); display: flex; flex-direction: column; gap: 2px; text-align: right; }
.sm-rank b { color: var(--text); font-size: 12.5px; }
</style>
