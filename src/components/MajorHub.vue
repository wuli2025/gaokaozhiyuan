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

// 别被名字骗 · 易混淆专业对照（源：载望资料·专业篇 专业精讲）
const CONFUSE = [
  { a: "口腔医学", b: "牙医（临床），收入高但考研难、开诊所难；≠ 口腔医学技术（做义齿假牙，非医生，授工学学位）" },
  { a: "眼视光医学", b: "眼科医生；≠ 眼视光学（验光配镜）≠ 眼视光技术（专科）" },
  { a: "医学影像学", b: "看片＋诊断，是医生；≠ 医学影像技术（拍片操作，非医生）" },
  { a: "生物医学工程", b: "是工科（医疗器械/信号），不是当医生" },
  { a: "信息与计算科学", b: "本质是数学，不是计算机/IT" },
  { a: "数理基础科学", b: "数学＋物理双修，搞研究的“高分专属”，本科难就业" },
  { a: "心理咨询师", b: "要应用/临床心理学硕士；≠ 心理医生（临床医学·精神病学方向）" },
  { a: "医学技术类", b: "授工学/理学学位、非医学，定位是“配合辅助”" },
];

const THREE = [
  { t: "① 别被名字骗", d: "“生物医学工程”不是医生、“信息与计算科学”是数学、“精算”门槛极高——名字光鲜 ≠ 出路光鲜。先看真实就业，再谈兴趣。" },
  { t: "② 看清要不要读研", d: "很多专业的真相是本科只是半成品(医学、生物、心理、金融、药学、数学)。报考时就要算清：你和家庭能否支撑这条长路？" },
  { t: "③ 兴趣要有证据", d: "“我喜欢”常是幻想，“我做过/查过/能坚持”才是兴趣——没有证据的热爱，撑不过四年硬课。" },
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

    <h2 class="sec">别被名字骗 · 易混淆专业对照</h2>
    <p class="sec-sub">名字光鲜 ≠ 出路光鲜。这些专业最常被望文生义报错。</p>
    <div class="confuse">
      <div v-for="c in CONFUSE" :key="c.a" class="cf-row">
        <div class="cf-a">{{ c.a }}</div>
        <div class="cf-arrow">其实是</div>
        <div class="cf-b">{{ c.b }}</div>
      </div>
    </div>

    <h2 class="sec">选专业 · 三句真话</h2>
    <div class="three">
      <div v-for="t in THREE" :key="t.t" class="th-card">
        <h4>{{ t.t }}</h4>
        <p>{{ t.d }}</p>
      </div>
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

.sec { font-family: var(--serif); font-size: 19px; color: var(--ink); margin: 34px 0 4px; }
.sec-sub { color: var(--muted); font-size: 12.5px; margin: 0 0 14px; }
.confuse { display: flex; flex-direction: column; gap: 8px; }
.cf-row { display: grid; grid-template-columns: 130px 54px 1fr; gap: 10px; align-items: center; background: var(--panel); border: 1px solid var(--border-soft); border-radius: 11px; padding: 11px 14px; box-shadow: var(--shadow-sm); }
.cf-a { font-weight: 700; color: var(--ink); font-size: 13.5px; }
.cf-arrow { font-size: 11px; color: var(--vermilion); font-weight: 700; text-align: center; }
.cf-b { font-size: 12.8px; color: var(--text-2); line-height: 1.7; }
.three { display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; margin-bottom: 20px; }
.th-card { background: var(--panel); border: 1px solid var(--border); border-radius: 14px; padding: 16px 18px; box-shadow: var(--shadow-sm); }
.th-card h4 { margin: 0 0 7px; color: var(--red-deep); font-size: 14px; }
.th-card p { margin: 0; font-size: 12.8px; color: var(--text-2); line-height: 1.75; }
@media (max-width: 720px) { .cats { grid-template-columns: repeat(4, 1fr); } .major-list { grid-template-columns: 1fr; } .three { grid-template-columns: 1fr; } .cf-row { grid-template-columns: 100px 40px 1fr; } }
</style>
