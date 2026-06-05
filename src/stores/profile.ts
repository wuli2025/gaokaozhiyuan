import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { gk } from "../tauri";

/** 考生档案：省份 / 首选(物理|历史) / 再选两科 / 分数 / 位次。
 *  整个填报流的输入源，前端 localStorage 持久化（一年一版，无需后端账号）。 */
export type Track = "物理" | "历史";

const KEY = "polaris.gk.profile.v1";

/** 志向画像维度键 */
export const ASPIRATION_DIMS = [
  { key: "advance", label: "升学 vs 就业", hint: "毕业即就业还是读研读博？" },
  { key: "family", label: "家庭期望", hint: "考公/编制？专项计划？" },
  { key: "idol", label: "偶像 / 梦想", hint: "偶像是谁、梦想职业（接偶像对话）" },
  { key: "salaryCity", label: "薪资 / 城市", hint: "薪资期望、理想城市、能否外省" },
  { key: "subjectAbility", label: "学科能力", hint: "各科强弱、数理底子" },
  { key: "interest", label: "兴趣证据", hint: "做过/查过/能坚持什么" },
  { key: "risk", label: "风险偏好", hint: "愿不愿冲名校被调剂、能否接受冷门" },
  { key: "note", label: "其他备注", hint: "体检/视力色觉、预算、性别(军警)等" },
] as const;

export type AspirationKey = (typeof ASPIRATION_DIMS)[number]["key"];

interface Persisted {
  province: string;
  track: Track;
  reselect: string[];
  score: number | null;
  rank: number | null;
  aspiration?: Record<string, string>;
}

function load(): Persisted | null {
  try {
    const raw = localStorage.getItem(KEY);
    if (raw) return JSON.parse(raw) as Persisted;
  } catch {
    /* ignore */
  }
  return null;
}

export const useProfileStore = defineStore("profile", () => {
  const saved = load();
  const province = ref<string>(saved?.province ?? "");
  const track = ref<Track>(saved?.track ?? "物理");
  const reselect = ref<string[]>(saved?.reselect ?? []);
  const score = ref<number | null>(saved?.score ?? null);
  const rank = ref<number | null>(saved?.rank ?? null);
  const aspiration = ref<Record<string, string>>(saved?.aspiration ?? {});
  const computing = ref(false);
  const error = ref<string>("");

  /** 志向画像已填维度数（用于完成度提示） */
  const aspirationFilled = computed(
    () => Object.values(aspiration.value).filter((v) => v && v.trim()).length
  );

  const ready = computed(
    () => !!province.value && !!track.value && rank.value != null && rank.value > 0
  );

  /** 完整的再选标签（含首选）——传给后端做选科双闸 */
  const subjects = computed(() => [track.value, ...reselect.value]);

  function persist() {
    try {
      localStorage.setItem(
        KEY,
        JSON.stringify({
          province: province.value,
          track: track.value,
          reselect: reselect.value,
          score: score.value,
          rank: rank.value,
          aspiration: aspiration.value,
        } satisfies Persisted)
      );
    } catch {
      /* storage may be unavailable */
    }
  }

  function toggleReselect(s: string) {
    const i = reselect.value.indexOf(s);
    if (i >= 0) reselect.value.splice(i, 1);
    else if (reselect.value.length < 2) reselect.value.push(s);
  }

  /** 由分数换算位次（真实一分一段插值），换算成功则落档 */
  async function computeRank(): Promise<boolean> {
    error.value = "";
    if (!province.value || score.value == null) {
      error.value = "请先选择省份并填写分数";
      return false;
    }
    computing.value = true;
    try {
      const r = await gk.scoreToRank(province.value, track.value, score.value);
      rank.value = r.rank;
      persist();
      return true;
    } catch (e: any) {
      error.value = typeof e === "string" ? e : e?.message ?? "换算失败";
      return false;
    } finally {
      computing.value = false;
    }
  }

  /** 直接录入位次（跳过分数） */
  function setRank(r: number) {
    rank.value = r;
    persist();
  }

  function reset() {
    province.value = "";
    track.value = "物理";
    reselect.value = [];
    score.value = null;
    rank.value = null;
    aspiration.value = {};
    persist();
  }

  /** 设置某个志向维度并持久化 */
  function setAspiration(key: string, value: string) {
    aspiration.value = { ...aspiration.value, [key]: value };
    persist();
  }

  return {
    province,
    track,
    reselect,
    score,
    rank,
    aspiration,
    aspirationFilled,
    computing,
    error,
    ready,
    subjects,
    persist,
    toggleReselect,
    computeRank,
    setRank,
    setAspiration,
    reset,
  };
});
