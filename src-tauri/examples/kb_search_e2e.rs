//! 端到端验证 KB 检索质量：直接调 polaris_app_lib::kb 的公开 API
//! 用真实志愿问题探查 Top-5 召回是否包含相关实体页。

use polaris_app_lib::kb::{kb_init, kb_search, KB_ROOT};
use std::path::PathBuf;

fn main() {
    let kb = PathBuf::from(r"D:\polaris\高考志愿填报\polaris_gaokao\PolarisKB");
    *KB_ROOT.write() = kb.clone();
    kb_init(Some(kb.join("wiki").to_string_lossy().to_string()));
    println!("KB 已初始化: {:?}\n", KB_ROOT.read().clone());

    // 真实志愿场景的查询 — 覆盖事实题/判断题/祛魅题/院校题
    let queries = &[
        ("会计学",                  "判断/专业内涵"),
        ("临床医学 规培",            "祛魅/后悔坑"),
        ("位次法 冲稳保",            "规则/方法论"),
        ("计算机 就业 35岁",         "判断+祛魅"),
        ("军校 体检 女生",           "升学途径+体检"),
        ("电气工程 电网",            "专业+行业"),
        ("985 211 双一流 区别",      "概念辨析"),
        ("中外合作办学 坑",          "升学途径+祛魅"),
        ("人工智能 专业 选哪个",     "判断/选择"),
        ("数学 师范 考公",           "专业+路径"),
    ];

    let mut total_rel = 0; let mut total_top5 = 0;
    for (q, kind) in queries {
        let hits = kb_search(q.to_string(), Some(5));
        let n = hits.len();
        total_top5 += n;
        // 粗判相关：title 或 snippet 包含核心词
        let qwords: Vec<&str> = q.split_whitespace().collect();
        let rel = hits.iter().filter(|h| {
            let s = format!("{} {}", h.title.to_lowercase(), h.snippet.to_lowercase());
            qwords.iter().any(|w| s.contains(&w.to_lowercase()))
        }).count();
        total_rel += rel;
        let tag = if rel >= 3 { "✅" } else if rel >= 1 { "⚠️" } else { "❌" };
        println!("{tag} [{kind}] 「{q}」 → 召回 {n}/5  相关 {rel}/5");
        for h in hits.iter().take(3) {
            let t = h.title.chars().take(28).collect::<String>();
            let s: String = h.snippet.chars().take(40).collect();
            println!("     · {}  | {}…  score={:.1}", t, s, h.score);
        }
        println!();
    }
    println!("══════════════════════════════════════════════");
    println!("总召回条目: {total_top5}");
    println!("命中核心词: {total_rel}/{total_top5}  ({:.0}%)",
             total_rel as f64 / total_top5.max(1) as f64 * 100.0);
}
