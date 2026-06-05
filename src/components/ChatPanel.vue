<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick, watch } from "vue";
import {
  ChevronDown,
  X,
  ArrowRight,
  Square,
  Wrench,
  FileText,
  Table,
  Image as ImageIcon,
  FileCode,
  File as FileIcon,
  ExternalLink,
  Paperclip,
  LoaderCircle,
  Ellipsis,
  PencilLine,
  Pin,
  PinOff,
  Copy,
  Trash2,
  Check,
  PanelRightOpen,
  PanelRightClose,
} from "@lucide/vue";
import {
  chat,
  convApi,
  type PermissionMode,
  type AttachedFile,
  type Message,
} from "../tauri";
import { marked } from "marked";
import { useAppStore } from "../stores/app";
import { useArtifactsStore } from "../stores/artifacts";
import { useChatStore, type Bubble } from "../stores/chat";
import { useFileDrop } from "../composables/useFileDrop";

function fileName(path: string): string {
  return path.split("/").pop() || path;
}

function fileExt(path: string): string {
  const n = fileName(path);
  const i = n.lastIndexOf(".");
  return i >= 0 ? n.slice(i + 1).toLowerCase() : "";
}

function artifactIcon(path: string) {
  const ext = fileExt(path);
  if (["html", "htm", "svg", "js", "ts", "css", "json", "xml"].includes(ext))
    return FileCode;
  if (["png", "jpg", "jpeg", "gif", "webp", "bmp", "ico", "avif"].includes(ext))
    return ImageIcon;
  if (["csv", "tsv", "xlsx", "xls"].includes(ext)) return Table;
  if (["md", "markdown", "txt", "pdf"].includes(ext)) return FileText;
  return FileIcon;
}

const app = useAppStore();
const artifactsStore = useArtifactsStore();
const chatStore = useChatStore();

/** 点击成品文件 chip → 展开右侧抽屉并预览 */
function openArtifact(path: string) {
  app.drawerCollapsed = false;
  artifactsStore.open(path);
}

const input = ref("");
// 多开：当前对话的气泡 / 运行态来自 chat store（按对话 id 维护，切走不丢、后台续流）
const bubbles = computed(() => chatStore.bubblesFor(app.currentConvId));
const sending = computed(() => chatStore.isSending(app.currentConvId));

// ─────────── 回复渲染：markdown + 终端码清洗 ───────────
// 后端发来的是干净 markdown，这里渲染成 HTML（剥掉极少数残留的 ANSI 控制码）。
const ANSI_RE = /\x1b\[[0-9;?]*[ -/]*[@-~]/g;
function renderMd(text: string): string {
  const clean = (text || "").replace(ANSI_RE, "");
  return marked.parse(clean, { gfm: true, breaks: true }) as string;
}

// 工具名 → 友好中文（对话里以优雅 pill 呈现，不再是终端灰块）
const TOOL_LABELS: Record<string, string> = {
  Bash: "运行命令",
  Read: "读取文件",
  Write: "写入文件",
  Edit: "编辑文件",
  MultiEdit: "批量编辑",
  NotebookEdit: "编辑笔记本",
  Glob: "查找文件",
  Grep: "搜索内容",
  WebSearch: "联网搜索",
  WebFetch: "抓取网页",
  Task: "子任务",
  TodoWrite: "更新清单",
};
function toolLabel(n: string): string {
  return TOOL_LABELS[n] ?? n;
}

// 一个「回合」= 一条用户消息 + 其后的助手正文/工具/产物，直到下一条用户消息。
// 助手多段文本拼成一块 markdown；工具折叠成 pill；所有生成文件聚合到回合末尾。
interface Turn {
  key: number;
  user?: Bubble;
  text: string;
  tools: { name: string }[];
  artifacts: string[];
  errors: string[];
  hasAssistant: boolean;
}
const ERR_RE = /^\[(错误|发送失败|result error)/;
const renderTurns = computed<Turn[]>(() => {
  const out: Turn[] = [];
  let cur: Turn | undefined;
  let k = 0;
  const startTurn = (user?: Bubble): Turn => {
    const turn: Turn = {
      key: k++,
      user,
      text: "",
      tools: [],
      artifacts: [],
      errors: [],
      hasAssistant: false,
    };
    out.push(turn);
    cur = turn;
    return turn;
  };
  for (const b of bubbles.value) {
    if (b.role === "user") {
      startTurn(b);
      continue;
    }
    const t: Turn = cur ?? startTurn(undefined);
    if (b.role === "tool") {
      const name = b.tool || "工具";
      // 合并连续同名工具，避免刷屏
      if (t.tools[t.tools.length - 1]?.name !== name) t.tools.push({ name });
    } else {
      const txt = b.text || "";
      if (ERR_RE.test(txt.trim())) {
        t.errors.push(txt);
      } else if (txt) {
        t.text += (t.text ? "\n\n" : "") + txt;
        t.hasAssistant = true;
      }
      if (b.artifacts) {
        for (const a of b.artifacts) if (!t.artifacts.includes(a)) t.artifacts.push(a);
      }
    }
  }
  return out;
});
function isPending(t: Turn): boolean {
  return sending.value && t === renderTurns.value[renderTurns.value.length - 1];
}

// 复制某一回合的回答正文（回答下方的「复制」按钮）
async function copyTurn(t: Turn) {
  if (!t.text) return;
  try {
    await navigator.clipboard.writeText(t.text);
    flashCopied("已复制回答");
  } catch {
    flashCopied("复制失败");
  }
}
const showPermDropdown = ref(false);
const permMode = ref<PermissionMode>("manual");
const scrollEl = ref<HTMLDivElement | null>(null);
const inputEl = ref<HTMLTextAreaElement | null>(null);

// 输入框高度随内容自动增长（仿豆包）：先归零再按 scrollHeight 撑高，到 CSS max-height 后内部滚动。
function autoGrow() {
  const el = inputEl.value;
  if (!el) return;
  el.style.height = "auto";
  el.style.height = `${el.scrollHeight}px`;
}
// 内容变化（手输 / 程序填入 / 发送清空）都重算高度
watch(input, () => nextTick(autoGrow));
onMounted(() => nextTick(autoGrow));

// ─────────── 拖拽上传附件到当前对话 ───────────
const attachments = ref<AttachedFile[]>([]);
/** 上传中的占位（大文件复制需要时间，显示转圈） */
const pendingAttach = ref<{ name: string }[]>([]);

function attachIcon(kind: string) {
  if (kind === "image") return ImageIcon;
  if (kind === "pdf") return FileText;
  if (kind === "office") return Table;
  if (kind === "text") return FileCode;
  return FileIcon;
}

function humanSize(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(0)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

async function onDropFiles(paths: string[]) {
  const convId = await ensureConversation();
  const placeholders = paths.map((p) => ({
    name: p.split(/[\\/]/).pop() || p,
  }));
  pendingAttach.value.push(...placeholders);
  try {
    const res = await chat.attachFiles(convId ?? undefined, paths);
    for (const r of res) {
      if (r.ok) attachments.value.push(r);
      else if (convId)
        chatStore.pushBubble(convId, {
          role: "assistant",
          text: `[附件失败] ${r.name}:${r.error ?? ""}`,
        });
    }
  } catch (e: any) {
    if (convId)
      chatStore.pushBubble(convId, {
        role: "assistant",
        text: `[附件失败] ${e?.message ?? e}`,
      });
  } finally {
    for (const ph of placeholders) {
      const idx = pendingAttach.value.indexOf(ph);
      if (idx >= 0) pendingAttach.value.splice(idx, 1);
    }
  }
}

function removeAttachment(i: number) {
  attachments.value.splice(i, 1);
}

const { isOver: dropOver } = useFileDrop({
  active: () => app.view === "chat",
  onDrop: onDropFiles,
});

const permLabel: Record<PermissionMode, string> = {
  manual: "手动授权",
  auto_current: "自动 · 仅当前会话",
  auto_all: "自动 · 所有会话",
  deny: "拒绝授权",
};


function scrollToBottom() {
  nextTick(() => {
    if (scrollEl.value) scrollEl.value.scrollTop = scrollEl.value.scrollHeight;
  });
}

// 切换对话：加载该对话历史（运行中的对话不会被历史覆盖），滚到底
watch(
  () => app.currentConvId,
  (cid) => {
    chatStore.loadHistory(cid).then(scrollToBottom);
  }
);

// 当前对话气泡变化（含后台流式增量推进）时自动滚到底
watch(bubbles, scrollToBottom, { deep: true });

onMounted(async () => {
  await chatStore.init(); // app 级流式监听只注册一次，按 conversationId 路由
  await chatStore.loadHistory(app.currentConvId);
  scrollToBottom();
});

async function ensureConversation(): Promise<string | null> {
  if (app.currentConvId) return app.currentConvId;
  let pid = app.currentProjectId;
  if (!pid) {
    await app.refreshProjects();
    pid = app.currentProjectId;
  }
  if (!pid) {
    const p = await app.createProject("默认项目");
    pid = p.id;
  }
  const c = await app.createConversation(pid);
  return c.id;
}

async function send() {
  const text = input.value.trim();
  const attached = attachments.value.slice();
  const hasAttach = attached.length > 0;
  // 多开：只拦「当前对话」正在发送，不阻止在别的对话并行发起
  if ((!text && !hasAttach) || sending.value) return;

  const convId = await ensureConversation();
  if (!convId) return;

  // 把附件绝对路径拼进 prompt，让 claude 能用 Read 等工具读取
  let prompt = text || "请查看我上传的附件。";
  if (hasAttach) {
    const lines = attached.map((a) => `- ${a.path}`).join("\n");
    prompt += `\n\n---\n[附件]（用户拖拽上传，可用 Read 等工具读取）：\n${lines}`;
  }

  const display = text || "（仅附件）";

  input.value = "";
  attachments.value = [];
  // 交给 chat store：推 user 气泡 + 调后端 + 记录 reqId/sending（按对话 id，多开）
  await chatStore.send(convId, prompt, display, attached, {
    permissionMode: permMode.value,
  });
}

async function cancel() {
  await chatStore.cancel(app.currentConvId);
}

function pickPerm(m: PermissionMode) {
  permMode.value = m;
  showPermDropdown.value = false;
}

function onKeydown(e: KeyboardEvent) {
  if (e.key !== "Enter") return;
  // Shift+Enter 仍然换行
  if (e.shiftKey) return;
  // 中文/日文等输入法在组合（选词）中按回车是确认候选词，不应发送
  if (e.isComposing || (e as any).keyCode === 229) return;
  e.preventDefault();
  send();
}

async function newChat() {
  let pid = app.currentProjectId;
  if (!pid) {
    await app.refreshProjects();
    pid = app.currentProjectId;
  }
  if (!pid) {
    const p = await app.createProject("默认项目");
    pid = p.id;
  }
  await app.createConversation(pid);
}

// ─────────── 对话「更多」菜单（标题旁 ··· ） ───────────
// 当前对话对象（标题、置顶、复制、删除等操作的目标）
const currentConv = computed(() => {
  const list =
    app.conversationsByProject[app.currentProjectId || ""] || [];
  return list.find((c) => c.id === app.currentConvId) || null;
});

const showConvMenu = ref(false);
function toggleConvMenu() {
  showConvMenu.value = !showConvMenu.value;
}
function closeConvMenu() {
  showConvMenu.value = false;
}
// 点空白处关菜单（菜单与触发按钮内部点击都 .stop，不会误关）
onMounted(() => window.addEventListener("click", closeConvMenu));
onBeforeUnmount(() => window.removeEventListener("click", closeConvMenu));

// 复制反馈小提示（顶栏中央浮现 ~1.6s）
const copied = ref("");
let copiedTimer: ReturnType<typeof setTimeout> | undefined;
function flashCopied(msg: string) {
  copied.value = msg;
  if (copiedTimer) clearTimeout(copiedTimer);
  copiedTimer = setTimeout(() => (copied.value = ""), 1600);
}

// 重命名：标题就地变输入框，Enter 提交 / Esc 取消 / 失焦提交
const renaming = ref(false);
const renameText = ref("");
const renameInput = ref<HTMLInputElement | null>(null);
function openRename() {
  closeConvMenu();
  renameText.value = currentConv.value?.title ?? "";
  renaming.value = true;
  nextTick(() => {
    renameInput.value?.focus();
    renameInput.value?.select();
  });
}
async function commitRename() {
  if (!renaming.value) return;
  const conv = currentConv.value;
  renaming.value = false;
  if (conv) await app.renameConversation(conv, renameText.value);
}
function cancelRename() {
  renaming.value = false;
}

function togglePinCurrent() {
  closeConvMenu();
  if (app.currentConvId) app.togglePin(app.currentConvId);
}

async function copyConvId() {
  closeConvMenu();
  const id = app.currentConvId;
  if (!id) return;
  try {
    await navigator.clipboard.writeText(id);
    flashCopied("已复制会话 ID");
  } catch {
    flashCopied("复制失败");
  }
}

function conversationToMarkdown(title: string, msgs: Message[]): string {
  const lines: string[] = [`# ${title}`, ""];
  for (const msg of msgs) {
    if (msg.role === "tool") continue; // 工具调用噪声不进转写
    const who = msg.role === "user" ? "你" : "志愿助手";
    const body = (msg.content || "").trim();
    if (!body) continue;
    lines.push(`**${who}：**`, "", body, "");
  }
  return lines.join("\n").trim() + "\n";
}

async function copyAsMarkdown() {
  closeConvMenu();
  const conv = currentConv.value;
  if (!conv) return;
  try {
    const msgs = await convApi.getMessages(conv.id);
    await navigator.clipboard.writeText(
      conversationToMarkdown(conv.title, msgs)
    );
    flashCopied("已复制为 Markdown");
  } catch {
    flashCopied("复制失败");
  }
}

async function deleteCurrentConv() {
  closeConvMenu();
  const conv = currentConv.value;
  if (!conv) return;
  if (confirm(`删除对话「${conv.title}」？(消息也会被清空)`)) {
    await app.deleteConversation(conv);
  }
}
</script>

<template>
  <div class="chat" :class="{ 'drag-active': dropOver }">
    <!-- 拖拽上传覆盖层 -->
    <div v-if="dropOver" class="drop-overlay">
      <div class="drop-card">
        <Paperclip :size="30" :stroke-width="1.4" />
        <div class="drop-title">松开以上传到当前对话</div>
        <div class="drop-sub">文件作为附件，发送时供 Claude 读取</div>
      </div>
    </div>
    <div class="chat-top">
      <div class="chat-title">
        <template v-if="app.currentConvId">
          <!-- 重命名：标题就地变输入框 -->
          <input
            v-if="renaming"
            ref="renameInput"
            v-model="renameText"
            class="t-rename"
            @keydown.enter.prevent="commitRename"
            @keydown.esc.prevent="cancelRename"
            @blur="commitRename"
            @click.stop
          />
          <template v-else>
            <Pin
              v-if="app.isPinned(app.currentConvId)"
              :size="12"
              :stroke-width="1.9"
              class="t-pin"
            />
            <span class="t-text">{{ currentConv?.title || "(对话)" }}</span>
          </template>

          <!-- 更多菜单 -->
          <div v-if="!renaming" class="conv-menu-wrap">
            <button
              class="conv-more"
              :class="{ active: showConvMenu }"
              title="更多"
              @click.stop="toggleConvMenu"
            >
              <Ellipsis :size="16" :stroke-width="2" />
            </button>
            <div v-if="showConvMenu" class="conv-menu" @click.stop>
              <button class="cm-item" @click="openRename">
                <PencilLine :size="14" :stroke-width="1.8" />
                <span>重命名对话</span>
              </button>
              <button class="cm-item" @click="togglePinCurrent">
                <component
                  :is="app.isPinned(app.currentConvId) ? PinOff : Pin"
                  :size="14"
                  :stroke-width="1.8"
                />
                <span>{{
                  app.isPinned(app.currentConvId) ? "取消置顶" : "置顶对话"
                }}</span>
              </button>
              <div class="cm-sep"></div>
              <button class="cm-item" @click="copyConvId">
                <Copy :size="14" :stroke-width="1.8" />
                <span>复制会话 ID</span>
              </button>
              <button class="cm-item" @click="copyAsMarkdown">
                <FileText :size="14" :stroke-width="1.8" />
                <span>复制为 Markdown</span>
              </button>
              <div class="cm-sep"></div>
              <button class="cm-item danger" @click="deleteCurrentConv">
                <Trash2 :size="14" :stroke-width="1.8" />
                <span>删除对话</span>
              </button>
            </div>
          </div>
        </template>
        <template v-else>
          <span class="t-text muted">未选择对话</span>
        </template>
      </div>
      <Transition name="copy-fade">
        <div v-if="copied" class="copy-toast">
          <Check :size="13" :stroke-width="2.2" />
          <span>{{ copied }}</span>
        </div>
      </Transition>
      <button
        class="drawer-toggle"
        :title="app.drawerCollapsed ? '展开文件抽屉' : '收起文件抽屉'"
        @click="app.toggleDrawer()"
      >
        <component
          :is="app.drawerCollapsed ? PanelRightOpen : PanelRightClose"
          :size="17"
          :stroke-width="1.7"
        />
      </button>
    </div>

    <div class="messages" ref="scrollEl">
      <div v-if="renderTurns.length === 0" class="hero-wrap">
        <div class="hero">你说,志愿助手答</div>
        <div class="hero-sub">
          本地优先 · 调用 Claude Code · 维基知识库 KB-first 召回
        </div>
      </div>

      <div v-for="t in renderTurns" :key="t.key" class="turn">
        <!-- 用户消息：右侧中性气泡，无头像 -->
        <div v-if="t.user" class="msg user">
          <div class="bubble-user">
            <div v-if="t.user.text" class="u-text">{{ t.user.text }}</div>
            <div
              v-if="t.user.files && t.user.files.length"
              class="attach-chips in-bubble"
            >
              <div
                v-for="f in t.user.files"
                :key="f.path"
                class="attach-chip readonly"
                :title="f.path"
              >
                <component :is="attachIcon(f.kind)" :size="14" :stroke-width="1.7" />
                <span class="ac-name">{{ f.name }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 助手回复：纯文本，无头像无边框（Codex 式） -->
        <div
          v-if="
            t.hasAssistant ||
            t.tools.length ||
            t.artifacts.length ||
            t.errors.length ||
            isPending(t)
          "
          class="msg ai"
        >
          <!-- 工具调用：低调 pill -->
          <div v-if="t.tools.length" class="tool-strip">
            <span v-for="(tl, j) in t.tools" :key="j" class="tool-pill">
              <Wrench :size="11" :stroke-width="1.8" />
              {{ toolLabel(tl.name) }}
            </span>
          </div>

          <!-- 正文：markdown 渲染 -->
          <div v-if="t.text" class="md" v-html="renderMd(t.text)"></div>

          <!-- 生成中：三点呼吸 -->
          <div v-if="isPending(t)" class="typing">
            <span></span><span></span><span></span>
          </div>

          <!-- 错误行 -->
          <div v-for="(e, j) in t.errors" :key="'e' + j" class="err-line">
            {{ e }}
          </div>

          <!-- 生成的文件：统一收在回答末尾 -->
          <div v-if="t.artifacts.length" class="files">
            <div class="files-head">生成的文件 · {{ t.artifacts.length }}</div>
            <div class="files-list">
              <button
                v-for="a in t.artifacts"
                :key="a"
                class="artifact-chip"
                :class="{ active: artifactsStore.current?.path === a }"
                :title="a"
                @click="openArtifact(a)"
              >
                <component
                  :is="artifactIcon(a)"
                  :size="15"
                  :stroke-width="1.7"
                />
                <span class="af-name">{{ fileName(a) }}</span>
                <ExternalLink :size="12" :stroke-width="1.8" class="af-open" />
              </button>
            </div>
          </div>

          <!-- 回答下方操作：复制 -->
          <div
            v-if="t.hasAssistant && t.text && !isPending(t)"
            class="turn-actions"
          >
            <button class="ta-btn" title="复制回答" @click="copyTurn(t)">
              <Copy :size="13" :stroke-width="1.8" />
              <span>复制</span>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 输入区域 -->
    <div class="input-area">
      <!-- 输入卡片 -->
      <div class="input-card">
        <!-- 待发送附件 -->
        <div
          v-if="attachments.length || pendingAttach.length"
          class="attach-chips"
        >
          <div
            v-for="(f, i) in attachments"
            :key="f.path"
            class="attach-chip"
            :title="f.path"
          >
            <component :is="attachIcon(f.kind)" :size="14" :stroke-width="1.7" />
            <span class="ac-name">{{ f.name }}</span>
            <span class="ac-size">{{ humanSize(f.size) }}</span>
            <button class="ac-remove" title="移除" @click="removeAttachment(i)">
              <X :size="11" :stroke-width="2" />
            </button>
          </div>
          <div
            v-for="(p, i) in pendingAttach"
            :key="'pending-' + i"
            class="attach-chip pending"
            :title="p.name"
          >
            <LoaderCircle :size="14" :stroke-width="2" class="spin" />
            <span class="ac-name">{{ p.name }}</span>
          </div>
        </div>
        <textarea
          ref="inputEl"
          v-model="input"
          placeholder="请输入消息 (Enter 发送 · Shift + Enter 换行，可拖文件进来作为附件) …"
          rows="2"
          @keydown="onKeydown"
          @input="autoGrow"
        ></textarea>
        <div class="toolbar">
          <div class="toolbar-left">
            <!-- 工具栏精简：技能 / 深度搜索 / 目标模式 / 请教毛主席 已移除 -->
          </div>
          <div class="toolbar-right">
            <button
              v-if="sending"
              class="send-btn stop"
              title="停止"
              @click="cancel"
            >
              <Square :size="14" :stroke-width="2" fill="currentColor" />
            </button>
            <button
              v-else
              class="send-btn"
              title="发送 (Enter)"
              :disabled="!input.trim()"
              @click="send()"
            >
              <ArrowRight :size="16" :stroke-width="2" />
            </button>
          </div>
        </div>
      </div>

      <!-- 底部授权栏 -->
      <div class="auth-bar">
        <div class="perm-wrap" style="margin-right: 48px;">
          <button
            class="auth-btn"
            :class="{ deny: permMode === 'deny' }"
            @click="showPermDropdown = !showPermDropdown"
          >
            <img
              v-if="permMode !== 'deny'"
              src="../assets/perm-hand.png"
              class="auth-hand"
              alt="授权"
            />
            <span v-else class="auth-deny">⊘</span>
            <span class="auth-label">{{ permLabel[permMode] }}</span>
            <ChevronDown :size="12" :stroke-width="2" />
          </button>
          <div v-if="showPermDropdown" class="dropdown">
            <div
              v-for="m in [
                { k: 'manual', l: '手动授权', d: '每次工具调用前确认' },
                {
                  k: 'auto_current',
                  l: '自动 · 仅当前会话',
                  d: '本会话放行非高危操作',
                },
                {
                  k: 'auto_all',
                  l: '自动 · 所有会话',
                  d: '所有会话放行非高危操作(不绕过权限确认)',
                },
                {
                  k: 'deny',
                  l: '拒绝授权(只读)',
                  d: '禁止写入/执行,只允许 Read/Grep/Glob',
                },
              ]"
              :key="m.k"
              class="perm-row"
              :class="{
                active: permMode === m.k,
                deny: m.k === 'deny',
              }"
              @click="pickPerm(m.k as PermissionMode)"
            >
              <div class="title">{{ m.l }}</div>
              <div class="desc">{{ m.d }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.chat {
  display: flex;
  flex-direction: column;
  height: 100vh;
  position: relative;
}
.chat-top {
  position: relative;
  padding: 16px 30px;
  display: flex;
  align-items: center;
  gap: 12px;
  /* 顶栏与下方回答区无缝连成一片：透明背景、无分隔线，不再是单独的异色条；
     比原来略高更有呼吸感（仿豆包 / Coda） */
  border-bottom: none;
  background: transparent;
}
.chat-title {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: var(--serif);
}
.t-text {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}
.t-text.muted {
  font-weight: 400;
  color: var(--muted);
}
/* 文件抽屉开关（移到顶栏右侧；收起后右侧整列消失，靠它再展开） */
.drawer-toggle {
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 7px;
  background: transparent;
  color: var(--muted);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: background 0.15s, color 0.15s;
}
.drawer-toggle:hover {
  background: var(--selection-bg);
  color: var(--text);
}

/* 已置顶标记（标题前的小别针） */
.t-pin {
  color: var(--gold);
  transform: rotate(35deg);
  flex-shrink: 0;
}

/* 标题就地重命名输入框 */
.t-rename {
  flex: 1;
  min-width: 0;
  max-width: 420px;
  font-family: var(--serif);
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
  padding: 3px 8px;
  border: 1px solid var(--primary);
  border-radius: 6px;
  background: var(--panel);
  outline: none;
  box-shadow: 0 0 0 3px var(--primary-soft);
}

/* ── 对话「更多」菜单 ── */
.conv-menu-wrap {
  position: relative;
  flex-shrink: 0;
}
.conv-more {
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--muted);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s, color 0.15s;
}
.conv-more:hover,
.conv-more.active {
  background: var(--selection-bg);
  color: var(--text);
}
.conv-menu {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  z-index: 40;
  min-width: 184px;
  padding: 5px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 10px;
  box-shadow: var(--shadow-lg);
  animation: cm-pop 130ms ease;
}
@keyframes cm-pop {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
}
.cm-item {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  padding: 8px 10px;
  border: none;
  background: transparent;
  color: var(--text-2);
  font-size: 12.5px;
  border-radius: 6px;
  text-align: left;
}
.cm-item:hover {
  background: var(--bg-soft);
  color: var(--text);
}
.cm-item.danger {
  color: var(--vermilion);
}
.cm-item.danger:hover {
  background: var(--vermilion-soft);
}
.cm-sep {
  height: 1px;
  margin: 5px 8px;
  background: var(--border-soft);
}

/* 复制反馈小提示 */
.copy-toast {
  position: absolute;
  top: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  z-index: 45;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--ink);
  color: #fafaf7;
  font-size: 12px;
  border-radius: 8px;
  box-shadow: var(--shadow-lg);
  pointer-events: none;
}
.copy-fade-enter-active,
.copy-fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}
.copy-fade-enter-from,
.copy-fade-leave-to {
  opacity: 0;
  transform: translate(-50%, -4px);
}

.messages {
  flex: 1;
  overflow-y: auto;
  padding: 40px 32px 16px;
}
.hero-wrap {
  margin: 60px auto 40px;
  text-align: center;
  max-width: 720px;
}
.hero {
  font-family: var(--serif);
  font-size: 36px;
  font-weight: 600;
  letter-spacing: 4px;
  color: var(--ink);
}
.hero-sub {
  margin-top: 16px;
  color: var(--muted);
  font-size: 13px;
  letter-spacing: 0.5px;
}

/* ═══════════ 对话渲染 (Codex 式：纯对话，无头像) ═══════════ */
.turn {
  max-width: 880px;
  margin: 0 auto 22px;
}

/* 用户：右对齐中性灰气泡，无头像 */
.msg.user {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 18px;
}
.bubble-user {
  max-width: 82%;
  background: var(--bg-soft);
  border: 1px solid var(--border-soft);
  border-radius: 16px;
  padding: 9px 15px;
}
.u-text {
  white-space: pre-wrap;
  word-break: break-word;
  font-size: 13.5px;
  line-height: 1.65;
  color: var(--text);
}

/* 助手：纯文本，无头像无边框（Codex 式） */
.msg.ai {
  min-width: 0;
}

/* 工具调用 pill */
.tool-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 10px;
}
.tool-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-2);
  background: var(--bg-soft);
  border: 1px solid var(--border-soft);
  padding: 3px 9px;
  border-radius: 20px;
}
.tool-pill :deep(svg) {
  color: var(--primary);
}

/* 生成中三点 */
.typing {
  display: flex;
  gap: 4px;
  padding: 4px 0 2px;
}
.typing span {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--primary);
  opacity: 0.5;
  animation: typing-bounce 1.2s ease-in-out infinite;
}
.typing span:nth-child(2) {
  animation-delay: 0.18s;
}
.typing span:nth-child(3) {
  animation-delay: 0.36s;
}
@keyframes typing-bounce {
  0%, 80%, 100% {
    transform: translateY(0);
    opacity: 0.4;
  }
  40% {
    transform: translateY(-4px);
    opacity: 1;
  }
}

.err-line {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--vermilion);
  background: var(--vermilion-soft);
  border-radius: 6px;
  padding: 6px 10px;
  margin-top: 8px;
  white-space: pre-wrap;
  word-break: break-word;
}

/* 生成的文件：回答末尾 */
.files {
  margin-top: 12px;
  padding-top: 11px;
  border-top: 1px dashed var(--border);
}
.files-head {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
  letter-spacing: 0.5px;
  color: var(--muted);
  margin-bottom: 8px;
}
.files-head :deep(svg) {
  color: var(--gold);
}
.files-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

/* 回答下方操作行（复制） —— 平时淡出，悬停回答时浮现 */
.turn-actions {
  margin-top: 10px;
  display: flex;
  gap: 6px;
  opacity: 0;
  transition: opacity 0.15s;
}
.msg.ai:hover .turn-actions {
  opacity: 1;
}
.ta-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 9px;
  border: 1px solid var(--border-soft);
  background: var(--panel);
  color: var(--muted);
  font-size: 11.5px;
  border-radius: 7px;
  transition: border-color 0.15s, color 0.15s, background 0.15s;
}
.ta-btn:hover {
  border-color: var(--border);
  color: var(--text);
  background: var(--bg-soft);
}

/* ── markdown 正文排版 ── */
.md {
  font-size: 13.5px;
  line-height: 1.72;
  color: var(--text);
  word-break: break-word;
}
.md :deep(> *:first-child) {
  margin-top: 0;
}
.md :deep(> *:last-child) {
  margin-bottom: 0;
}
.md :deep(h1),
.md :deep(h2),
.md :deep(h3),
.md :deep(h4) {
  font-family: var(--serif);
  line-height: 1.35;
  margin: 1.1em 0 0.5em;
  color: var(--ink);
}
.md :deep(h1) {
  font-size: 1.5em;
}
.md :deep(h2) {
  font-size: 1.3em;
}
.md :deep(h3) {
  font-size: 1.12em;
}
.md :deep(h4) {
  font-size: 1em;
}
.md :deep(p) {
  margin: 0.55em 0;
}
.md :deep(ul),
.md :deep(ol) {
  margin: 0.55em 0;
  padding-left: 1.5em;
}
.md :deep(li) {
  margin: 0.25em 0;
}
.md :deep(li::marker) {
  color: var(--muted);
}
.md :deep(a) {
  color: var(--primary);
  text-decoration: none;
  border-bottom: 1px solid var(--primary-soft);
}
.md :deep(a:hover) {
  border-bottom-color: var(--primary);
}
.md :deep(strong) {
  color: var(--ink);
  font-weight: 600;
}
.md :deep(hr) {
  border: none;
  border-top: 1px solid var(--border);
  margin: 1.1em 0;
}
.md :deep(blockquote) {
  margin: 0.7em 0;
  padding: 0.4em 0.9em;
  border-left: 3px solid var(--primary);
  background: var(--primary-soft);
  border-radius: 0 6px 6px 0;
  color: var(--text-2);
}
.md :deep(blockquote p) {
  margin: 0.2em 0;
}
/* 行内代码 */
.md :deep(:not(pre) > code) {
  font-family: var(--mono);
  font-size: 0.88em;
  background: var(--code-bg);
  color: var(--primary-deep);
  padding: 0.12em 0.4em;
  border-radius: 5px;
  border: 1px solid var(--border-soft);
}
/* 代码块：深色卡片，横向滚动，盒绘对齐 */
.md :deep(pre) {
  background: #0f1b2d;
  color: #dbe6f5;
  border-radius: 10px;
  padding: 13px 15px;
  overflow-x: auto;
  margin: 0.8em 0;
  line-height: 1.55;
}
.md :deep(pre code) {
  font-family: var(--mono);
  font-size: 12.4px;
  background: none;
  border: none;
  padding: 0;
  color: inherit;
  white-space: pre;
}
/* 表格 */
.md :deep(table) {
  border-collapse: collapse;
  width: 100%;
  margin: 0.8em 0;
  font-size: 12.8px;
  display: block;
  overflow-x: auto;
}
.md :deep(th),
.md :deep(td) {
  border: 1px solid var(--border);
  padding: 6px 11px;
  text-align: left;
}
.md :deep(thead th) {
  background: var(--bg-soft);
  font-weight: 600;
  color: var(--text);
}
.md :deep(img) {
  max-width: 100%;
  border-radius: 6px;
}

/* 成品文件 chips —— 回答末尾的可点击文件 */
.artifact-chip {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  max-width: 320px;
  padding: 6px 10px;
  background: var(--bg-soft);
  border: 1px solid var(--border);
  border-radius: 8px;
  color: var(--primary);
  font-size: 12.5px;
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s;
}
.artifact-chip:hover {
  border-color: var(--primary);
  background: var(--primary-soft);
}
.artifact-chip.active {
  border-color: var(--primary);
  background: var(--primary-soft);
}
.artifact-chip .af-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}
.artifact-chip .af-open {
  opacity: 0.5;
  flex-shrink: 0;
}
.artifact-chip:hover .af-open {
  opacity: 0.9;
}

/* ─────────── 输入区域 ─────────── */
.input-area {
  padding: 12px 32px 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  position: relative;
}

/* 输入卡片 —— 仿豆包：明显更宽（约原来的 1.7 倍），输入多了高度自动撑大 */
.input-card {
  width: 100%;
  max-width: 1394px;
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 16px;
  box-shadow: var(--shadow);
  padding: 14px 18px;
}
textarea {
  width: 100%;
  border: none;
  outline: none;
  resize: none;
  font-size: 14.5px;
  background: transparent;
  color: var(--text);
  padding: 4px 2px;
  line-height: 1.75;
  /* 高度随内容自动增长（JS 控制），最多到上限后内部滚动 */
  min-height: 60px;
  max-height: 300px;
  overflow-y: auto;
}

/* 工具栏 */
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-top: 8px;
  padding-top: 8px;
  border-top: 1px solid var(--border-soft);
}
.toolbar-left {
  display: flex;
  align-items: center;
  gap: 6px;
}
.toolbar-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 10px;
  border-radius: 6px;
  font-size: 12px;
  color: var(--text-2);
  border: none;
  background: transparent;
  cursor: pointer;
  position: relative;
}
.toolbar-btn:hover {
  background: var(--bg-soft);
  color: var(--text);
}
.toolbar-btn.active {
  background: var(--primary-soft);
  color: var(--primary);
}
/* Tooltip — 放在按钮下方，避免顶部穿模 */
.btn-tooltip {
  position: absolute;
  top: calc(100% + 6px);
  left: 50%;
  transform: translateX(-50%);
  z-index: 25;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.15s;
}
.toolbar-btn:hover .btn-tooltip {
  opacity: 1;
}
.btn-tooltip-inner {
  background: var(--ink);
  color: #fafaf7;
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 12px;
  white-space: nowrap;
  line-height: 1.5;
}
.btn-tooltip-sub {
  font-size: 11px;
  color: var(--dim);
}



.toolbar-right {
  display: flex;
  align-items: center;
  gap: 6px;
}
.send-btn {
  width: 32px;
  height: 32px;
  background: var(--ink);
  color: #fafaf7;
  border: none;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}
.send-btn:hover {
  background: var(--primary);
}
.send-btn:disabled {
  background: var(--border);
  cursor: not-allowed;
}
.send-btn.stop {
  background: var(--vermilion);
}

/* ─────────── 底部授权栏 ─────────── */
.auth-bar {
  width: 100%;
  max-width: 1394px;
  display: flex;
  justify-content: flex-end;
}
.perm-wrap {
  position: relative;
}
.auth-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 12px;
  color: var(--text-2);
  border: 1px solid var(--border-soft);
  background: transparent;
  cursor: pointer;
}
.auth-btn:hover {
  border-color: var(--border);
  color: var(--text);
}
.auth-btn.deny {
  color: var(--vermilion);
  border-color: rgba(192, 57, 43, 0.2);
}
.auth-hand {
  width: 13px;
  height: 13px;
  object-fit: contain;
}
.auth-deny {
  color: var(--vermilion);
}
.auth-label {
  margin-right: 2px;
}

/* 授权下拉菜单 — 向上展开 */
.dropdown {
  position: absolute;
  right: 0;
  bottom: calc(100% + 6px);
  background: var(--panel);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: var(--shadow-lg);
  width: 280px;
  padding: 6px;
  z-index: 20;
}
.perm-row {
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
}
.perm-row:hover {
  background: var(--bg-soft);
}
.perm-row.active {
  background: var(--primary-soft);
}
.perm-row.deny .title {
  color: var(--vermilion);
}
.perm-row .title {
  font-size: 13px;
  color: var(--text);
  font-weight: 600;
}
.perm-row .desc {
  font-size: 11.5px;
  color: var(--muted);
  margin-top: 2px;
  line-height: 1.5;
}

/* ─────────── 拖拽上传覆盖层 ─────────── */
.drop-overlay {
  position: absolute;
  inset: 10px;
  z-index: 50;
  background: rgba(44, 70, 97, 0.06);
  border: 2px dashed var(--primary);
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(1px);
  pointer-events: none;
}
.drop-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--primary);
}
.drop-title {
  font-family: var(--serif);
  font-size: 16px;
  font-weight: 600;
  letter-spacing: 1px;
}
.drop-sub {
  font-size: 12px;
  color: var(--muted);
}

/* ─────────── 附件 chips ─────────── */
.attach-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
}
.attach-chips.in-bubble {
  margin-top: 8px;
  margin-bottom: 0;
}
.attach-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  max-width: 260px;
  padding: 4px 8px;
  background: var(--bg-soft);
  border: 1px solid var(--border);
  border-radius: 8px;
  font-size: 12px;
  color: var(--text-2);
}
.attach-chip .ac-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
  color: var(--text);
}
.attach-chip .ac-size {
  color: var(--dim);
  font-size: 11px;
  flex-shrink: 0;
}
.attach-chip.readonly {
  background: transparent;
  color: var(--primary-deep);
}
.attach-chip.pending {
  color: var(--muted);
}
.ac-remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border: none;
  background: transparent;
  color: var(--muted);
  border-radius: 4px;
  cursor: pointer;
  flex-shrink: 0;
}
.ac-remove:hover {
  background: var(--border);
  color: var(--text);
}
.spin {
  animation: spin 0.9s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
