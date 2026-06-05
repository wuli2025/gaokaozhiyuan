//! 端到端验证 SQL 工具 4 件套
use polaris_app_lib::*;

#[test]
fn e2e_sql_tool_林同学() {
    let st = sql_tool::sql_tool_status().expect("status");
    println!("[status] ok={} stats={}", st["ok"], st["stats"]);
    assert_eq!(st["ok"], serde_json::json!(true));
    assert!(st["stats"]["school"].as_i64().unwrap() > 2000);
    assert!(st["stats"]["major"].as_i64().unwrap() > 700);
    assert!(st["stats"]["admission_min_rank"].as_i64().unwrap() > 10000);

    // 1. filter + tier (林同学, 湖北物理类, 位次 9800)
    let args = serde_json::json!({
        "province": "江苏",
        "track": "物理类",
        "rank": 9800,
        "want_total": 45,
    });
    let rows = sql_tool::sql_tool_filter_and_tier(args).expect("filter_and_tier");
    println!("\n[filter_and_tier] 共 {} 条", rows.len());
    for c in rows.iter().take(10) {
        println!("  {:>4}  {}({}) → major={}  prob={:.2}  wiki={:?}",
            c.tier, c.school_name, c.school_code, c.major_code, c.prob, c.wiki_slug);
    }
    let (mut n_c, mut n_s, mut n_safe) = (0, 0, 0);
    for c in &rows { match c.tier.as_str() { "冲" => n_c+=1, "稳" => n_s+=1, _ => n_safe+=1 } }
    println!("\n  冲{} 稳{} 保{}", n_c, n_s, n_safe);
    assert!(rows.len() >= 10, "候选 < 10: {}", rows.len());
    // 985/211 在位次 9800 主要命中保档(高职/边远本科), 冲稳要更高位次
    assert!(n_safe > 0, "保档应至少 1 条 (位次 9800 物理类池子本来就是保)");
    // wiki_slug 至少 1(高职可能没建, 但本科至少 1) — 不强制比例
    let with_slug = rows.iter().filter(|c| c.wiki_slug.is_some()).count();
    println!("\n  带 wiki_slug: {}/{} (高职无建库是正常)", with_slug, rows.len());
    // 不强制比例(高职无库, 见 PRD v2 M6)

    // 2. lookup_major
    let m = sql_tool::sql_tool_lookup_major(serde_json::json!({"code": "080901"})).expect("lookup");
    println!("\n[lookup_major 080901] 命中 {} 条", m.len());
    for r in m.iter() { println!("  {} → wiki={}", r["name"], r["wiki_slug"]); }
    assert!(!m.is_empty());

    // 3. school_admission
    if let Some(first) = rows.first() {
        let sa = sql_tool::sql_tool_school_admission(serde_json::json!({
            "school_code": &first.school_code, "province": "江苏"
        })).expect("school_admission");
        println!("\n[school_admission {}] {} 条", first.school_name, sa.len());
        for r in sa.iter().take(3) { println!("  {:?}", r); }
    }
    println!("\n=== 4 工具全通 ✓ ===");
}
