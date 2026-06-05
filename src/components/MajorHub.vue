<script setup lang="ts">
import { ref, onMounted } from "vue";
import { gk } from "../tauri";

interface MajorRow { major_code: string; name: string; category: string; subcategory: string; degree: string }
const q = ref("");
const rows = ref<MajorRow[]>([]);
const loading = ref(false);

// 本科 12 大学科门类（教育部目录）
const CATEGORIES = [
  { name: "哲学", icon: "🧭" }, { name: "经济学", icon: "💹" }, { name: "法学", icon: "⚖️" },
  { name: "教育学", icon: "🎓" }, { name: "文学", icon: "📖" }, { name: "历史学", icon: "🏛" },
  { name: "理学", icon: "🔬" }, { name: "工学", icon: "⚙️" }, { name: "农学", icon: "🌾" },
  { name: "医学", icon: "⚕️" }, { name: "管理学", icon: "📊" }, { name: "艺术学", icon: "🎨" },
];

async function search() {
  loading.value = true;
  try {
    rows.value = await gk.lookupMajor(q.value.trim());
  } catch {
    rows.value = [];
  } finally {
    loading.value = false;
  }
}
function pick(cat: string) {
  q.value = cat;
  search();
}
onMounted(search);
</script>

<template>
  <div class="page">
    <header class="ph">
      <h1>专业库</h1>
      <p>本科 12 大学科门类 · 793 个专业。搜索专业名了解所属门类与授予学位。</p>
    </header>

    <div class="searchbar">
      <input v-model="q" placeholder="搜专业，如 计算机 / 临床医学 / 法学" @keyup.enter="search" />
      <button @click="search">搜索</button>
    </div>

    <div class="cats">
      <button v-for="c in CATEGORIES" :key="c.name" class="cat" @click="pick(c.name)">
        <span class="cat-ic">{{ c.icon }}</span>
        <span class="cat-name">{{ c.name }}</span>
      </button>
    </div>

    <div class="res-title">{{ loading ? "查询中…" : `搜索结果 ${rows.length} 条` }}</div>
    <div class="major-list">
      <div v-for="m in rows" :key="m.major_code + m.name" class="major">
        <div class="m-name">{{ m.name }}</div>
        <div class="m-meta">
          <span class="m-tag" v-if="m.category">{{ m.category }}</span>
          <span v-if="m.subcategory" class="m-sub">{{ m.subcategory }}</span>
          <span v-if="m.degree" class="m-deg">{{ m.degree }}学位</span>
        </div>
      </div>
      <div v-if="!loading && !rows.length" class="empty">没有找到相关专业，换个关键词试试。</div>
    </div>
  </div>
</template>

<style scoped>
.page { height: 100vh; overflow-y: auto; max-width: 880px; margin: 0 auto; padding: 36px 28px 60px; }
.ph h1 { font-family: var(--serif); font-size: 26px; margin: 0; color: var(--ink); }
.ph p { color: var(--muted); font-size: 13.5px; margin: 8px 0 22px; }
.searchbar { display: flex; gap: 10px; margin-bottom: 22px; }
.searchbar input { flex: 1; border: 1.5px solid var(--border); border-radius: 12px; padding: 12px 16px; font-size: 14px; outline: none; }
.searchbar input:focus { border-color: var(--primary); }
.searchbar button { background: var(--primary); color: #fff; border: none; border-radius: 12px; padding: 0 26px; font-weight: 700; font-size: 14px; }
.cats { display: grid; grid-template-columns: repeat(6, 1fr); gap: 10px; margin-bottom: 28px; }
.cat { display: flex; flex-direction: column; align-items: center; gap: 6px; padding: 16px 6px; background: #fff; border: 1px solid var(--border-soft); border-radius: 14px; transition: .15s; box-shadow: var(--shadow-sm); }
.cat:hover { transform: translateY(-2px); border-color: var(--primary); box-shadow: var(--shadow-lg); }
.cat-ic { font-size: 22px; }
.cat-name { font-size: 12.5px; color: var(--text-2); font-weight: 600; }
.res-title { font-size: 12.5px; color: var(--muted); margin-bottom: 12px; }
.major-list { display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px; }
.major { background: #fff; border: 1px solid var(--border-soft); border-radius: 12px; padding: 14px 16px; box-shadow: var(--shadow-sm); }
.m-name { font-size: 14.5px; font-weight: 700; color: var(--ink); }
.m-meta { display: flex; gap: 8px; align-items: center; margin-top: 7px; flex-wrap: wrap; }
.m-tag { background: var(--primary-soft); color: var(--primary-deep); font-size: 11px; padding: 2px 9px; border-radius: 6px; font-weight: 700; }
.m-sub { font-size: 12px; color: var(--text-2); }
.m-deg { font-size: 11px; color: var(--muted); }
.empty { grid-column: 1 / -1; text-align: center; color: var(--muted); padding: 40px; font-size: 13px; }
@media (max-width: 720px) { .cats { grid-template-columns: repeat(4, 1fr); } .major-list { grid-template-columns: 1fr; } }
</style>
