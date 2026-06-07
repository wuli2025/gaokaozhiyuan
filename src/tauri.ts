/**
 * Typed wrappers around Tauri commands.
 *
 * Designed so the renderer can still mount in a plain browser (npm run dev) by
 * detecting absence of __TAURI_INTERNALS__ and returning empty / stub data.
 */
import { invoke as rawInvoke } from "@tauri-apps/api/core";
import { listen as rawListen, type UnlistenFn } from "@tauri-apps/api/event";

export const isTauri =
  typeof window !== "undefined" &&
  // @ts-ignore tauri injects this
  typeof (window as any).__TAURI_INTERNALS__ !== "undefined";

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri) {
    // Browser-only stubs so the UI is still navigable during pure-web dev.
    return browserStub(cmd, args) as T;
  }
  return rawInvoke<T>(cmd, args);
}

export async function listen<T>(
  event: string,
  cb: (payload: T) => void
): Promise<UnlistenFn> {
  if (!isTauri) return () => {};
  return rawListen<T>(event, (e) => cb(e.payload));
}

// ──────────────────────────────────────────────────────────────
// KB module
// ──────────────────────────────────────────────────────────────
export interface KbHit {
  path: string;
  title: string;
  snippet: string;
  score: number;
}
export interface KbNode {
  id: string;
  title: string;
  category: string;
  /** "doc" 文档 | "folder" 目录中枢 | "root" 知识库根 */
  kind: "doc" | "folder" | "root";
}
export interface KbEdge {
  source: string;
  target: string;
}
export interface KbGraph {
  nodes: KbNode[];
  edges: KbEdge[];
}
/** 知识库拖拽上传的逐文件结果 */
export interface KbUploadResult {
  name: string;
  relPath: string;
  ok: boolean;
  message: string;
}
/** 个人资料专区里的一份文件（档案页清单用） */
export interface PersonalFile {
  name: string;
  relPath: string;
  size: number;
  /** 修改时间 Unix 秒 */
  modified: number;
}

export const kb = {
  scan: () => invoke<number>("kb_scan"),
  search: (q: string, topK = 8) =>
    invoke<KbHit[]>("kb_search", { query: q, topK }),
  list: (subdir: string | null = null) =>
    invoke<string[]>("kb_list", { subdir }),
  read: (relPath: string) => invoke<string>("kb_read", { relPath }),
  /** 删除一份资料(浏览页 ×)，返回剩余文件数 */
  delete: (relPath: string) => invoke<number>("kb_delete", { relPath }),
  /** 清空资料库(管理页)，返回剩余文件数 */
  clear: () => invoke<number>("kb_clear"),
  ingest: (sourcePath: string) =>
    invoke<string>("kb_ingest", { sourcePath }),
  /** 拖拽上传：任意格式 → 转 markdown 入 raw/，返回逐文件结果 */
  uploadFiles: (paths: string[]) =>
    invoke<KbUploadResult[]>("kb_upload_files", { paths }),
  /** 档案页上传：任意格式 → 转 markdown 入「个人档案/」专区，返回逐文件结果 */
  uploadPersonal: (paths: string[]) =>
    invoke<KbUploadResult[]>("kb_upload_personal", { paths }),
  /** 列出「个人档案/」专区里的文件（按修改时间倒序） */
  listPersonal: () => invoke<PersonalFile[]>("kb_list_personal"),
  graph: () => invoke<KbGraph>("kb_graph"),
  root: () => invoke<string>("kb_root"),
  defaultRoot: () => invoke<string>("kb_default_root"),
  setRoot: (newPath: string) =>
    invoke<number>("kb_set_root", { newPath }),
};

// ──────────────────────────────────────────────────────────────
// GK 高考填报核心引擎 (优志愿式流) — 真实数据
// ──────────────────────────────────────────────────────────────
export interface GkProvince {
  province: string;
  rows: number;
}
export interface GkCandidate {
  school: string;
  city: string;
  region: string;
  level: string; // 985 | 211 | 双一流 | 普通本科
  school_type: string;
  major: string;
  subject_group: string;
  min_rank: number;
  min_score: number | null;
  group_code: string | null;
  tier: "冲" | "稳" | "保";
  prob: number; // 0-1
  is985: boolean;
  is211: boolean;
  double_first: boolean;
  rank_delta: number; // 学生位次 - 该专业最低位次（正=有优势）
}
export interface GkFacetItem {
  key: string;
  count: number;
}
export interface GkMatchResult {
  rank: number;
  province: string;
  track: string;
  stats: {
    total: number;
    charge: number;
    steady: number;
    safe: number;
    c985: number;
    c211: number;
    double_first: number;
  };
  facets: {
    region: GkFacetItem[];
    level: GkFacetItem[];
    type: GkFacetItem[];
  };
  page: number;
  page_size: number;
  rows: GkCandidate[];
}
export interface GkMatchArgs {
  province: string;
  track: string; // 物理 | 历史
  subjects?: string[]; // 再选科目
  rank?: number;
  score?: number;
  regions?: string[];
  levels?: string[];
  types?: string[];
  tiers?: string[];
  keyword?: string;
  sort?: "prob" | "rank";
  page?: number;
  page_size?: number;
}
export interface GkSchoolDetail {
  info: {
    name: string;
    city?: string | null;
    is985?: boolean;
    is211?: boolean;
    double_first?: string | null;
    is_c9?: boolean;
    is_central?: boolean;
    school_type?: string | null;
    dept?: string | null;
  };
  majors: Array<{
    major: string;
    subject_group: string | null;
    min_rank: number | null;
    min_score: number | null;
    group_code: string | null;
    year: number;
  }>;
  years: number[];
}

export interface GkGroupMajors {
  majors: Array<{ major: string; plan_count: number | null }>;
  plan_year: number | null;
}

export const gk = {
  provinces: () => invoke<GkProvince[]>("gk_provinces"),
  scoreToRank: (province: string, track: string, score: number) =>
    invoke<{ rank: number; score: number; province: string; track: string }>(
      "gk_score_to_rank",
      { args: { province, track, score } }
    ),
  match: (args: GkMatchArgs) =>
    invoke<GkMatchResult>("gk_match", { args: args as unknown as Record<string, unknown> }),
  schoolDetail: (name: string, province: string, track: string) =>
    invoke<GkSchoolDetail>("gk_school_detail", { args: { name, province, track } }),
  groupMajors: (name: string, group_code: string, year: number) =>
    invoke<GkGroupMajors>("gk_group_majors", { args: { name, group_code, year } }),
  lookupMajor: (keyword: string) =>
    invoke<Array<{ major_code: string; name: string; category: string; subcategory: string; degree: string }>>(
      "sql_tool_lookup_major",
      { args: { keyword, code: "" } }
    ),
};

// ──────────────────────────────────────────────────────────────
// Sandbox module → 已迁出至 features/sandbox/api.ts (架构重构 Phase 1)
// 浏览器降级 stub 仍保留在本文件下方的 browserStub() 中。
// ──────────────────────────────────────────────────────────────

// ──────────────────────────────────────────────────────────────
// Chat module
// ──────────────────────────────────────────────────────────────
export type PermissionMode =
  | "manual"
  | "auto_current"
  | "auto_all"
  | "deny";

export interface ChatSendArgs {
  prompt: string;
  permissionMode: PermissionMode;
  useSandbox?: boolean;
  skillIds?: string[];
  conversationId?: string;
  /** 目标模式：完成条件。设置后 Claude 会持续推进直到达成，不中途收尾。 */
  goal?: string;
  /** 「请教毛主席」：注入毛选式客观分析指令，调用毛主席资料库，生成标注来源的 HTML。 */
  consultMao?: boolean;
  /** 「一键整理个人 wiki」：读个人档案+画像，生成结构化考试报告写入 wiki/students/。 */
  genReport?: boolean;
  /** 学生画像快照（province/track/subjects/score/rank/aspiration）。后端据此跑智能填报锁池注入。 */
  profile?: Record<string, unknown>;
}

export interface ChatStreamEvent {
  reqId: string;
  kind: "delta" | "tool" | "error" | "done" | "artifact";
  text?: string;
  tool?: string;
  conversationId?: string;
}

/** 对话拖拽上传的附件（复制进会话 uploads 目录） */
export interface AttachedFile {
  name: string;
  /** uploads 目录里的绝对路径（正斜杠） */
  path: string;
  /** text | image | pdf | office | binary */
  kind: "text" | "image" | "pdf" | "office" | "binary";
  size: number;
  ok: boolean;
  error?: string;
}

export const chat = {
  send: (args: ChatSendArgs) =>
    invoke<string>("chat_send", { args: args as unknown as Record<string, unknown> }),
  cancel: (reqId: string) => invoke<void>("chat_cancel", { reqId }),
  /** 拖拽上传：把文件复制进当前会话，返回附件清单 */
  attachFiles: (conversationId: string | undefined, paths: string[]) =>
    invoke<AttachedFile[]>("chat_attach_files", {
      conversationId: conversationId ?? null,
      paths,
    }),
};

// ──────────────────────────────────────────────────────────────
// Artifacts module — 对话生成的成品文件，右侧抽屉预览
// ──────────────────────────────────────────────────────────────
export type ArtifactKind =
  | "html"
  | "svg"
  | "image"
  | "markdown"
  | "text"
  | "binary";

export interface ArtifactPayload {
  path: string;
  name: string;
  ext: string;
  kind: ArtifactKind;
  /** 文本类(html/svg/markdown/text)内容 */
  text?: string;
  /** 图片类的 data URL */
  dataUrl?: string;
  size: number;
}

/** 「参考资料」文件夹视图的一条文件记录 */
export interface ArtifactEntry {
  path: string;
  name: string;
  ext: string;
  kind: ArtifactKind;
  size: number;
  /** 修改时间 Unix 秒 */
  modified: number;
}

export const artifacts = {
  read: (path: string) => invoke<ArtifactPayload>("artifact_read", { path }),
  openExternal: (path: string) =>
    invoke<void>("artifact_open_external", { path }),
  /** 在系统文件管理器中定位并选中该文件（资源管理器 / 访达） */
  reveal: (path: string) => invoke<void>("artifact_reveal", { path }),
  /** 列出某会话产物文件，按修改时间倒序 */
  list: (conversationId?: string) =>
    invoke<ArtifactEntry[]>("artifact_list", {
      conversationId: conversationId ?? null,
    }),
  /** 跨所有对话检索历史产物文件（文件名 + 正文） */
  search: (query: string) =>
    invoke<ArtifactSearchHit[]>("artifact_search", { query }),
};

/** 跨对话产物搜索命中 */
export interface ArtifactSearchHit {
  path: string;
  name: string;
  kind: ArtifactKind;
  conversationId: string;
  snippet: string;
  modified: number;
  score: number;
}

// ──────────────────────────────────────────────────────────────
// CLAUDE.md 主上下文 module
// 每个 conv 项目一份 + KB 共享一份
// ──────────────────────────────────────────────────────────────
export interface ProjectClaudeMd {
  projectId: string;
  projectName: string;
  absPath: string;
  exists: boolean;
  active: boolean;
  size: number;
}

export interface KbClaudeMd {
  absPath: string;
  exists: boolean;
  active: boolean;
  size: number;
}

export type ClaudeMdArea = "kb" | "project";

export const claudeMd = {
  listProjects: () => invoke<ProjectClaudeMd[]>("claude_md_list_projects"),
  kbInfo: () => invoke<KbClaudeMd>("claude_md_kb_info"),
  read: (area: ClaudeMdArea, projectId?: string) =>
    invoke<string>("claude_md_read", { area, projectId: projectId ?? null }),
  write: (area: ClaudeMdArea, projectId: string | undefined, content: string) =>
    invoke<void>("claude_md_write", {
      area,
      projectId: projectId ?? null,
      content,
    }),
};

// ──────────────────────────────────────────────────────────────
// Conv module (项目 + 对话历史)
// ──────────────────────────────────────────────────────────────
export interface Project {
  id: string;
  name: string;
  createdAt: number;
  archived: boolean;
}

export interface Conversation {
  id: string;
  projectId: string;
  title: string;
  createdAt: number;
  updatedAt: number;
}

export interface Message {
  id: string;
  conversationId: string;
  role: "user" | "assistant" | "tool";
  content: string;
  createdAt: number;
}

// Rust 端用 snake_case, serde 默认行为, 这里手动映射回 camelCase
type RawProject = {
  id: string;
  name: string;
  created_at: number;
  archived: boolean;
};
type RawConv = {
  id: string;
  project_id: string;
  title: string;
  created_at: number;
  updated_at: number;
};
type RawMsg = {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  created_at: number;
};

const p = (r: RawProject): Project => ({
  id: r.id,
  name: r.name,
  createdAt: r.created_at,
  archived: r.archived,
});
const c = (r: RawConv): Conversation => ({
  id: r.id,
  projectId: r.project_id,
  title: r.title,
  createdAt: r.created_at,
  updatedAt: r.updated_at,
});
const m = (r: RawMsg): Message => ({
  id: r.id,
  conversationId: r.conversation_id,
  role: r.role as Message["role"],
  content: r.content,
  createdAt: r.created_at,
});

export const convApi = {
  listProjects: async () => (await invoke<RawProject[]>("conv_list_projects")).map(p),
  createProject: async (name: string) =>
    p(await invoke<RawProject>("conv_create_project", { name })),
  archiveProject: (projectId: string) =>
    invoke<void>("conv_archive_project", { projectId }),
  openProjectDir: (projectId: string) =>
    invoke<void>("conv_open_project_dir", { projectId }),
  listConversations: async (projectId: string) =>
    (await invoke<RawConv[]>("conv_list_conversations", { projectId })).map(c),
  createConversation: async (projectId: string) =>
    c(await invoke<RawConv>("conv_create_conversation", { projectId })),
  deleteConversation: (conversationId: string) =>
    invoke<void>("conv_delete_conversation", { conversationId }),
  renameConversation: (conversationId: string, title: string) =>
    invoke<void>("conv_rename_conversation", { conversationId, title }),
  getMessages: async (conversationId: string) =>
    (await invoke<RawMsg[]>("conv_get_messages", { conversationId })).map(m),
};

// ──────────────────────────────────────────────────────────────
// API 供应商坞 + 用量看板 module
// ──────────────────────────────────────────────────────────────
export interface ProviderView {
  id: string;
  name: string;
  note: string;
  baseUrl: string;
  tokenField: string;
  category: string; // official | cn_official | aggregator | third_party | cloud_provider | custom
  websiteUrl: string;
  color: string;
  kind: string; // official | key | codex | copilot | custom
  isPreset: boolean;
  hasKey: boolean;
  authToken: string;
  /** 完整 settings_config（env + includeCoAuthoredBy/attribution 等） */
  settingsConfig: any;
}
export interface ProviderListResult {
  providers: ProviderView[];
  currentId: string;
}
export interface ProviderSaveInput {
  id?: string;
  name: string;
  note?: string;
  websiteUrl?: string;
  tokenField?: string;
  /** 完整 settings_config（env 含 base_url + token + 开关） */
  settingsConfig: any;
}
export interface TokenBucket {
  input: number;
  output: number;
  cacheRead: number;
  cacheCreation: number;
  total: number;
  requests: number;
  cost: number;
}
export interface DailyUsage {
  date: string;
  label: string;
  total: number;
  cost: number;
}
export interface UsageSummary {
  available: boolean;
  today: TokenBucket;
  week: TokenBucket;
  month: TokenBucket;
  year: TokenBucket;
  daily: DailyUsage[];
}
export interface CodexStatus {
  installed: boolean;
  loggedIn: boolean;
  authPath: string;
}

export const provider = {
  list: () => invoke<ProviderListResult>("provider_list"),
  switch: (id: string) => invoke<string>("provider_switch", { id }),
  save: (input: ProviderSaveInput) =>
    invoke<string>("provider_save", { input }),
  delete: (id: string) => invoke<void>("provider_delete", { id }),
  usage: () => invoke<UsageSummary>("usage_summary"),
  codexStatus: () => invoke<CodexStatus>("codex_status"),
  codexLogin: () => invoke<void>("codex_login"),
};

// ──────────────────────────────────────────────────────────────
// Browser stubs (when running in plain `npm run dev` without Tauri)
// ──────────────────────────────────────────────────────────────
function browserStub(cmd: string, _args?: Record<string, unknown>): unknown {
  switch (cmd) {
    case "kb_scan":
      return 0;
    case "kb_search":
      return [];
    case "kb_list":
      return [];
    case "kb_read":
      return "_(browser stub)_  本文件需要 Tauri 后端读取。";
    case "kb_delete":
      return 0;
    case "kb_clear":
      return 0;
    case "kb_ingest":
      return "browser-stub";
    case "kb_upload_files": {
      const paths = (_args?.paths as string[]) ?? [];
      return paths.map((p) => ({
        name: p.split(/[\\/]/).pop() || p,
        relPath: `raw/${p.split(/[\\/]/).pop() || p}`,
        ok: true,
        message: "(browser stub)",
      }));
    }
    case "kb_upload_personal": {
      const paths = (_args?.paths as string[]) ?? [];
      return paths.map((p) => ({
        name: p.split(/[\\/]/).pop() || p,
        relPath: `个人档案/${p.split(/[\\/]/).pop() || p}`,
        ok: true,
        message: "(browser stub)",
      }));
    }
    case "kb_list_personal":
      return [];
    case "chat_attach_files": {
      const paths = (_args?.paths as string[]) ?? [];
      return paths.map((p) => ({
        name: p.split(/[\\/]/).pop() || p,
        path: p,
        kind: "binary",
        size: 0,
        ok: true,
      }));
    }
    case "gk_provinces":
      return ["湖北", "江苏", "广东", "浙江", "山东", "四川", "贵州", "辽宁", "福建", "湖南"].map(
        (province, i) => ({ province, rows: 2600 - i * 180 })
      );
    case "gk_score_to_rank": {
      const a = (_args?.args as any) || {};
      const score = a.score ?? 530;
      return { rank: Math.max(1, Math.round((750 - score) * 320)), score, province: a.province, track: a.track };
    }
    case "gk_match": {
      const a = (_args?.args as any) || {};
      const rank = a.rank ?? 38000;
      const regions = ["北京", "上海", "江苏", "广东", "湖北", "四川", "陕西", "浙江"];
      const levels = ["985", "211", "双一流", "普通本科"];
      const types = ["综合", "理工", "师范", "医药", "财经", "农林"];
      const majors = ["计算机科学与技术", "电子信息工程", "临床医学", "法学", "金融学", "机械工程", "汉语言文学", "电气工程及其自动化"];
      const schools = ["东南大学", "武汉大学", "华中科技大学", "西安交通大学", "四川大学", "中山大学", "苏州大学", "南京理工大学", "深圳大学", "湖南大学"];
      const rows: GkCandidate[] = Array.from({ length: 40 }, (_, i) => {
        const delta = Math.round((i - 12) * rank * 0.04);
        const min_rank = Math.max(800, rank - delta);
        const prob = 1 / (1 + Math.exp(-((min_rank - rank) / rank) / 0.16));
        const tier = prob >= 0.78 ? "保" : prob >= 0.45 ? "稳" : "冲";
        const is985 = i % 9 === 0, is211 = i % 5 === 0;
        return {
          school: schools[i % schools.length], city: "—", region: regions[i % regions.length],
          level: is985 ? "985" : is211 ? "211" : levels[i % levels.length], school_type: types[i % types.length],
          major: majors[i % majors.length], subject_group: a.track === "历史" ? "首选历史，再选不限" : "首选物理，再选化学",
          min_rank, min_score: 600 - i, group_code: null, tier: tier as any, prob: +prob.toFixed(2),
          is985, is211, double_first: i % 3 === 0, rank_delta: rank - min_rank,
        };
      });
      const charge = rows.filter((r) => r.tier === "冲").length;
      const steady = rows.filter((r) => r.tier === "稳").length;
      const safe = rows.filter((r) => r.tier === "保").length;
      return {
        rank, province: a.province ?? "湖北", track: a.track ?? "物理",
        stats: { total: 609, charge: 235, steady: 69, safe: 305, c985: 1, c211: 10, double_first: 15 },
        facets: {
          region: regions.map((key, i) => ({ key, count: 80 - i * 7 })),
          level: levels.map((key, i) => ({ key, count: [12, 41, 56, 500][i] })),
          type: types.map((key, i) => ({ key, count: 120 - i * 15 })),
        },
        page: 0, page_size: 40, rows,
      };
    }
    case "sql_tool_lookup_major":
      return ["计算机科学与技术", "软件工程", "临床医学", "法学", "金融学", "电气工程及其自动化"].map((name, i) => ({
        major_code: "0809" + (10 + i), name, category: ["工学", "工学", "医学", "法学", "经济学", "工学"][i],
        subcategory: name.slice(0, 2) + "类", degree: ["工学", "工学", "医学", "法学", "经济学", "工学"][i],
      }));
    case "gk_school_detail": {
      const a = (_args?.args as any) || {};
      return {
        info: { name: a.name ?? "同济大学", city: "杨浦区", is985: true, is211: true, double_first: "双一流", is_c9: false, is_central: true, school_type: "综合", dept: "教育部" },
        years: [2024, 2023, 2022],
        majors: [2024, 2023, 2022].flatMap((yr, yi) =>
          Array.from({ length: 6 }, (_, i) => ({
            major: ["计算机类", "土木类", "建筑学", "临床医学", "经济管理试验班", "数学类"][i],
            subject_group: a.track === "历史" ? "首选历史，再选不限" : "首选物理，再选化学",
            min_rank: 5000 + i * 600 + yi * 400, min_score: 640 - i * 3 - yi * 2, group_code: null, year: yr,
          }))
        ),
      };
    }
    case "gk_group_majors": {
      const a = (_args?.args as any) || {};
      const pool = ["电子信息工程技术", "云计算技术应用", "人工智能技术应用", "大数据技术", "信息安全技术应用", "工业互联网技术"];
      return {
        plan_year: (a.year ?? 2025) >= 2025 ? 2024 : a.year ?? 2024,
        majors: pool.slice(0, 3 + ((Number(a.group_code) || 0) % 4)).map((m, i) => ({ major: m, plan_count: 30 - i * 4 })),
      };
    }
    case "kb_graph":
      return { nodes: [], edges: [] };
    case "kb_root":
      return "(browser-only, no fs access)";
    case "kb_default_root":
      return "(browser-only)";
    case "kb_set_root":
      return 0;
    case "sandbox_status":
      return {
        docker_installed: false,
        docker_running: false,
        image_built: false,
        image_name: "polaris-sandbox:alpine",
        container_running: false,
        container_name: "polaris-sandbox",
        notes: ["浏览器模式 - 仅 UI 预览,无 Docker 能力"],
      };
    case "sandbox_build_image":
    case "sandbox_start":
    case "sandbox_stop":
    case "sandbox_exec":
      return "(browser stub)";
    case "cube_config_get":
      return { backend: "docker", endpoint: "", apiKey: "" };
    case "cube_config_set":
      return (_args?.config as unknown) ?? { backend: "docker", endpoint: "", apiKey: "" };
    case "cube_status":
      return {
        backend: "docker",
        endpoint: "",
        configured: false,
        reachable: false,
        note: "浏览器模式 - 无后端探测",
      };
    case "chat_send":
      return "stub-req-id";
    case "artifact_read": {
      const path = String(_args?.path ?? "demo.html");
      return {
        path,
        name: path.split("/").pop() || path,
        ext: "html",
        kind: "html",
        text:
          "<!doctype html><html><body style='font-family:sans-serif;padding:40px;text-align:center'><h1>预览占位</h1><p>浏览器模式无后端，无法读取真实文件。</p></body></html>",
        size: 0,
      };
    }
    case "artifact_open_external":
      return undefined;
    case "artifact_list":
      return [];
    case "artifact_search":
      return [];
    case "list_skills":
      return [
        { id: "deep-research", name: "深度搜索", description: "使用 LLM 大规模联网搜索相关内容，自动检索、汇总、交叉验证多来源信息", source: "third-party", installed: true, removable: false },
        { id: "skill-creator", name: "Skill 创建向导", description: "引导用户创建自定义 Skill，自动生成模板和配置文件", source: "official", installed: true, removable: false },
        { id: "pdf", name: "PDF 文档处理", description: "提取 / 生成 / 编辑 PDF：抽取文本表格、合并拆分、Markdown 转 PDF、表单与 OCR", source: "official", installed: false, removable: false },
        { id: "xlsx", name: "Excel 表格", description: "读取分析与生成 Excel：透视统计、公式、图表、多 sheet 报表", source: "official", installed: false, removable: false },
        { id: "pptx", name: "PPT 演示文稿", description: "把 PDF / 文档 / 数据转成有高级感的 PPT：母版配色、版式层级、图表，python-pptx 生成", source: "official", installed: false, removable: false },
        { id: "edge-tts", name: "语音合成 Edge-TTS", description: "把文本转成自然语音音频，多语言多音色，免费无需 key", source: "third-party", installed: false, removable: false },
        { id: "hyperframes", name: "视频动画 Hyperframes", description: "用逐帧 / 分镜方式生成短视频与动画，ffmpeg 合成，可配 Edge-TTS 旁白", source: "third-party", installed: false, removable: false },
        { id: "web-search", name: "联网搜索", description: "实时联网检索，基于 Tavily / Brave 等真实来源回答并交叉验证", source: "third-party", installed: false, removable: false },
        { id: "image-gen", name: "AI 生图 gpt-image-2", description: "用 OpenAI gpt-image-2 模型按描述生成图片，自动扩写提示词，支持多候选与改图", source: "third-party", installed: false, removable: false },
        { id: "cloak-browser", name: "CloakBrowser 浏览器", description: "Agent 默认浏览器：源码级隐身 Chromium，drop-in 替换 Playwright，过 Cloudflare / 反爬。可随时关闭移除", source: "third-party", installed: true, removable: false },
      ];
    case "get_skill":
      return { id: "deep-research", name: "深度搜索", description: "使用 LLM 大规模联网搜索相关内容", source: "third-party", installed: true, removable: false };
    case "import_skill":
      return ["browser-stub-skill"];
    case "create_skill":
    case "install_skill":
    case "delete_skill":
      return undefined;
    case "conv_list_projects":
      return [
        {
          id: "p-stub",
          name: "(浏览器) 示例项目",
          created_at: 0,
          archived: false,
        },
      ];
    case "conv_create_project":
      return {
        id: "p-stub-new",
        name: (_args?.name as string) || "新项目",
        created_at: 0,
        archived: false,
      };
    case "conv_list_conversations":
      return [];
    case "conv_create_conversation":
      return {
        id: "c-stub-new",
        project_id: _args?.projectId as string,
        title: "新对话",
        created_at: 0,
        updated_at: 0,
      };
    case "conv_get_messages":
      return [];
    case "conv_archive_project":
    case "conv_open_project_dir":
    case "conv_delete_conversation":
    case "conv_rename_conversation":
      return undefined;
    case "claude_md_list_projects":
      return [];
    case "claude_md_kb_info":
      return {
        absPath: "(browser-only)",
        exists: false,
        active: false,
        size: 0,
      };
    case "claude_md_read":
      return "_(browser stub)_  本文件需要 Tauri 后端读取。";
    case "claude_md_write":
      return undefined;
    case "provider_list": {
      const mk = (id: string, name: string, baseUrl: string, category: string, color: string, kind: string, hasKey: boolean, authToken = "") => ({
        id, name, note: "", baseUrl, tokenField: "ANTHROPIC_AUTH_TOKEN", category, websiteUrl: baseUrl, color, kind, isPreset: true, hasKey, authToken,
        settingsConfig: { env: baseUrl ? { ANTHROPIC_BASE_URL: baseUrl, ...(authToken ? { ANTHROPIC_AUTH_TOKEN: authToken } : {}) } : {} },
      });
      return {
        providers: [
          mk("claude-official", "Claude 官方", "", "official", "#D97757", "official", true),
          mk("zhipu-glm", "智谱 GLM", "https://open.bigmodel.cn/api/anthropic", "cn_official", "#2c6fff", "key", false),
          mk("kimi", "Kimi 月之暗面", "https://api.moonshot.cn/anthropic", "cn_official", "#2c6fff", "key", true, "sk-demo"),
          mk("deepseek", "DeepSeek 深度求索", "https://api.deepseek.com/anthropic", "cn_official", "#2c6fff", "key", false),
          mk("openrouter", "OpenRouter", "https://openrouter.ai/api", "aggregator", "#7c5cff", "key", false),
          mk("aihubmix", "AiHubMix", "https://aihubmix.com", "aggregator", "#7c5cff", "key", false),
          mk("packycode", "PackyCode", "https://www.packyapi.com", "third_party", "#e8833a", "key", false),
          mk("github-copilot", "GitHub Copilot", "https://api.githubcopilot.com", "third_party", "#e8833a", "copilot", false),
          mk("codex", "Codex (ChatGPT)", "https://chatgpt.com/backend-api/codex", "third_party", "#e8833a", "codex", false),
        ],
        currentId: "kimi",
      };
    }
    case "provider_switch":
      return String(_args?.id ?? "claude-official");
    case "provider_save":
      return "custom-stub";
    case "provider_delete":
      return undefined;
    case "codex_status":
      return { installed: false, loggedIn: false, authPath: "(browser-only)" };
    case "codex_login":
      return undefined;
    case "env_check": {
      const tool = (key: string, name: string, found: boolean, required = false) => ({
        key, name, found,
        version: found ? "(browser stub) v0.0.0" : null,
        path: found ? `/usr/local/bin/${key}` : null,
        onPath: found, required,
        hint: found ? "(browser stub) 已安装" : "未安装 —— 浏览器预览无法真实检测",
      });
      return {
        os: "browser",
        claude: tool("claude", "Claude Code", false, true),
        pwsh: tool("pwsh", "PowerShell 7", false),
        node: tool("node", "Node.js", true),
        npm: tool("npm", "npm", true),
        claudeDir: null,
        claudeDirOnUserPath: true,
        shellReady: false,
        ready: false,
      };
    }
    case "env_fix_path":
      return {
        ok: false,
        dir: null,
        status: "skipped",
        message: "浏览器预览模式无法修改环境变量。",
      };
    case "env_install_claude":
    case "env_install_node":
    case "env_install_pwsh":
    case "env_update_claude":
      return "env-stub-req";
    case "env_claude_update_check":
      return {
        installed: true,
        current: "1.0.0",
        latest: "1.0.1",
        updateAvailable: true,
        checked: true,
        message: "(browser stub) 发现新版本 1.0.1 (当前 1.0.0)。",
      };
    case "env_cancel":
      return undefined;
    case "usage_summary": {
      const daily = Array.from({ length: 14 }, (_, i) => {
        const d = new Date(Date.now() - (13 - i) * 86400000);
        const label = `${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
        return { date: label, label, total: Math.round(300000 + Math.random() * 1600000), cost: +(Math.random() * 6).toFixed(4) };
      });
      return {
        available: true,
        today: { input: 75600, output: 644800, cacheRead: 45506800, cacheCreation: 1637200, total: 720483 + 47144001, requests: 411, cost: 49.107 },
        week: { input: 280000, output: 64000, cacheRead: 6100000, cacheCreation: 410000, total: 6854000, requests: 248, cost: 112.4 },
        month: { input: 980000, output: 240000, cacheRead: 22000000, cacheCreation: 1400000, total: 24620000, requests: 940, cost: 421.8 },
        year: { input: 1900000, output: 520000, cacheRead: 44000000, cacheCreation: 2800000, total: 49220000, requests: 1894, cost: 980.5 },
        daily,
      };
    }
    default:
      return null;
  }
}
