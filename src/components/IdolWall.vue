<script setup lang="ts">
import { ref, computed, nextTick } from "vue";
import { convApi } from "../tauri";
import { useChatStore } from "../stores/chat";
import { useAppStore } from "../stores/app";
import { useProfileStore } from "../stores/profile";

const chatStore = useChatStore();
const app = useAppStore();
const profile = useProfileStore();

interface Idol {
  key: string;
  name: string;
  avatar: string;
  field: string;
  major: string; // 对应高考专业方向
  persona: string; // 人格设定
}
const IDOLS: Idol[] = [
  { key: "qianxuesen", name: "钱学森", avatar: "钱", field: "航天 · 国防", major: "航空航天 / 力学 / 自动化",
    persona: "中国航天之父，严谨、家国情怀深重，强调系统工程思维与基础理论功底。" },
  { key: "curie", name: "居里夫人", avatar: "居", field: "物理 · 化学", major: "物理学 / 化学 / 材料",
    persona: "两获诺奖的科学家，专注、坚韧、淡泊名利，相信长期主义与对真理的纯粹热爱。" },
  { key: "feynman", name: "费曼", avatar: "费", field: "物理 · 学习", major: "物理学 / 数学 / 基础理科",
    persona: "顽童式物理学家，主张‘费曼学习法’，反对死记硬背，强调把问题想透、保持好奇与诚实。" },
  { key: "jobs", name: "乔布斯", avatar: "乔", field: "设计 · 产品", major: "工业设计 / 计算机 / 人机交互",
    persona: "苹果创始人，极致产品主义、审美洁癖，强调‘求知若饥、虚心若愚’与做减法。" },
  { key: "musk", name: "马斯克", avatar: "马", field: "工程 · 航天", major: "机械 / 航空航天 / 电气 / 能源动力",
    persona: "工程实干家，第一性原理思考，敢冒险、抠成本、重制造，关注用工程解决物理世界的大问题。" },
  { key: "karpathy", name: "Karpathy", avatar: "K", field: "AI · 工程", major: "人工智能 / 计算机 / 自动化",
    persona: "AI 工程师与教育者，务实、爱开源、善于把复杂概念讲清楚，强调动手实现与扎实数学基础。" },
  { key: "zhangyiming", name: "张一鸣", avatar: "张", field: "产品 · 组织", major: "计算机 / 信息管理 / 工商管理",
    persona: "字节跳动创始人，理性克制、延迟满足，重视‘Context not Control’与用数据驱动决策。" },
  { key: "munger", name: "芒格", avatar: "芒", field: "投资 · 思维", major: "金融 / 经济学 / 工商管理",
    persona: "投资大家，多元思维模型、逆向思考，强调避免愚蠢胜过追求聪明、终身学习。" },
  { key: "naval", name: "纳瓦尔", avatar: "纳", field: "财富 · 哲学", major: "计算机 / 经济 / 创业",
    persona: "硅谷投资人与思想家，讲杠杆(代码/资本/媒体)、特定知识与复利，追求财富与内心的自由。" },
];

const selected = ref<Idol>(IDOLS[0]);
const input = ref("");
const convId = ref<string | null>(null);
const starting = ref(false);

const bubbles = computed(() =>
  chatStore.bubblesFor(convId.value).filter((b) => b.role !== "tool")
);
const sending = computed(() => chatStore.isSending(convId.value));

async function ensureConv(): Promise<string | null> {
  if (convId.value) return convId.value;
  starting.value = true;
  try {
    let pid = app.currentProjectId;
    if (!pid) {
      await app.refreshProjects();
      pid = app.currentProjectId;
    }
    if (!pid) {
      const p = await convApi.createProject("偶像对话");
      pid = p.id;
      app.currentProjectId = pid;
    }
    const c = await convApi.createConversation(pid);
    convId.value = c.id;
    return c.id;
  } catch {
    return null;
  } finally {
    starting.value = false;
  }
}

function buildPrompt(userText: string): string {
  const i = selected.value;
  const a = profile.aspiration;
  const asp = [
    a.advance && `升学/就业: ${a.advance}`,
    a.family && `家庭期望: ${a.family}`,
    a.salaryCity && `薪资/城市: ${a.salaryCity}`,
    a.subjectAbility && `学科能力: ${a.subjectAbility}`,
    a.interest && `兴趣证据: ${a.interest}`,
    a.risk && `风险偏好: ${a.risk}`,
  ].filter(Boolean).join("；");
  const prof = profile.ready
    ? `考生画像：${profile.province} · ${profile.track}类${profile.reselect.length ? "+" + profile.reselect.join("") : ""}，估分 ${profile.score ?? "?"} / 位次≈${profile.rank ?? "?"}。${asp ? "志向：" + asp + "。" : ""}`
    : "考生尚未建立完整档案。";
  return [
    `你现在扮演「${i.name}」（${i.field}）。${i.persona}`,
    `以其视角、口吻与价值观，和这位中国高考考生对话，帮他厘清专业与院校方向。`,
    `这是基于公开资料的拟人演绎、非本人观点；只聊专业选择与人生方向，不杜撰史实、不算命。`,
    prof,
    `考生说：${userText}`,
    `请以 ${i.name} 的身份回答（3–6 句，犀利、具体、可落地），结尾用一句话点出对应的高考专业方向（参考：${i.major}）。`,
  ].join("\n");
}

async function send() {
  const text = input.value.trim();
  if (!text || sending.value) return;
  const cid = await ensureConv();
  if (!cid) return;
  input.value = "";
  await chatStore.send(cid, buildPrompt(text), text, undefined, {
    permissionMode: "manual",
    skillIds: [],
  });
  await nextTick();
  scrollBottom();
}

const scroller = ref<HTMLElement | null>(null);
function scrollBottom() {
  const el = scroller.value;
  if (el) el.scrollTop = el.scrollHeight;
}

function pickIdol(i: Idol) {
  selected.value = i;
}

const SUGGEST = [
  "我数理还行，但不知道学机械还是计算机，哪个更适合我？",
  "我想稳，又怕选错专业，你会怎么权衡？",
  "我对这个领域只是觉得酷，算真的喜欢吗？",
];
function useSuggest(s: string) { input.value = s; }
</script>

<template>
  <div class="page">
    <header class="ph">
      <div class="eyebrow">★ 为“想成为谁”而来，被“该学什么”留下</div>
      <h1>偶像对话</h1>
      <p>选一位偶像来对标、与他对话，在交流中厘清你的理想专业与学校。基于公开资料的拟人演绎，非本人观点。</p>
    </header>

    <div class="idolwall">
      <button
        v-for="i in IDOLS"
        :key="i.key"
        class="idol"
        :class="{ on: selected.key === i.key }"
        @click="pickIdol(i)"
      >
        <span class="f">{{ i.avatar }}</span>
        <b>{{ i.name }}</b>
        <span class="fd">{{ i.field }}</span>
      </button>
    </div>

    <div class="chatbox">
      <div class="cb-head">
        <span class="cb-ava">{{ selected.avatar }}</span>
        <div class="cb-titles">
          <b>正在对话：{{ selected.name }}</b>
          <span>{{ selected.field }} · 对应专业方向：{{ selected.major }}</span>
        </div>
      </div>

      <div class="cb-body" ref="scroller">
        <div v-if="!bubbles.length" class="cb-empty">
          <p>和 <b>{{ selected.name }}</b> 聊聊你的纠结吧。试试：</p>
          <button v-for="s in SUGGEST" :key="s" class="sugg" @click="useSuggest(s)">{{ s }}</button>
        </div>
        <template v-else>
          <div
            v-for="(b, idx) in bubbles"
            :key="idx"
            class="bub"
            :class="b.role === 'user' ? 'u' : 'a'"
          >
            <b v-if="b.role !== 'user'" class="who">{{ selected.name }} ·</b>
            <span class="bub-text">{{ b.text }}</span>
          </div>
        </template>
        <div v-if="sending" class="bub a typing"><span class="dot"></span><span class="dot"></span><span class="dot"></span></div>
      </div>

      <div class="cb-input">
        <textarea
          v-model="input"
          rows="1"
          :placeholder="`问问 ${selected.name}…`"
          @keydown.enter.exact.prevent="send"
        ></textarea>
        <button class="send" :disabled="sending || starting || !input.trim()" @click="send">
          {{ starting ? "连接中" : "发送" }}
        </button>
      </div>
      <div class="cb-foot">拟人演绎仅供参考；对话识别到的志向可回写「我的档案」并刷新冲稳保。</div>
    </div>
  </div>
</template>

<style scoped>
.page { height: 100vh; overflow-y: auto; max-width: 860px; margin: 0 auto; padding: 30px 28px 40px; display: flex; flex-direction: column; }
.eyebrow { font-family: var(--mono); font-size: 11px; color: var(--gold-deep); font-weight: 700; letter-spacing: .18em; }
.ph h1 { font-family: var(--serif); font-size: 27px; margin: 6px 0 0; color: var(--ink); }
.ph p { color: var(--text-2); font-size: 13.5px; margin: 8px 0 20px; }

.idolwall { display: grid; grid-template-columns: repeat(5, 1fr); gap: 10px; margin-bottom: 18px; }
.idol { border: 1px solid var(--border); border-radius: 14px; padding: 14px 6px; text-align: center; background: var(--panel); box-shadow: var(--shadow-sm); transition: .18s; }
.idol:hover { transform: translateY(-3px); box-shadow: var(--shadow-lg); }
.idol .f { width: 44px; height: 44px; border-radius: 50%; margin: 0 auto 7px; background: linear-gradient(135deg, #7a5a9a, #3a2440); color: #fff; display: flex; align-items: center; justify-content: center; font-weight: 700; font-family: var(--serif); font-size: 18px; }
.idol.on { border-color: var(--vermilion); box-shadow: 0 0 0 2px rgba(200,55,45,.18); background: var(--vermilion-soft); }
.idol.on .f { background: linear-gradient(135deg, #c8372d, #e0644f); }
.idol b { font-size: 13px; color: var(--ink); display: block; }
.idol .fd { display: block; font-size: 10.5px; color: var(--muted); margin-top: 2px; }

.chatbox { flex: 1; display: flex; flex-direction: column; background: var(--panel); border: 1px solid var(--border); border-radius: 16px; overflow: hidden; box-shadow: var(--shadow); min-height: 380px; }
.cb-head { display: flex; align-items: center; gap: 11px; padding: 13px 18px; border-bottom: 1px solid var(--border); background: var(--bg-soft); }
.cb-ava { width: 38px; height: 38px; border-radius: 50%; background: linear-gradient(135deg, #c8372d, #e0644f); color: #fff; display: flex; align-items: center; justify-content: center; font-weight: 700; font-family: var(--serif); }
.cb-titles b { font-size: 14px; color: var(--ink); display: block; }
.cb-titles span { font-size: 11.5px; color: var(--muted); }

.cb-body { flex: 1; overflow-y: auto; padding: 18px; display: flex; flex-direction: column; gap: 11px; min-height: 200px; max-height: 46vh; }
.cb-empty { color: var(--muted); font-size: 13px; }
.cb-empty p { margin: 0 0 12px; }
.cb-empty b { color: var(--vermilion); }
.sugg { display: block; width: 100%; text-align: left; background: var(--bg-soft); border: 1px solid var(--border-soft); border-radius: 10px; padding: 10px 13px; margin-bottom: 8px; font-size: 12.8px; color: var(--text-2); transition: .14s; }
.sugg:hover { border-color: var(--primary); color: var(--primary); }

.bub { max-width: 82%; padding: 10px 14px; border-radius: 14px; font-size: 13.5px; line-height: 1.7; }
.bub.u { align-self: flex-end; background: linear-gradient(135deg, #c8372d, #a82a23); color: #fff; border-bottom-right-radius: 4px; box-shadow: var(--shadow-sm); }
.bub.a { align-self: flex-start; background: var(--panel); border: 1px solid var(--border); color: var(--ink-2); border-bottom-left-radius: 4px; box-shadow: var(--shadow-sm); white-space: pre-wrap; }
.bub .who { color: var(--red-deep); margin-right: 5px; }
.bub.typing { display: flex; gap: 5px; align-items: center; }
.bub.typing .dot { width: 6px; height: 6px; border-radius: 50%; background: var(--dim); animation: blink 1.2s infinite both; }
.bub.typing .dot:nth-child(2) { animation-delay: .2s; }
.bub.typing .dot:nth-child(3) { animation-delay: .4s; }
@keyframes blink { 0%, 80%, 100% { opacity: .25; } 40% { opacity: 1; } }

.cb-input { display: flex; gap: 10px; padding: 12px 14px; border-top: 1px solid var(--border); }
.cb-input textarea { flex: 1; border: 1px solid var(--border); border-radius: 11px; padding: 10px 13px; font-size: 13.5px; resize: none; outline: none; font-family: var(--sans); line-height: 1.5; max-height: 120px; }
.cb-input textarea:focus { border-color: var(--primary); }
.send { background: var(--grad); color: #fff; border: none; border-radius: 11px; padding: 0 24px; font-weight: 700; font-size: 14px; }
.send:disabled { opacity: .5; }
.cb-foot { padding: 8px 16px 12px; font-size: 11px; color: var(--dim); text-align: center; }

@media (max-width: 720px) { .idolwall { grid-template-columns: repeat(3, 1fr); } }
</style>
