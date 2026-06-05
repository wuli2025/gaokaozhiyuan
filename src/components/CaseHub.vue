<script setup lang="ts">
const CASES = [
  { who: "湖北物理类 · 位次约 9800", q: "想学航天，该冲哪些？", a: "用位次锁定可报院校组，再沿专业前景与就业去向逐条比对，最终冲 2 稳 4 保 3，主攻西工大 / 哈工大方向。", tags: ["航空航天", "冲稳保"] },
  { who: "江苏历史类 · 位次约 4200", q: "文科怎么选不踩坑？", a: "避开收缩中的纯财经，向法学 + 城市（南京 / 上海）倾斜，优先双一流的优势专业。", tags: ["文科", "城市优先"] },
  { who: "广东物理类 · 中段", q: "冲名校还是保专业？", a: "分数中段建议专业优先，选强二线城市的行业特色校，实习与就业网络更实在。", tags: ["专业优先"] },
  { who: "家长提问 · 计算机", q: "35 岁危机是真的吗？", a: "需求仍在但更新快，靠持续学习与转管理 / 架构对冲；不必因焦虑放弃，但要认清节奏。", tags: ["IT", "祛魅"] },
  { who: "家长提问 · 学医", q: "学医到底稳不稳？", a: "门槛高、周期长（规培），但越老越吃香，稳定性强；要做好长线投入的准备。", tags: ["医学"] },
  { who: "复读生 · 再选纠结", q: "再选选物化还是物生？", a: "物化覆盖专业最广（尤其工科 / 计算机），物生限制更多；以目标专业要求倒推。", tags: ["选科", "双闸"] },
];
</script>

<template>
  <div class="page">
    <header class="ph">
      <h1>真实案例</h1>
      <p>看别人怎么权衡——同样的分数，不同的取舍，背后是同一套方法。</p>
    </header>
    <div class="timeline">
      <div v-for="(c, i) in CASES" :key="i" class="case">
        <div class="dot"></div>
        <div class="card">
          <div class="who">{{ c.who }}</div>
          <div class="q">“{{ c.q }}”</div>
          <div class="a">{{ c.a }}</div>
          <div class="tags"><span v-for="t in c.tags" :key="t">#{{ t }}</span></div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page { height: 100vh; overflow-y: auto; max-width: 720px; margin: 0 auto; padding: 36px 28px 60px; }
.ph h1 { font-family: var(--serif); font-size: 26px; margin: 0; color: var(--ink); }
.ph p { color: var(--muted); font-size: 13.5px; margin: 8px 0 22px; }
.timeline { position: relative; padding-left: 22px; }
.timeline::before { content: ""; position: absolute; left: 5px; top: 6px; bottom: 6px; width: 2px; background: var(--border); }
.case { position: relative; margin-bottom: 16px; }
.dot { position: absolute; left: -22px; top: 18px; width: 11px; height: 11px; border-radius: 50%; background: #fff; border: 3px solid var(--primary); }
.card { background: #fff; border: 1px solid var(--border-soft); border-radius: 14px; padding: 16px 20px; box-shadow: var(--shadow-sm); }
.who { font-size: 11.5px; color: var(--muted); font-weight: 600; }
.q { font-size: 15px; font-weight: 800; color: var(--ink); margin: 6px 0 9px; font-family: var(--serif); }
.a { font-size: 13px; color: var(--text-2); line-height: 1.8; }
.tags { display: flex; gap: 10px; margin-top: 11px; }
.tags span { font-size: 11.5px; color: var(--primary); }
</style>
