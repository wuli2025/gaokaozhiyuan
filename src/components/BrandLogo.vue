<script setup lang="ts">
// 红底高考图标：朱砂渐变圆角方 + 白色学士帽 + 金色流苏星
withDefaults(defineProps<{ size?: number; radius?: number }>(), {
  size: 28,
  radius: 8,
});
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 100 100" class="brand-logo" aria-label="高考志愿">
    <defs>
      <linearGradient id="bl-red" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#e0483b" />
        <stop offset="55%" stop-color="#d23a2c" />
        <stop offset="100%" stop-color="#a82a23" />
      </linearGradient>
      <linearGradient id="bl-shine" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#ffffff" stop-opacity=".28" />
        <stop offset="100%" stop-color="#ffffff" stop-opacity="0" />
      </linearGradient>
    </defs>
    <rect x="3" y="3" width="94" height="94" :rx="radius * 100 / size" fill="url(#bl-red)" />
    <rect x="3" y="3" width="94" height="50" :rx="radius * 100 / size" fill="url(#bl-shine)" />
    <!-- 学士帽顶板 -->
    <polygon points="50,27 86,44 50,61 14,44" fill="#fff" />
    <!-- 帽身 -->
    <path d="M28 52 L28 66 Q50 79 72 66 L72 52 L50 62 Z" fill="#fff" opacity=".93" />
    <!-- 流苏 -->
    <path d="M86 44 L86 67" stroke="#ffd56b" stroke-width="3.4" stroke-linecap="round" fill="none" />
    <circle cx="86" cy="71" r="4.4" fill="#ffd56b" />
  </svg>
</template>

<style scoped>
.brand-logo { display: block; flex: none; filter: drop-shadow(0 2px 5px rgba(168, 42, 35, 0.32)); }
</style>
