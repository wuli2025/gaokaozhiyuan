<script setup lang="ts">
import { ref, onMounted } from "vue";
import { claudeMd } from "../tauri";

const content = ref("");
const loading = ref(true);
onMounted(async () => {
  try {
    content.value = await claudeMd.read("kb");
  } catch {
    content.value = "";
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="page">
    <header class="ph">
      <h1>知识规则</h1>
      <p>知识库的行为准则（CLAUDE.md）—— AI 回答志愿问题时遵循的取证与反幻觉规范。</p>
    </header>
    <div v-if="loading" class="muted">加载中…</div>
    <pre v-else-if="content" class="md">{{ content }}</pre>
    <div v-else class="muted">未读取到知识库规则文件（预览模式或文件不存在）。</div>
  </div>
</template>

<style scoped>
.page { height: 100vh; overflow-y: auto; max-width: 820px; margin: 0 auto; padding: 36px 28px 60px; }
.ph h1 { font-family: var(--serif); font-size: 26px; margin: 0; color: var(--ink); }
.ph p { color: var(--muted); font-size: 13.5px; margin: 8px 0 22px; }
.muted { color: var(--muted); font-size: 13px; padding: 30px 0; }
.md { white-space: pre-wrap; word-break: break-word; background: #fff; border: 1px solid var(--border-soft); border-radius: 14px; padding: 22px 24px; font-size: 13px; line-height: 1.85; color: var(--text-2); box-shadow: var(--shadow-sm); font-family: var(--sans); }
</style>
