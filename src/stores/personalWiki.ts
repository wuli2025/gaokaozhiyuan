import { defineStore } from "pinia";
import { ref } from "vue";
import {
  kb,
  chat as chatApi,
  listen,
  type PersonalFile,
  type ChatStreamEvent,
} from "../tauri";
import { useProfileStore } from "./profile";

/** 整理出的结构化考试报告（与 personal_report_directive 里的 JSON schema 对应） */
export interface ExamReport {
  headline?: string;
  score_profile?: {
    score?: number | null;
    rank?: number | null;
    province?: string;
    track?: string;
    subjects?: string;
  };
  subjects?: { name: string; level: "强" | "中" | "弱" | string; note?: string }[];
  strengths?: string[];
  risks?: { level: "high" | "mid" | "low" | string; text: string }[];
  directions?: string[];
  gaps?: string[];
  sources?: string[];
}

/** 报告文件在 KB 里的固定相对路径（与后端 personal_report_directive 约定一致） */
const REPORT_REL = "wiki/students/我的档案.md";

/**
 * 个人 wiki（考生个人资料专区）运行时 store。
 *
 * - 资料上传/列表/删除：落进 KB 的「个人档案/」专区（模型对话时按需 Read）。
 * - 一键整理：调模型读个人档案+画像 → 写结构化报告到 wiki/students/，本 store 读回并解析渲染。
 */
export const usePersonalWikiStore = defineStore("personalWiki", () => {
  const materials = ref<PersonalFile[]>([]);
  const loadingList = ref(false);
  const uploading = ref(false);

  const report = ref<ExamReport | null>(null);
  const reportLoaded = ref(false);
  const generating = ref(false);
  /** 生成过程中的最近一条进度文字（模型流式输出 / 工具调用） */
  const progress = ref("");
  const error = ref("");

  async function refreshMaterials() {
    loadingList.value = true;
    try {
      materials.value = await kb.listPersonal();
    } catch (e: any) {
      error.value = typeof e === "string" ? e : e?.message ?? "读取资料失败";
    } finally {
      loadingList.value = false;
    }
  }

  /** 上传一批文件（绝对路径）到个人专区，完成后刷新列表 */
  async function upload(paths: string[]) {
    if (!paths.length) return;
    uploading.value = true;
    error.value = "";
    try {
      const res = await kb.uploadPersonal(paths);
      const failed = res.filter((r) => !r.ok);
      if (failed.length) {
        error.value = `${failed.length} 个文件上传失败：${failed
          .map((f) => f.name)
          .join("、")}`;
      }
      await refreshMaterials();
    } catch (e: any) {
      error.value = typeof e === "string" ? e : e?.message ?? "上传失败";
    } finally {
      uploading.value = false;
    }
  }

  async function remove(relPath: string) {
    try {
      await kb.delete(relPath);
      await refreshMaterials();
    } catch (e: any) {
      error.value = typeof e === "string" ? e : e?.message ?? "删除失败";
    }
  }

  /** 从报告 md 文本里抠出 <!--REPORT_JSON ... --> 机读块并解析 */
  function parseReport(md: string): ExamReport | null {
    const m = md.match(/<!--REPORT_JSON([\s\S]*?)-->/);
    if (!m) return null;
    try {
      return JSON.parse(m[1].trim()) as ExamReport;
    } catch {
      return null;
    }
  }

  /** 读回已生成的报告（应用启动 / 进入档案页时调一次） */
  async function loadReport() {
    try {
      const md = await kb.read(REPORT_REL);
      const parsed = parseReport(md);
      if (parsed) report.value = parsed;
    } catch {
      /* 还没生成过报告，正常 */
    } finally {
      reportLoaded.value = true;
    }
  }

  /**
   * 一键整理：让模型读个人档案+画像，生成结构化报告写入 wiki/students/，
   * 然后读回并解析渲染。返回 Promise，在生成结束后 resolve。
   */
  async function generateReport(): Promise<void> {
    if (generating.value) return;
    const profile = useProfileStore();
    generating.value = true;
    error.value = "";
    progress.value = "正在读取你的个人资料…";

    // 把画像快照一并交给模型（画像存在 localStorage，模型看不到，需随 prompt 传入）
    const snapshot = {
      province: profile.province,
      track: profile.track,
      reselect: profile.reselect,
      score: profile.score,
      rank: profile.rank,
      aspiration: profile.aspiration,
    };
    const prompt =
      "请整理我上传的个人考试资料，生成一份结构化考试报告。\n\n" +
      "我的学生画像（JSON，以此为准）：\n```json\n" +
      JSON.stringify(snapshot, null, 2) +
      "\n```";

    let unlisten: (() => void) | null = null;
    try {
      const reqId = await chatApi.send({
        prompt,
        permissionMode: "auto_current", // 需要读个人档案 + 写报告文件
        genReport: true,
        // 不挂 conversationId：不污染对话列表，作为档案页的后台整理任务
      });

      // 先把流监听挂好（同步 await，避免漏掉先到的事件），再等它发 done
      let resolveDone!: () => void;
      const done = new Promise<void>((r) => {
        resolveDone = r;
      });
      unlisten = await listen<ChatStreamEvent>("chat:stream", (ev) => {
        if (ev.reqId !== reqId) return;
        if (ev.kind === "delta" && ev.text) {
          progress.value = ev.text.slice(-60);
        } else if (ev.kind === "tool") {
          progress.value = `整理中：${ev.tool ?? ""}`;
        } else if (ev.kind === "error") {
          error.value = ev.text ?? "整理出错";
        } else if (ev.kind === "done") {
          resolveDone();
        }
      });
      await done;

      progress.value = "正在读取报告…";
      // 模型经 CLI 直接写了 wiki/students/ 报告，重扫索引让它进入 KB，
      // 之后每次对话都会被全文自动注入（“对话时自动采集个人 wiki”）。
      try {
        await kb.scan();
      } catch {
        /* 重扫失败不影响本次报告读取 */
      }
      await loadReport();
      if (!report.value) {
        error.value = "报告已生成，但未能解析出结构化内容，请重试或检查资料。";
      }
    } catch (e: any) {
      error.value = typeof e === "string" ? e : e?.message ?? "整理失败";
    } finally {
      if (unlisten) unlisten();
      generating.value = false;
      progress.value = "";
    }
  }

  return {
    materials,
    loadingList,
    uploading,
    report,
    reportLoaded,
    generating,
    progress,
    error,
    refreshMaterials,
    upload,
    remove,
    loadReport,
    generateReport,
  };
});
