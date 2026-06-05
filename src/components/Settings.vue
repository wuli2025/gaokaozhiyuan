<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { LoaderCircle } from "@lucide/vue";
import { kb, isTauri } from "../tauri";
import { useThemeStore, THEMES, type ThemeKey } from "../stores/theme";
import ProviderDock from "./ProviderDock.vue";
import {
  currentVersion,
  updateVersion,
  updateNotes,
  updating,
  updateProgress,
  updateError,
  checking,
  upToDate,
  lastCheckedAt,
  ensureCurrentVersion,
  manualCheck,
  applyUpdate,
} from "../composables/useUpdater";

const theme = useThemeStore();
function pickTheme(k: ThemeKey) {
  theme.set(k);
}

// 打开设置时确保拿到当前版本号（开发/浏览器态拿不到会被静默忽略）
onMounted(ensureCurrentVersion);

const lastCheckedText = computed(() => {
  if (!lastCheckedAt.value) return "";
  const d = new Date(lastCheckedAt.value);
  const p = (n: number) => String(n).padStart(2, "0");
  return `${p(d.getHours())}:${p(d.getMinutes())}`;
});

// 把「端点没有 release / 无网络」这类报错翻成友好中文，避免看着像 bug。
// 返回 { soft } 表示「正常无更新」用淡提示，否则按真错误红字显示。
const updateHint = computed<{ soft: boolean; text: string } | null>(() => {
  const e = updateError.value;
  if (!e) return null;
  if (/release JSON|fetch|network|404|not\s*found|无法|timeout|request/i.test(e)) {
    return {
      soft: true,
      text: "暂无可用更新：尚未发布新版本，或此刻连不上更新服务器。",
    };
  }
  return { soft: false, text: e };
});

const currentRoot = ref("");
const defaultRoot = ref("");
const draft = ref("");
const busy = ref(false);
const message = ref<{ kind: "ok" | "err"; text: string } | null>(null);

async function refresh() {
  currentRoot.value = await kb.root();
  defaultRoot.value = await kb.defaultRoot();
  draft.value = currentRoot.value;
}

onMounted(refresh);

async function pickFolder() {
  if (!isTauri) {
    message.value = { kind: "err", text: "浏览器模式不支持选择目录" };
    return;
  }
  const picked = await open({
    directory: true,
    multiple: false,
    title: "选择 KB 根目录",
  });
  if (typeof picked === "string" && picked) {
    draft.value = picked;
  }
}

async function save() {
  const v = draft.value.trim();
  if (!v) {
    message.value = { kind: "err", text: "路径不能为空" };
    return;
  }
  busy.value = true;
  message.value = null;
  try {
    const n = await kb.setRoot(v);
    await refresh();
    message.value = {
      kind: "ok",
      text: `已切换。重新扫描完成,索引 ${n} 篇文档。`,
    };
  } catch (e) {
    message.value = { kind: "err", text: String(e) };
  } finally {
    busy.value = false;
  }
}

function useDefault() {
  draft.value = defaultRoot.value;
}
</script>

<template>
  <div class="settings">
    <header class="head">
      <div class="eyebrow">★ 高考志愿 · 设置</div>
      <h1>个性化与工作台</h1>
      <p class="sub">配置外观主题与本地路径，让工作台更贴合你的使用习惯。</p>
    </header>

    <!-- ── 外观主题 ─────────────────────────────────────── -->
    <section class="block">
      <div class="b-head">
        <div>
          <div class="b-title">外观主题</div>
          <div class="b-desc">
            选一套配色作为全局风格，<strong>即时生效并自动记忆</strong>。默认采用
            <em>鎏金朱砂 · 喜庆</em>——为高考家庭定制的暖色基调，沉稳而有格调。
          </div>
        </div>
        <span class="now-tag">
          当前 · {{ THEMES.find((t) => t.key === theme.current)?.name }}
        </span>
      </div>

      <div class="theme-grid">
        <button
          v-for="t in THEMES"
          :key="t.key"
          class="tcard"
          :class="{ active: theme.current === t.key }"
          @click="pickTheme(t.key)"
        >
          <span v-if="t.recommended" class="badge-default">默认 · 推荐</span>
          <span class="check" aria-hidden="true">✓</span>

          <!-- 迷你产品预览 -->
          <div class="tprev" :style="{ background: t.paper }">
            <div class="tprev-bar" :style="{ background: t.bar }">
              <i></i><i></i><i></i>
            </div>
            <div class="tprev-body">
              <div class="tprev-side">
                <span :style="{ background: t.dots[0] }"></span>
                <span :style="{ background: t.dots[1] }"></span>
                <span :style="{ background: t.dots[2] }"></span>
              </div>
              <div class="tprev-main">
                <div class="tline w80"></div>
                <div class="tline w95"></div>
                <div class="tline w55"></div>
                <div class="tprev-btn" :style="{ background: t.bar }"></div>
              </div>
            </div>
          </div>

          <div class="tmeta">
            <div class="tname">
              {{ t.name }}<small>{{ t.alias }}</small>
            </div>
            <p class="ttag">{{ t.tagline }}</p>
          </div>
        </button>
      </div>
    </section>

    <!-- ── KB 根目录 ────────────────────────────────────── -->
    <section class="block">
      <div class="b-title">知识库根目录(KB 根)</div>
      <div class="b-desc">
        Polaris 在此目录下维护
        <code>raw/</code> · <code>output/</code> · <code>wiki/</code>
        三层结构。修改后立即生效,索引自动重扫,旧目录不会被删除。
      </div>

      <div class="row labels">
        <span>当前</span>
      </div>
      <div class="row">
        <input class="path-ro" :value="currentRoot" readonly />
      </div>

      <div class="row labels">
        <span>新路径</span>
        <button class="link-btn" @click="useDefault" :disabled="busy">
          填入默认 ({{ defaultRoot }})
        </button>
      </div>
      <div class="row">
        <input
          class="path-in"
          v-model="draft"
          placeholder="例如 C:\Users\mi\Polaris\PolarisKB"
          :disabled="busy"
        />
        <button class="btn" @click="pickFolder" :disabled="busy">浏览…</button>
        <button
          class="btn primary"
          @click="save"
          :disabled="busy || draft.trim() === currentRoot"
        >
          {{ busy ? "正在切换…" : "保存并重扫" }}
        </button>
      </div>

      <div
        v-if="message"
        class="msg"
        :class="{ ok: message.kind === 'ok', err: message.kind === 'err' }"
      >
        {{ message.text }}
      </div>
    </section>

    <!-- ── 应用更新 ─────────────────────────────────────── -->
    <section class="block">
      <div class="b-head">
        <div>
          <div class="b-title">应用更新</div>
          <div class="b-desc">
            Polaris 经 <strong>GitHub Releases</strong> 推送新版本。可随时手动检查；
            发现新版后<em>一键下载安装并自动重启</em>生效——即「关掉、过一会再开就是新版」。
          </div>
        </div>
        <span class="now-tag">当前 · v{{ currentVersion || "—" }}</span>
      </div>

      <div class="row upd-row">
        <button class="btn" @click="manualCheck" :disabled="checking || updating">
          <LoaderCircle
            v-if="checking"
            :size="14"
            :stroke-width="2"
            class="spin"
          />
          <span>{{ checking ? "检查中…" : "检查更新" }}</span>
        </button>
        <span v-if="lastCheckedText" class="upd-meta">
          上次检查 · {{ lastCheckedText }}
        </span>
      </div>

      <!-- 发现新版本 -->
      <div v-if="updateVersion" class="upd-found">
        <div class="upd-found-top">
          <span class="upd-newver">发现新版本 · v{{ updateVersion }}</span>
          <button
            class="btn primary"
            :disabled="updating"
            @click="applyUpdate"
          >
            <LoaderCircle
              v-if="updating"
              :size="14"
              :stroke-width="2"
              class="spin"
            />
            <span>{{
              updating ? `更新中 ${updateProgress}%` : "立即更新并重启"
            }}</span>
          </button>
        </div>
        <div v-if="updateNotes" class="upd-notes">{{ updateNotes }}</div>
        <div v-if="updating" class="upd-bar">
          <div class="upd-bar-fill" :style="{ width: updateProgress + '%' }"></div>
        </div>
      </div>

      <!-- 状态消息 -->
      <div
        v-if="updateHint"
        class="msg"
        :class="updateHint.soft ? 'info' : 'err'"
      >
        {{ updateHint.text }}
      </div>
      <div v-else-if="upToDate && !updateVersion" class="msg ok">
        已是最新版本 ✓
      </div>
    </section>

    <!-- ── AI 服务商（从侧栏收纳进设置） ─────────────────── -->
    <section class="block">
      <div class="b-title">AI 服务商</div>
      <div class="b-desc">
        志愿匹配与冲稳保<strong>不依赖联网 AI</strong>，均由本地真实数据计算；仅
        <em>偶像对话 / AI 咨询</em> 需要配置一个 AI 服务商。普通用户可忽略此项。
      </div>
      <div class="provider-wrap">
        <ProviderDock :collapsed="false" />
      </div>
    </section>

    <section class="block muted">
      <div class="b-title sm">即将开放</div>
      <ul class="todo">
        <li>Claude Code 二进制路径</li>
        <li>沙箱镜像名 / Docker socket</li>
        <li>自定义主题（导入配色方案）</li>
      </ul>
    </section>
  </div>
</template>

<style scoped>
.settings {
  flex: 1;
  overflow-y: auto;
  padding: 40px 56px 80px;
  max-width: 880px;
  margin: 0 auto;
  width: 100%;
}
.head {
  border-bottom: 1px solid var(--hairline);
  padding-bottom: 20px;
  margin-bottom: 32px;
}
.eyebrow {
  font-family: var(--mono);
  font-size: 10.5px;
  font-weight: 700;
  letter-spacing: 0.34em;
  text-transform: uppercase;
  color: transparent;
  background: var(--grad);
  -webkit-background-clip: text;
  background-clip: text;
  margin-bottom: 12px;
}
.head h1 {
  font-family: var(--serif);
  font-size: 24px;
  font-weight: 700;
  letter-spacing: 1.5px;
  margin: 0 0 8px;
  color: var(--ink);
}
.head .sub {
  font-size: 12.5px;
  color: var(--muted);
  margin: 0;
  letter-spacing: 0.4px;
}

.block {
  background: var(--panel);
  border: 1px solid var(--hairline);
  border-radius: var(--radius);
  padding: 24px 26px;
  margin-bottom: 22px;
  box-shadow: var(--shadow-sm);
}
.block.muted {
  background: transparent;
  box-shadow: none;
  border-color: var(--border-soft);
  border-style: dashed;
}
.b-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}
.b-title {
  font-family: var(--serif);
  font-size: 15px;
  font-weight: 700;
  color: var(--ink);
  letter-spacing: 1.2px;
  margin-bottom: 6px;
}
.b-title.sm {
  font-size: 12px;
  color: var(--muted);
  font-weight: 500;
}
.b-desc {
  font-size: 12.5px;
  color: var(--text-2);
  line-height: 1.85;
  margin-bottom: 18px;
  max-width: 560px;
}
.b-desc em {
  font-style: normal;
  font-weight: 700;
  color: var(--primary-deep);
}
.b-desc code {
  background: var(--code-bg);
  color: var(--code-text);
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  font-family: var(--mono);
  font-size: 11.5px;
}
.now-tag {
  flex: none;
  font-family: var(--mono);
  font-size: 10.5px;
  letter-spacing: 0.06em;
  color: var(--primary-deep);
  background: var(--primary-soft);
  border: 1px solid var(--border-soft);
  border-radius: 999px;
  padding: 5px 12px;
  white-space: nowrap;
}

/* ── 主题卡片网格 ──────────────────────────────────────── */
.theme-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}
@media (max-width: 720px) {
  .theme-grid {
    grid-template-columns: 1fr;
  }
}
.tcard {
  position: relative;
  text-align: left;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 12px 12px 14px;
  cursor: pointer;
  overflow: hidden;
  transition: transform 0.2s ease, box-shadow 0.2s ease, border-color 0.2s ease;
}
.tcard:hover {
  transform: translateY(-3px);
  box-shadow: var(--shadow-lg);
  border-color: var(--border-strong);
}
.tcard.active {
  border-color: transparent;
  box-shadow: 0 0 0 2px var(--primary), var(--shadow);
}
.tcard:focus-visible {
  box-shadow: 0 0 0 3px var(--ring);
}

.badge-default {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 3;
  font-family: var(--mono);
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: #fff;
  background: var(--grad);
  border-radius: 999px;
  padding: 3px 9px;
  box-shadow: 0 3px 8px var(--glow);
}
.check {
  position: absolute;
  top: 10px;
  right: 10px;
  z-index: 3;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--primary);
  color: #fff;
  font-size: 12px;
  font-weight: 800;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transform: scale(0.5);
  transition: opacity 0.2s ease, transform 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
  box-shadow: var(--shadow-sm);
}
.tcard.active .check {
  opacity: 1;
  transform: scale(1);
}

/* 迷你预览 */
.tprev {
  border-radius: var(--radius-sm);
  overflow: hidden;
  border: 1px solid var(--hairline);
  box-shadow: var(--shadow-sm);
  margin-bottom: 12px;
}
.tprev-bar {
  height: 26px;
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 0 10px;
}
.tprev-bar i {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.7);
}
.tprev-body {
  display: flex;
  height: 84px;
}
.tprev-side {
  width: 30px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px 0;
  align-items: center;
  border-right: 1px solid rgba(0, 0, 0, 0.05);
}
.tprev-side span {
  width: 11px;
  height: 11px;
  border-radius: 4px;
}
.tprev-main {
  flex: 1;
  padding: 13px 14px;
  display: flex;
  flex-direction: column;
  gap: 7px;
}
.tline {
  height: 6px;
  border-radius: 3px;
  background: rgba(0, 0, 0, 0.08);
}
.tline.w55 { width: 55%; }
.tline.w80 { width: 80%; }
.tline.w95 { width: 95%; }
.tprev-btn {
  margin-top: auto;
  width: 46px;
  height: 14px;
  border-radius: 5px;
}

.tmeta {
  padding: 0 2px;
}
.tname {
  font-family: var(--serif);
  font-size: 14.5px;
  font-weight: 700;
  color: var(--ink);
  letter-spacing: 0.5px;
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.tname small {
  font-family: var(--sans);
  font-size: 10.5px;
  font-weight: 600;
  color: var(--muted);
  letter-spacing: 0.3px;
}
.ttag {
  margin: 5px 0 0;
  font-size: 11.5px;
  line-height: 1.7;
  color: var(--text-2);
}

/* ── KB 根目录区块 ─────────────────────────────────────── */
.row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 10px;
}
.row.labels {
  margin-bottom: 4px;
  font-size: 11.5px;
  color: var(--dim);
  letter-spacing: 1px;
  font-family: var(--serif);
  justify-content: space-between;
}
.path-ro,
.path-in {
  flex: 1;
  padding: 9px 11px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  font-family: var(--mono);
  font-size: 12px;
  background: var(--panel);
  color: var(--text);
}
.path-ro {
  background: var(--bg-soft);
  color: var(--muted);
}
.path-in:focus {
  outline: none;
  border-color: var(--primary);
  box-shadow: 0 0 0 3px var(--ring);
}

.btn {
  padding: 9px 15px;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text-2);
  font-size: 12.5px;
  letter-spacing: 0.5px;
  cursor: pointer;
  transition: 0.18s;
}
.btn:hover:not(:disabled) {
  border-color: var(--ink);
  color: var(--ink);
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.btn.primary {
  background: var(--ink);
  color: #fff;
  border-color: var(--ink);
}
.btn.primary:hover:not(:disabled) {
  background: var(--primary);
  border-color: var(--primary);
}

.link-btn {
  background: transparent;
  border: none;
  color: var(--primary);
  font-size: 11.5px;
  letter-spacing: 0.3px;
  cursor: pointer;
  padding: 0;
}
.link-btn:hover:not(:disabled) {
  text-decoration: underline;
}
.link-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.msg {
  margin-top: 14px;
  padding: 9px 13px;
  border-radius: var(--radius-sm);
  font-size: 12.5px;
  letter-spacing: 0.3px;
}
.msg.ok {
  background: var(--primary-soft);
  color: var(--primary-deep);
  border-left: 2px solid var(--primary);
}
.msg.err {
  background: var(--vermilion-soft);
  color: var(--vermilion);
  border-left: 2px solid var(--vermilion);
}
.msg.info {
  background: var(--bg-soft);
  color: var(--muted);
  border-left: 2px solid var(--border-strong);
}

.provider-wrap {
  border: 1px solid var(--border-soft);
  border-radius: var(--radius);
  overflow: hidden;
  background: var(--bg-soft);
}
.todo {
  margin: 0;
  padding-left: 18px;
  font-size: 12px;
  color: var(--muted);
  line-height: 2;
}

/* ── 应用更新区块 ─────────────────────────────────────── */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}
.upd-row {
  margin-top: 4px;
  margin-bottom: 0;
}
.upd-meta {
  font-family: var(--mono);
  font-size: 11px;
  letter-spacing: 0.04em;
  color: var(--dim);
}
.upd-found {
  margin-top: 16px;
  padding: 14px 16px;
  background: var(--primary-soft);
  border: 1px solid var(--border-soft);
  border-radius: var(--radius-sm);
}
.upd-found-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.upd-newver {
  font-family: var(--serif);
  font-size: 13.5px;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: var(--primary-deep);
}
.upd-notes {
  margin-top: 12px;
  max-height: 120px;
  overflow-y: auto;
  padding: 9px 12px;
  background: var(--panel);
  border: 1px solid var(--hairline);
  border-radius: var(--radius-sm);
  font-size: 11.5px;
  line-height: 1.7;
  color: var(--text-2);
  white-space: pre-wrap;
}
.upd-bar {
  margin-top: 12px;
  height: 5px;
  border-radius: 3px;
  background: var(--border-soft);
  overflow: hidden;
}
.upd-bar-fill {
  height: 100%;
  background: var(--primary);
  border-radius: 3px;
  transition: width 0.2s ease;
}
.spin {
  animation: upd-spin 0.9s linear infinite;
}
@keyframes upd-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
