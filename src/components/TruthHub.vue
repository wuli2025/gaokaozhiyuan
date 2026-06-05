<script setup lang="ts">
import { ref } from "vue";
const open = ref<number | null>(0);
const CARDS = [
  { t: "“热门专业”陷阱", cold: "报考时热 = 四年后供给过剩、内卷加剧。", hot: "真正稳的是国家长期需要的硬技能，不是当下最火的名字。" },
  { t: "“服从调剂”要不要勾", cold: "不勾，专业满额就退档，整个批次滑走。", hot: "冲的院校建议勾保命；分数有余量的保底校可不勾。" },
  { t: "名校光环 vs 专业", cold: "为冲名校接受天坑专业，四年痛苦、就业受限。", hot: "分数顶尖看学校，分数中段优先看专业与城市。" },
  { t: "“大类招生”的水", cold: "进校后分流，热门方向卷绩点，可能去不了想去的方向。", hot: "看清分流规则与比例，别被“大类”名字迷惑。" },
  { t: "城市的隐形价值", cold: "同层次院校，城市决定实习、视野与就业网络。", hot: "一线 / 强二线的普通校，机会常胜过偏远地区的好学校。" },
  { t: "中外合作 / 高收费专业", cold: "分数低、学费高，部分项目性价比存疑。", hot: "认准办学资质与升学出口，别只图“低分上名校”。" },
];
</script>

<template>
  <div class="page">
    <header class="ph">
      <h1>避坑真话</h1>
      <p>把冷面和热面都讲清楚——好的一面要知道，坑也要提前看见。</p>
    </header>
    <div class="cards">
      <div v-for="(c, i) in CARDS" :key="c.t" class="card" :class="{ open: open === i }" @click="open = open === i ? null : i">
        <div class="c-head">
          <span class="c-t">{{ c.t }}</span>
          <span class="c-chev">{{ open === i ? "−" : "+" }}</span>
        </div>
        <div v-if="open === i" class="c-body">
          <div class="face cold"><span class="f-tag">⚠ 冷面</span>{{ c.cold }}</div>
          <div class="face hot"><span class="f-tag">✓ 热面</span>{{ c.hot }}</div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page { height: 100vh; overflow-y: auto; max-width: 720px; margin: 0 auto; padding: 36px 28px 60px; }
.ph h1 { font-family: var(--serif); font-size: 26px; margin: 0; color: var(--ink); }
.ph p { color: var(--muted); font-size: 13.5px; margin: 8px 0 22px; }
.cards { display: flex; flex-direction: column; gap: 10px; }
.card { background: #fff; border: 1px solid var(--border-soft); border-radius: 14px; padding: 16px 20px; cursor: pointer; box-shadow: var(--shadow-sm); transition: .15s; }
.card:hover { border-color: var(--border); }
.card.open { box-shadow: var(--shadow); }
.c-head { display: flex; justify-content: space-between; align-items: center; }
.c-t { font-size: 14.5px; font-weight: 700; color: var(--ink); }
.c-chev { color: var(--dim); font-size: 18px; }
.c-body { margin-top: 14px; display: flex; flex-direction: column; gap: 10px; }
.face { font-size: 13px; line-height: 1.7; padding: 11px 14px; border-radius: 10px; }
.face.cold { background: #fdeeec; color: #8b2e23; }
.face.hot { background: #eaf5ef; color: #1f6b4a; }
.f-tag { font-weight: 700; margin-right: 8px; }
</style>
