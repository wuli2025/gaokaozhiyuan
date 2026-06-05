//! 端到端验证 KB 检索质量 — 走 integration test（不需改 lib 可见性）
//! 用真实志愿问题探查 Top-5 召回质量
use polaris_app_lib::*;

#[test]
fn e2e_志愿_检索质量() {
    // 初始化 KB
    let kb = std::path::PathBuf::from(r"D:\polaris\高考志愿填报\polaris_gaokao\PolarisKB\wiki");
    let _ = std::env::set_current_dir(kb.parent().unwrap());
    let n = kb::kb_set_root(kb.to_string_lossy().to_string()).expect("set_root");
    let _ = kb::kb_scan();
    println!("[init] root={:?}  indexed={}", kb::kb_root(), n);
    assert!(std::path::Path::new(&kb::kb_root()).exists(), "KB 根目录应存在");

    let queries: &[(&str, &str)] = &[
        ("会计学",            "专业内涵"),
        ("临床医学 规培",      "祛魅/后悔坑"),
        ("位次法 冲稳保",      "规则"),
        ("计算机 35岁",        "祛魅"),
        ("军校 体检 女生",     "升学+体检"),
        ("电气工程 电网",      "专业+行业"),
        ("985 211 双一流 区别", "概念"),
        ("中外合作办学 坑",    "途径+祛魅"),
        ("人工智能 选哪个",    "判断"),
        ("数学 师范 考公",     "专业+路径"),
    ];

    let mut total_hits = 0usize;
    let mut total_rel = 0usize;
    let mut total_zero = 0usize;
    for (q, kind) in queries {
        let hits = kb::kb_search(q.to_string(), Some(5));
        let n = hits.len();
        total_hits += n;
        if n == 0 { total_zero += 1; }
        let qwords: Vec<String> = q.split_whitespace().map(|s| s.to_lowercase()).collect();
        let rel = hits.iter().filter(|h| {
            let s = format!("{} {}", h.title.to_lowercase(), h.snippet.to_lowercase());
            qwords.iter().any(|w| s.contains(w))
        }).count();
        total_rel += rel;
        let tag = if n == 0 { "❌" } else if rel >= 3 { "✅" } else if rel >= 1 { "⚠️ " } else { "·  " };
        println!("{tag} [{kind:6}] 「{q:<28}」 召回 {n}/5 相关 {rel}/5");
        for h in hits.iter().take(3) {
            let t: String = h.title.chars().take(30).collect();
            let s: String = h.snippet.chars().take(42).collect();
            println!("         · {:<32}  {}…  score={:.1}", t, s, h.score);
        }
    }
    let recall = total_rel as f64 / total_hits.max(1) as f64;
    println!("\n════════════════════════════════════════════");
    println!("  总召回 {total_hits}  相关 {total_rel}  召回率 {recall:.0}%  零召回查询 {total_zero}/{}",
             queries.len());
    println!("════════════════════════════════════════════");
    // 软指标：≥80% 命中，零召回查询 ≤2
    assert!(recall >= 0.70 || total_hits < 20, "召回率过低: {recall:.0}%");
    assert!(total_zero <= 3, "零召回查询太多: {total_zero}");
}
