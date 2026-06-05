<script setup lang="ts">
const INDUSTRIES = [
  { name: "信息技术 / 互联网", icon: "💻", hot: "高", majors: "计算机、软件工程、人工智能、电子信息", note: "需求大、起薪高，但更新快、35 岁焦虑真实存在，靠持续学习对冲。", cities: "北京、深圳、杭州、上海" },
  { name: "医疗健康", icon: "⚕️", hot: "稳", majors: "临床医学、口腔、护理、药学", note: "门槛高、周期长（规培），越老越吃香，稳定性强。", cities: "全国，大医院集中在省会" },
  { name: "央国企 / 电力能源", icon: "🏭", hot: "稳", majors: "电气工程、能源动力、自动化、土木", note: "进电网 / 三桶油稳定体面，电气是性价比之王。", cities: "省会及地市均有分布" },
  { name: "高端制造", icon: "⚙️", hot: "升", majors: "机械、材料、车辆、航空航天", note: "国家发力方向，新能源车 / 半导体带动景气度回升。", cities: "苏州、武汉、西安、长三角" },
  { name: "金融财经", icon: "💹", hot: "降", majors: "金融、会计、经济、统计", note: "光环褪去、行业收缩，名校 + 考证 + 实习才有竞争力。", cities: "上海、北京、深圳" },
  { name: "教育 / 公共服务", icon: "🎓", hot: "稳", majors: "师范、法学、行政管理", note: "求稳之选，考编 / 考公是主要出路，地域影响大。", cities: "户籍地 / 意向就业地" },
];
function hotColor(h: string) {
  return h === "高" || h === "升" ? "#e0483b" : h === "降" ? "#8a8a92" : "#1f9d6b";
}
</script>

<template>
  <div class="page">
    <header class="ph">
      <h1>行业前景</h1>
      <p>专业的尽头是行业。先看就业去向与城市分布，再倒推该选什么专业。</p>
    </header>
    <div class="grid">
      <div v-for="ind in INDUSTRIES" :key="ind.name" class="card">
        <div class="c-top">
          <span class="c-ic">{{ ind.icon }}</span>
          <span class="c-name">{{ ind.name }}</span>
          <span class="c-hot" :style="{ color: hotColor(ind.hot), borderColor: hotColor(ind.hot) }">景气{{ ind.hot }}</span>
        </div>
        <div class="c-line"><b>对口专业</b>{{ ind.majors }}</div>
        <div class="c-note">{{ ind.note }}</div>
        <div class="c-cities">📍 {{ ind.cities }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page { height: 100vh; overflow-y: auto; max-width: 880px; margin: 0 auto; padding: 36px 28px 60px; }
.ph h1 { font-family: var(--serif); font-size: 26px; margin: 0; color: var(--ink); }
.ph p { color: var(--muted); font-size: 13.5px; margin: 8px 0 22px; }
.grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 14px; }
.card { background: #fff; border: 1px solid var(--border-soft); border-radius: 16px; padding: 18px 20px; box-shadow: var(--shadow-sm); transition: .15s; }
.card:hover { transform: translateY(-2px); box-shadow: var(--shadow-lg); }
.c-top { display: flex; align-items: center; gap: 9px; }
.c-ic { font-size: 22px; }
.c-name { font-size: 15px; font-weight: 800; color: var(--ink); flex: 1; }
.c-hot { font-size: 11px; font-weight: 700; border: 1px solid; border-radius: 999px; padding: 1px 9px; }
.c-line { font-size: 12.5px; color: var(--text-2); margin-top: 12px; }
.c-line b { color: var(--muted); font-weight: 600; margin-right: 6px; }
.c-note { font-size: 12.5px; color: var(--text-2); margin-top: 9px; line-height: 1.75; }
.c-cities { font-size: 12px; color: var(--muted); margin-top: 10px; }
@media (max-width: 720px) { .grid { grid-template-columns: 1fr; } }
</style>
