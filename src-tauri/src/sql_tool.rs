//! SQL 工具 4 件套(PRD v2 §04 §05) — 给模型当工具调, 全部只读参数化。
//! 事实/数字/位次/概率走这里, 模型不写 SQL。

use rusqlite::{Connection, params};
use serde::Serialize;
use std::path::PathBuf;
use std::collections::HashSet;

/// 解析高考数据库路径, 按优先级:
/// 1. 环境变量 `POLARIS_GK_DB` —— 显式覆盖, 应对异常盘符 / 自定义安装位置;
/// 2. `<用户主目录>/PolarisGaokao/polaris-gaokao.db` —— 跨机器 / 跨用户自适配,
///    与 ETL 脚本 (`_gd_etl.py`) 落库位置保持一致。
///
/// 绝不再写死 `C:\Users\mi\...` 这类带具体用户名/盘符的绝对路径。
fn db_path() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("POLARIS_GK_DB") {
        if !p.trim().is_empty() {
            return Ok(PathBuf::from(p));
        }
    }
    let home = directories::UserDirs::new()
        .ok_or_else(|| "无法定位用户主目录".to_string())?
        .home_dir()
        .to_path_buf();
    Ok(home.join("PolarisGaokao").join("polaris-gaokao.db"))
}

fn open_db() -> Result<Connection, String> {
    let path = db_path()?;
    if !path.exists() {
        return Err(format!(
            "数据库文件不存在: {} (可设环境变量 POLARIS_GK_DB 指向其它位置)",
            path.display()
        ));
    }
    Connection::open(&path)
        .map_err(|e| format!("open db failed: {e} (path={})", path.display()))
}

#[derive(Serialize, Debug)]
pub struct CandidateRow {
    pub school_code: String,
    pub school_name: String,
    pub province: String,
    pub major_code: String,
    pub major_name: String,
    pub group_code: Option<String>,
    pub min_rank: Option<i64>,
    pub min_score: Option<i64>,
    pub plan_count: Option<i64>,
    pub tier: String,        // 冲/稳/保
    pub prob: f64,           // 0-1 录取概率
    pub wiki_slug: Option<String>,  // 拼出的 wiki_slug, 供阶段② Read
    pub subject_group: Option<String>, // 选科要求原文(首选X，再选Y), 供前端展示两道闸理由
}

// ===== 选科分轨与筛选: 第一道闸(首选物理/历史) + 第二道闸(再选科目) =====

/// 规范化单个科目名 → 标准 6 科之一(兼容简写: 政→思想政治、生→生物 等)
fn normalize_subject(s: &str) -> Option<&'static str> {
    match s.trim() {
        "物理" | "物" => Some("物理"),
        "历史" | "史" => Some("历史"),
        "化学" | "化" => Some("化学"),
        "生物" | "生" => Some("生物"),
        "思想政治" | "政治" | "政" => Some("思想政治"),
        "地理" | "地" => Some("地理"),
        _ => None,
    }
}

/// 解析考生选科 → (首选物理/历史, 再选集合)。
/// 兼容三种写法: 数组["物理","化学","生物"] / "物理+化学+生物" / 紧凑形 "物化生"。
fn parse_subjects(raw: &str) -> (Option<String>, HashSet<String>) {
    let mut tokens: Vec<String> = vec![];
    for seg in raw.split(|c| c == '+' || c == '、' || c == '，' || c == ','
                            || c == '/' || c == ' ' || c == '|') {
        let seg = seg.trim();
        if seg.is_empty() { continue; }
        if let Some(n) = normalize_subject(seg) {
            tokens.push(n.to_string());
        } else {
            // 紧凑形(物化生 / 史政地): 逐字拆
            for ch in seg.chars() {
                if let Some(n) = normalize_subject(&ch.to_string()) {
                    tokens.push(n.to_string());
                }
            }
        }
    }
    let mut first = None;
    let mut reselect = HashSet::new();
    for t in tokens {
        if t == "物理" || t == "历史" { first = Some(t); }
        else { reselect.insert(t); }
    }
    (first, reselect)
}

/// 第二道闸: 考生再选科目是否满足某专业组的"再选"要求。
/// subject_group 形如 "首选物理，再选化学" / "再选不限" / "再选化学、生物(2科必选)"。
/// 顿号/逗号 = 且(2科必选), "或" = 或; "不限"/解析失败一律放行。
fn reselect_ok(subject_group: &str, reselect: &HashSet<String>) -> bool {
    let req = match subject_group.split("再选").nth(1) {
        Some(r) => r.trim(),
        None => return true,
    };
    if req.is_empty() || req.starts_with("不限") {
        return true;
    }
    // 去掉尾注 "(2科必选)" / "（2科必选）" / "(2选1)"
    let core = req.split(|c| c == '(' || c == '（').next().unwrap_or(req).trim();
    // 或 / "/" / "／" = 任选其一(OR); 顿号逗号 = 都要(AND)
    if core.contains('或') || core.contains('/') || core.contains('／') {
        return core.split(|c| c == '或' || c == '/' || c == '／')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .any(|s| reselect.contains(s));
    }
    core.split(|c| c == '、' || c == '，' || c == ',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .all(|s| reselect.contains(s))
}

/// 阶段① filter_eligible + rank_to_tier 一步完成(并轨实现)
/// 模型入参: {province, subjects, rank, body_limits?, prefer_cities?, prefer_school_types?, track}
/// 返回 ≥45 条候选, 冲20%/稳40%/保40% 比例, 每条带 wiki_slug。
#[tauri::command]
pub fn sql_tool_filter_and_tier(args: serde_json::Value) -> Result<Vec<CandidateRow>, String> {
    let province = args.get("province").and_then(|v| v.as_str())
        .ok_or("missing province")?.to_string();
    let rank = args.get("rank").and_then(|v| v.as_i64())
        .ok_or("missing rank")? as i64;
    let track = args.get("track").and_then(|v| v.as_str()).unwrap_or("");
    // 解析考生选科 → 第一道闸(首选) + 第二道闸(再选)
    let subjects_raw = match args.get("subjects") {
        Some(serde_json::Value::Array(a)) => a.iter()
            .filter_map(|x| x.as_str()).collect::<Vec<_>>().join("+"),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => String::new(),
    };
    let (subj_first, reselect) = parse_subjects(&subjects_raw);
    // 首选物理/历史: track 入参优先, 否则从 subjects 推断
    let first_pref = if track.contains("物理") { Some("物理".to_string()) }
                     else if track.contains("历史") { Some("历史".to_string()) }
                     else { subj_first };
    let apply_reselect_gate = !reselect.is_empty();
    let prefer_cities: Vec<String> = args.get("prefer_cities")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
        .unwrap_or_default();
    // 默认冲 20%/稳 40%/保 40%
    let want_total = args.get("want_total").and_then(|v| v.as_i64()).unwrap_or(45) as usize;
    let (n_charge, n_steady, n_safe) = (
        (want_total as f64 * 0.20) as usize,
        (want_total as f64 * 0.40) as usize,
        (want_total as f64 * 0.40) as usize,
    );

    let conn = open_db()?;
    // 第一道闸: 按"首选物理/首选历史"分轨(再选项里不含物理/历史二字, 故精确隔离首选)
    let like_track = match first_pref.as_deref() {
        Some("物理") => "%首选物理%".to_string(),
        Some("历史") => "%首选历史%".to_string(),
        _ => "%".to_string(),
    };
    let city_clause = if prefer_cities.is_empty() { String::new() }
        else { format!(" AND s.city IN ({})", prefer_cities.iter().map(|c| format!("'{}'", c)).collect::<Vec<_>>().join(",")) };

    let pool_sql = format!(
        "SELECT a.school_code, s.name, a.province, a.major_code, a.min_rank, a.min_score, a.plan_count, a.subject_group
         FROM admission_min_rank a
         LEFT JOIN school s ON s.name = a.school_code
         WHERE a.province = ?1 AND a.subject_group LIKE ?2
           AND a.year = 2024 AND a.min_rank >= ?3
           {city_clause}
         ORDER BY a.min_rank DESC
         LIMIT ?4"
    );
    // 取整省候选(再选闸在 Rust 侧过滤, 故先放宽 LIMIT, 后按 tier 配额裁到 want_total)
    let pool_size = 100000usize;
    let mut stmt = conn.prepare(&pool_sql).map_err(|e| format!("prepare: {e}"))?;
    let rows = stmt.query_map(
        params![province, like_track, rank, pool_size as i64],
        |r| {
            Ok((
                r.get::<_, Option<String>>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, Option<i64>>(6)?,
                r.get::<_, Option<String>>(7)?,
            ))
        }
    ).map_err(|e| format!("query: {e}"))?;
    let pool: Vec<_> = rows.filter_map(|x| x.ok()).collect();

    // rank_to_tier 规则(按 PRD §05 默认): 冲=min_rank 8000-15000, 稳=15000-25000, 保>=25000
    // 简化: 用 min_rank vs user_rank 的比值给 tier
    let mut out = vec![];
    for row in pool.iter() {
        let (sc, sn, prov, mc, mr, ms, pc, sg) = row;
        let min_rank = match mr { Some(x) => *x, None => continue };
        // 第二道闸: 再选科目筛选(考生未给 subjects 时不拦, 向后兼容)
        if apply_reselect_gate && !reselect_ok(sg.as_deref().unwrap_or(""), &reselect) {
            continue;
        }
        // 优志愿式档位(与 gk_match 对齐, 单一事实地基):
        // ratio = min_rank / user_rank, ratio 越大学校去年越容易录取, 概率越高。
        // ratio<0.72 = 够不着删, 0.72-1.0=冲, 1.0-1.3=稳, ≥1.3=保; 1.6 以上=浪费分删。
        // 注: 本函数取不到 school.city, 省外升档的逻辑只在 gk_match 里实现(那是用户实际看到的列表)。
        let ratio = min_rank as f64 / rank as f64;
        if ratio < 0.72 || ratio > 1.6 { continue; }
        let (tier, prob) = if ratio < 1.0 { ("冲", 0.30) }
                          else if ratio < 1.3 { ("稳", 0.62) }
                          else { ("保", 0.85) };
        // 拼 wiki_slug: 形式 major-XXXX 或 school-{name}
        let wiki_slug = mc.as_ref().filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
            .map(|c| format!("major-{}", c));
        out.push(CandidateRow {
            school_code: sc.clone().unwrap_or_default(),
            school_name: sn.clone().unwrap_or_default(),
            province: prov.clone(),
            major_code: mc.clone().unwrap_or_default(),
            major_name: String::new(),
            group_code: None,
            min_rank: Some(min_rank),
            min_score: *ms,
            plan_count: *pc,
            tier: tier.into(),
            prob,
            wiki_slug,
            subject_group: sg.clone(),
        });
    }
    // 按 tier 配额(冲 → 稳 → 保)排序后取 want_total
    out.sort_by_key(|c| (c.tier.clone(), -c.min_rank.unwrap_or(0)));
    let mut counts = (0, 0, 0);
    let mut picked = vec![];
    for c in out.into_iter() {
        let n = match c.tier.as_str() { "冲" => &mut counts.0, "稳" => &mut counts.1, _ => &mut counts.2 };
        if *n < match c.tier.as_str() { "冲" => n_charge, "稳" => n_steady, _ => n_safe } {
            *n += 1;
            picked.push(c);
        }
    }
    Ok(picked)
}

/// 查某专业事实行(代码/关键词)
#[tauri::command]
pub fn sql_tool_lookup_major(args: serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
    let code = args.get("code").and_then(|v| v.as_str()).unwrap_or("");
    let keyword = args.get("keyword").and_then(|v| v.as_str()).unwrap_or("");
    let conn = open_db()?;
    let sql = "SELECT major_code, name, category, subcategory, degree FROM major
               WHERE (?1 = '' OR major_code = ?1) OR (?2 = '' OR name LIKE '%'||?2||'%')
               LIMIT 20";
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![code, keyword], |r| {
        Ok(serde_json::json!({
            "major_code": r.get::<_, Option<String>>(0)?,
            "name":       r.get::<_, Option<String>>(1)?,
            "category":   r.get::<_, Option<String>>(2)?,
            "subcategory": r.get::<_, Option<String>>(3)?,
            "degree":     r.get::<_, Option<String>>(4)?,
            "wiki_slug":  format!("major-{}", r.get::<_, Option<String>>(0)?.unwrap_or_default()),
        }))
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// 查某院校近年提档位次
#[tauri::command]
pub fn sql_tool_school_admission(args: serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
    let sc = args.get("school_code").and_then(|v| v.as_str()).unwrap_or("");
    let prov = args.get("province").and_then(|v| v.as_str()).unwrap_or("");
    if sc.is_empty() { return Err("missing school_code".into()); }
    let conn = open_db()?;
    let sql = "SELECT year, subject_group, min_rank, min_score FROM admission_min_rank
               WHERE school_code = ?1 AND (?2 = '' OR province = ?2)
               ORDER BY year DESC, subject_group LIMIT 30";
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![sc, prov], |r| {
        Ok(serde_json::json!({
            "year":    r.get::<_, Option<i64>>(0)?,
            "subject_group": r.get::<_, Option<String>>(1)?,
            "min_rank": r.get::<_, Option<i64>>(2)?,
            "min_score": r.get::<_, Option<i64>>(3)?,
        }))
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ════════════════════════════════════════════════════════════════════════
//  GK 核心引擎 (优志愿式填报流) — 真实数据驱动, 全部基于 admission_min_rank(2024)
//  分数→位次插值 / 冲稳保匹配 / 地区·层次 facet / 院校详情
// ════════════════════════════════════════════════════════════════════════

/// 院校所在地省份推断: school 表 province 字段为空, 但 city(市/区)有值。
/// 用一张映射把"海淀区/杭州市/南京"等归到省份, 支撑"地区筛选"。未命中归"其他"。
fn city_to_province(city: &str) -> &'static str {
    let c = city.trim();
    const BJ: &[&str] = &["海淀","朝阳","东城","西城","丰台","石景山","通州","昌平","大兴","房山","顺义","门头沟","平谷","怀柔","密云","延庆"];
    const SH: &[&str] = &["杨浦","徐汇","黄浦","静安","长宁","普陀","虹口","闵行","宝山","嘉定","浦东","金山","松江","青浦","奉贤","崇明"];
    const TJ: &[&str] = &["和平","河东","河西","南开","红桥","东丽","西青","津南","北辰","武清","宝坻","滨海","宁河","静海","蓟州"];
    const CQ: &[&str] = &["渝中","江北","沙坪坝","九龙坡","南岸","北碚","渝北","巴南","万州","涪陵","永川","合川","江津","璧山","大渡口"];
    for k in BJ { if c.contains(k) { return "北京"; } }
    for k in SH { if c.contains(k) { return "上海"; } }
    for k in TJ { if c.contains(k) { return "天津"; } }
    for k in CQ { if c.contains(k) { return "重庆"; } }
    if c.starts_with("北京") { return "北京"; }
    if c.starts_with("上海") { return "上海"; }
    if c.starts_with("天津") { return "天津"; }
    if c.starts_with("重庆") { return "重庆"; }
    let map: &[(&str,&str)] = &[
        ("广州","广东"),("深圳","广东"),("珠海","广东"),("汕头","广东"),("佛山","广东"),("东莞","广东"),("湛江","广东"),("中山","广东"),
        ("南京","江苏"),("苏州","江苏"),("无锡","江苏"),("常州","江苏"),("徐州","江苏"),("南通","江苏"),("扬州","江苏"),("镇江","江苏"),
        ("杭州","浙江"),("宁波","浙江"),("温州","浙江"),("嘉兴","浙江"),("金华","浙江"),("绍兴","浙江"),("台州","浙江"),
        ("武汉","湖北"),("宜昌","湖北"),("襄阳","湖北"),("黄石","湖北"),("十堰","湖北"),
        ("成都","四川"),("绵阳","四川"),("南充","四川"),("泸州","四川"),
        ("西安","陕西"),("咸阳","陕西"),("宝鸡","陕西"),("延安","陕西"),
        ("长沙","湖南"),("湘潭","湖南"),("株洲","湖南"),("衡阳","湖南"),("岳阳","湖南"),
        ("济南","山东"),("青岛","山东"),("烟台","山东"),("威海","山东"),("淄博","山东"),("潍坊","山东"),("泰安","山东"),("济宁","山东"),
        ("郑州","河南"),("开封","河南"),("洛阳","河南"),("新乡","河南"),
        ("沈阳","辽宁"),("大连","辽宁"),("鞍山","辽宁"),("锦州","辽宁"),
        ("哈尔滨","黑龙江"),("大庆","黑龙江"),("齐齐哈尔","黑龙江"),
        ("长春","吉林"),("延边","吉林"),
        ("福州","福建"),("厦门","福建"),("泉州","福建"),("漳州","福建"),
        ("合肥","安徽"),("芜湖","安徽"),("蚌埠","安徽"),("马鞍山","安徽"),
        ("南昌","江西"),("赣州","江西"),("九江","江西"),("景德镇","江西"),
        ("石家庄","河北"),("保定","河北"),("唐山","河北"),("秦皇岛","河北"),("廊坊","河北"),("邯郸","河北"),
        ("太原","山西"),("大同","山西"),("晋中","山西"),
        ("昆明","云南"),("大理","云南"),("曲靖","云南"),
        ("贵阳","贵州"),("遵义","贵州"),
        ("南宁","广西"),("桂林","广西"),("柳州","广西"),("北海","广西"),
        ("兰州","甘肃"),("天水","甘肃"),
        ("呼和浩特","内蒙古"),("包头","内蒙古"),("赤峰","内蒙古"),
        ("乌鲁木齐","新疆"),("石河子","新疆"),("克拉玛依","新疆"),
        ("银川","宁夏"),("石嘴山","宁夏"),
        ("西宁","青海"),
        ("拉萨","西藏"),
        ("海口","海南"),("三亚","海南"),
        ("吉林","吉林"),
    ];
    for (k,v) in map { if c.contains(k) { return v; } }
    "其他"
}

/// 兜底: s.city 缺失时(ETL 漏录省外校在 s 表里的 city), 用校名常见地名前缀识别省份。
/// 命中率不高(很多省外高校校名不带省份), 但能捞回一部分 "在广东招生但 s 表没记 city" 的省外校,
/// 让「省外稳→冲」升档真正生效。识别不出返回 "其他", 不动档(避免误伤本省校)。
fn school_region_from_name(name: &str) -> String {
    // 直辖市
    if name.starts_with("北京") { return "北京".into(); }
    if name.starts_with("上海") { return "上海".into(); }
    if name.starts_with("天津") { return "天津".into(); }
    if name.starts_with("重庆") { return "重庆".into(); }
    // 省份: 一字简称前缀
    const PREFIX: &[(&str, &str)] = &[
        ("河北", "河北"), ("山西", "山西"), ("辽宁", "辽宁"), ("吉林", "吉林"), ("黑龙江", "黑龙江"),
        ("江苏", "江苏"), ("浙江", "浙江"), ("安徽", "安徽"), ("福建", "福建"), ("江西", "江西"),
        ("山东", "山东"), ("河南", "河南"), ("湖北", "湖北"), ("湖南", "湖南"), ("广东", "广东"),
        ("海南", "海南"), ("四川", "四川"), ("贵州", "贵州"), ("云南", "云南"), ("陕西", "陕西"),
        ("甘肃", "甘肃"), ("青海", "青海"), ("台湾", "台湾"),
    ];
    for (k, v) in PREFIX {
        if name.starts_with(k) { return (*v).to_string(); }
    }
    // 自治区分两字(避免和省份冲突) + 三字
    const AUT: &[(&str, &str)] = &[
        ("内蒙古", "内蒙古"), ("广西", "广西"), ("西藏", "西藏"),
        ("宁夏", "宁夏"), ("新疆", "新疆"),
    ];
    for (k, v) in AUT {
        if name.starts_with(k) { return (*v).to_string(); }
    }
    "其他".to_string()
}

/// 首选 → subject_group LIKE 模板
fn track_like(track: &str) -> String {
    if track.contains("物理") { "%首选物理%".to_string() }
    else if track.contains("历史") { "%首选历史%".to_string() }
    else { "%".to_string() }
}

/// 概率模型: 越好(位次数越小)于该专业去年录取最低位次 → 概率越高。
fn prob_of(user_rank: i64, min_rank: i64) -> f64 {
    if min_rank <= 0 || user_rank <= 0 { return 0.0; }
    let diff = (min_rank - user_rank) as f64 / user_rank as f64;
    1.0 / (1.0 + (-diff / 0.16).exp())
}
fn tier_of(p: f64) -> &'static str {
    if p >= 0.78 { "保" } else if p >= 0.45 { "稳" } else { "冲" }
}

/// 省份列表(供省份选择器)
#[tauri::command]
pub fn gk_provinces() -> Result<Vec<serde_json::Value>, String> {
    let conn = open_db()?;
    let mut stmt = conn.prepare(
        "SELECT province, COUNT(*) FROM admission_min_rank WHERE year=2024 GROUP BY province ORDER BY 2 DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |r| Ok(serde_json::json!({
        "province": r.get::<_, String>(0)?,
        "rows": r.get::<_, i64>(1)?,
    }))).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// 分数 → 位次 (基于该省该轨真实(min_score,min_rank)对线性插值, 近似一分一段)
#[tauri::command]
pub fn gk_score_to_rank(args: serde_json::Value) -> Result<serde_json::Value, String> {
    let province = args.get("province").and_then(|v| v.as_str()).ok_or("missing province")?;
    let track = args.get("track").and_then(|v| v.as_str()).unwrap_or("");
    let score = args.get("score").and_then(|v| v.as_i64()).ok_or("missing score")?;
    let conn = open_db()?;
    let rank = interp_rank(&conn, province, track, score)?;
    Ok(serde_json::json!({ "rank": rank, "score": score, "province": province, "track": track }))
}

/// 内部: 分数→位次插值
/// 把用户 track 归一到一分一段表(score_rank)的科类取值。
/// 注意判定顺序:"物理"含"理", 必须先判物理再判理科。
fn score_track(track: &str) -> &'static str {
    if track.contains("物理") { "物理类" }
    else if track.contains("历史") { "历史类" }
    else if track.contains("理") { "理科" }
    else if track.contains("文") { "文科" }
    else { "不分科" }
}

/// 精确分数→位次:取该省该科类**最新年份**的一分一段表线性插值。
/// 有数据返回 Some(累计位次), 否则 None(由调用方回退到录取位次点插值)。
fn interp_rank_score_table(conn: &Connection, province: &str, track: &str, score: i64) -> Option<i64> {
    let st = score_track(track);
    let year: i64 = conn
        .query_row(
            "SELECT MAX(year) FROM score_rank WHERE province=?1 AND track=?2",
            params![province, st],
            |r| r.get(0),
        )
        .ok()?; // 无行时 MAX 为 NULL → get::<i64> 失败 → None
    let mut stmt = conn
        .prepare(
            "SELECT score, rank_cumulative FROM score_rank
             WHERE province=?1 AND track=?2 AND year=?3 ORDER BY score ASC",
        )
        .ok()?;
    let pts: Vec<(i64, i64)> = stmt
        .query_map(params![province, st, year], |r| Ok((r.get(0)?, r.get(1)?)))
        .ok()?
        .filter_map(|x| x.ok())
        .collect();
    if pts.len() < 2 {
        return None;
    }
    // 分数↑ → 累计位次↓(越好)。线性插值。
    if score >= pts.last().unwrap().0 {
        return Some(pts.last().unwrap().1.max(1));
    }
    if score <= pts.first().unwrap().0 {
        return Some(pts.first().unwrap().1);
    }
    for w in pts.windows(2) {
        let (s0, r0) = w[0];
        let (s1, r1) = w[1];
        if score >= s0 && score <= s1 {
            let t = if s1 == s0 { 0.0 } else { (score - s0) as f64 / (s1 - s0) as f64 };
            let r = r0 as f64 + (r1 - r0) as f64 * t;
            return Some((r.round() as i64).max(1));
        }
    }
    Some(pts.last().unwrap().1.max(1))
}

fn interp_rank(conn: &Connection, province: &str, track: &str, score: i64) -> Result<i64, String> {
    // ① 优先用精确一分一段表(score_rank); 命中即返回。
    if let Some(r) = interp_rank_score_table(conn, province, track, score) {
        return Ok(r);
    }
    // ② 回退: 用该省录取最低位次点插值(旧逻辑, 精度较粗)。
    let like = track_like(track);
    let mut stmt = conn.prepare(
        "SELECT min_score, AVG(min_rank) FROM admission_min_rank
         WHERE province=?1 AND subject_group LIKE ?2 AND min_score IS NOT NULL AND min_rank IS NOT NULL
         GROUP BY min_score ORDER BY min_score ASC"
    ).map_err(|e| e.to_string())?;
    let pts: Vec<(i64, f64)> = stmt.query_map(params![province, like], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?))
    }).map_err(|e| e.to_string())?.filter_map(|x| x.ok()).collect();
    if pts.is_empty() { return Err("该省该科类暂无数据".into()); }
    if score >= pts.last().unwrap().0 {
        return Ok(pts.last().unwrap().1.max(1.0).round() as i64);
    }
    if score <= pts.first().unwrap().0 {
        return Ok(pts.first().unwrap().1.round() as i64);
    }
    for w in pts.windows(2) {
        let (s0, r0) = w[0];
        let (s1, r1) = w[1];
        if score >= s0 && score <= s1 {
            let t = if s1 == s0 { 0.0 } else { (score - s0) as f64 / (s1 - s0) as f64 };
            let r = r0 + (r1 - r0) * t;
            return Ok(r.max(1.0).round() as i64);
        }
    }
    Ok(pts.last().unwrap().1.round() as i64)
}

/// 核心匹配: 位次 × 选科 × 筛选 → 看板统计 + facet + 分页列表
#[tauri::command]
pub fn gk_match(args: serde_json::Value) -> Result<serde_json::Value, String> {
    let province = args.get("province").and_then(|v| v.as_str()).ok_or("missing province")?.to_string();
    let track = args.get("track").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let conn = open_db()?;

    let rank = match args.get("rank").and_then(|v| v.as_i64()) {
        Some(r) => r,
        None => {
            let score = args.get("score").and_then(|v| v.as_i64()).ok_or("missing rank or score")?;
            interp_rank(&conn, &province, &track, score)?
        }
    };

    let subjects_raw = match args.get("subjects") {
        Some(serde_json::Value::Array(a)) => a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join("+"),
        Some(serde_json::Value::String(s)) => s.clone(),
        _ => String::new(),
    };
    let (_first, reselect) = parse_subjects(&subjects_raw);
    let apply_reselect = !reselect.is_empty();

    let like = track_like(&track);
    let sql = "SELECT a.school_code, s.city, s.is_985, s.is_211, s.is_double_first, s.school_type,
                      a.major_code, a.subject_group, a.min_rank, a.min_score, a.group_code
               FROM admission_min_rank a LEFT JOIN school s ON s.name = a.school_code
               WHERE a.province=?1 AND a.year=2024 AND a.subject_group LIKE ?2 AND a.min_rank IS NOT NULL";
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let raw = stmt.query_map(params![province, like], |r| Ok((
        r.get::<_, String>(0)?,
        r.get::<_, Option<String>>(1)?,
        r.get::<_, Option<i64>>(2)?.unwrap_or(0),
        r.get::<_, Option<i64>>(3)?.unwrap_or(0),
        r.get::<_, Option<String>>(4)?,
        r.get::<_, Option<String>>(5)?,
        r.get::<_, String>(6)?,
        r.get::<_, String>(7)?,
        r.get::<_, i64>(8)?,
        r.get::<_, Option<i64>>(9)?,
        r.get::<_, Option<String>>(10)?,
    ))).map_err(|e| e.to_string())?.filter_map(|x| x.ok()).collect::<Vec<_>>();

    #[derive(Clone)]
    struct Cand {
        school: String, city: String, region: String, level: String, stype: String,
        major: String, subject_group: String, min_rank: i64, min_score: Option<i64>,
        group_code: Option<String>, tier: String, prob: f64, is985: bool, is211: bool, isdf: bool,
    }
    // 优志愿式"适合"带: 冲档拉远到分差 ≈110 分, 保底分差放到 80 分, 稳档维持分差 0-80。
    // - 下限 lo = 位次×0.6: 冲档放宽, 给"搏一搏"留更多选择(够不够得着由概率自然标"冲")。
    // - 上限 hi = 位次×1.6: 防窗口太宽把差太远的也卷进来。
    let lo = (rank as f64 * 0.60) as i64;
    let hi = (rank as f64 * 1.6) as i64;
    let mut cands: Vec<Cand> = Vec::new();
    for (school, city, c985, c211, df, stype, major, sg, min_rank, min_score, gc) in raw {
        if apply_reselect && !reselect_ok(&sg, &reselect) { continue; }
        if min_rank < lo || min_rank > hi { continue; }
        let is985 = c985 == 1;
        let is211 = c211 == 1;
        let isdf = df.as_deref().map(|s| s.contains("双一流")).unwrap_or(false);
        let level = if is985 { "985" } else if is211 { "211" } else if isdf { "双一流" } else { "普通本科" };
        let region = city_to_province(city.as_deref().unwrap_or(""));
        let p = prob_of(rank, min_rank);
        // 优志愿式风险修正: 省外院校异地招生波动大、风险高 —— 本来算"稳"的降一档归入"冲"。
        // 优先看 s.city 推断; s.city 为空/未识别时, 用校名常见前缀/地名词二次识别(补 ETL 漏录的省外校)。
        let region = {
            let r1 = city_to_province(city.as_deref().unwrap_or(""));
            if r1 != "其他" {
                r1.to_string()
            } else {
                school_region_from_name(&school)
            }
        };
        let mut tier = tier_of(p).to_string();
        if tier == "稳" && region != "其他" && region != province.as_str() {
            tier = "冲".to_string();
        }
        cands.push(Cand {
            school, city: city.unwrap_or_default(), region: region.to_string(),
            level: level.to_string(), stype: stype.unwrap_or_else(|| "其他".to_string()),
            major, subject_group: sg, min_rank, min_score, group_code: gc,
            tier, prob: p, is985, is211, isdf,
        });
    }

    let mut f_region: std::collections::BTreeMap<String, i64> = Default::default();
    let mut f_level: std::collections::BTreeMap<String, i64> = Default::default();
    let mut f_type: std::collections::BTreeMap<String, i64> = Default::default();
    for c in &cands {
        *f_region.entry(c.region.clone()).or_insert(0) += 1;
        *f_level.entry(c.level.clone()).or_insert(0) += 1;
        *f_type.entry(c.stype.clone()).or_insert(0) += 1;
    }

    let regions: Vec<String> = args.get("regions").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let levels: Vec<String> = args.get("levels").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let types: Vec<String> = args.get("types").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let tiers: Vec<String> = args.get("tiers").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
    let keyword = args.get("keyword").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();

    // 保底进一步收紧: 只留分差 ≤ ~80 分的保底(位次 1.4×rank 内)。
    // 冲/稳的窗口已在上面 0.65-1.0 / 1.0-1.3 圈定, 此处不再加窗。
    let filtered: Vec<&Cand> = cands.iter().filter(|c| {
        let tier_window = match c.tier.as_str() {
            "保" => (rank as f64 * 1.0) as i64..=(rank as f64 * 1.4) as i64,
            _ => 0..=i64::MAX,
        };
        if !tier_window.contains(&c.min_rank) { return false; }
        (regions.is_empty() || regions.contains(&c.region)) &&
        (levels.is_empty() || levels.contains(&c.level)) &&
        (types.is_empty() || types.contains(&c.stype)) &&
        (tiers.is_empty() || tiers.contains(&c.tier)) &&
        (keyword.is_empty() || c.school.contains(&keyword) || c.major.contains(&keyword))
    }).collect();

    // 配额裁到 ~200-300(总), 冲:稳:保 ≈ 1:1:1, 让每个分段都尽量铺满候选,
    // 用户再靠筛选器(地区/层次/类型/关键词)挑到想要的志愿。
    // 池子分两路: a) 每档按配额裁到 want; b) 总数仍按 page_size 翻页。
    // 注意: stats 用裁前的 "filtered" 之外的 cands 数(显示池子规模)——这里 stat 用裁后即可, 用户要的就是缩量后的池子。
    // 冲/稳/保 各自配额, 不足则有多少给多少(不做补位, 免稀释档位语义);
    // 分段候选多的(如热门位次)就铺得满, 候选少的有多少给多少。
    let quota = |tier: &str| -> usize {
        match tier {
            "冲" => 100, // 冲档窗口 0.65×rank(分差 ≈100), 配额给到 100 让"搏一搏"更丰富
            "稳" => 100, // 稳档窗口 1.0-1.3×rank(分差 0-80), 配额给到 100; 池子不到 100 也不补
            _ => 100,    // 保底 1.4×rank(分差 ≈80), 配额给够
        }
    };
    // 各档先排序, 再按配额裁
    let mut by_tier: std::collections::BTreeMap<&str, Vec<&Cand>> = std::collections::BTreeMap::new();
    for c in &filtered {
        by_tier.entry(c.tier.as_str()).or_default().push(*c);
    }
    for (_, v) in by_tier.iter_mut() {
        v.sort_by(|a, b| b.prob.partial_cmp(&a.prob).unwrap_or(std::cmp::Ordering::Equal));
    }
    let mut truncated: Vec<&Cand> = Vec::new();
    for tier in ["冲", "稳", "保"] {
        if let Some(v) = by_tier.get(tier) {
            truncated.extend(v.iter().take(quota(tier)).copied());
        }
    }

    let (mut n_charge, mut n_steady, mut n_safe) = (0i64, 0i64, 0i64);
    let (mut n985, mut n211, mut ndf) = (0i64, 0i64, 0i64);
    for c in &truncated {
        match c.tier.as_str() { "冲" => n_charge += 1, "稳" => n_steady += 1, _ => n_safe += 1 }
        if c.is985 { n985 += 1; }
        if c.is211 { n211 += 1; }
        if c.isdf { ndf += 1; }
    }

    let sort = args.get("sort").and_then(|v| v.as_str()).unwrap_or("prob");
    let mut sorted = truncated.clone();
    if sort == "rank" {
        sorted.sort_by_key(|c| c.min_rank);
    } else {
        sorted.sort_by(|a, b| b.prob.partial_cmp(&a.prob).unwrap_or(std::cmp::Ordering::Equal));
    }

    let page = args.get("page").and_then(|v| v.as_i64()).unwrap_or(0).max(0) as usize;
    // 配额裁后总 ≤ 300, 默认 300 一次取全, 前端自行分页展示。
    let page_size = args.get("page_size").and_then(|v| v.as_i64()).unwrap_or(300).clamp(1, 320) as usize;
    let total = sorted.len();
    let start = (page * page_size).min(total);
    let end = (start + page_size).min(total);
    let rows_json: Vec<serde_json::Value> = sorted[start..end].iter().map(|c| serde_json::json!({
        "school": c.school, "city": c.city, "region": c.region, "level": c.level,
        "school_type": c.stype, "major": c.major, "subject_group": c.subject_group,
        "min_rank": c.min_rank, "min_score": c.min_score, "group_code": c.group_code,
        "tier": c.tier, "prob": (c.prob * 100.0).round() / 100.0,
        "is985": c.is985, "is211": c.is211, "double_first": c.isdf,
        "rank_delta": rank - c.min_rank,
    })).collect();

    let to_facet = |m: std::collections::BTreeMap<String,i64>| -> Vec<serde_json::Value> {
        let mut v: Vec<_> = m.into_iter().map(|(k,n)| serde_json::json!({"key":k,"count":n})).collect();
        v.sort_by(|a,b| b["count"].as_i64().unwrap_or(0).cmp(&a["count"].as_i64().unwrap_or(0)));
        v
    };

    Ok(serde_json::json!({
        "rank": rank,
        "province": province,
        "track": track,
        "stats": {
            "total": total,
            "charge": n_charge, "steady": n_steady, "safe": n_safe,
            "c985": n985, "c211": n211, "double_first": ndf,
        },
        "facets": {
            "region": to_facet(f_region),
            "level": to_facet(f_level),
            "type": to_facet(f_type),
        },
        "page": page, "page_size": page_size,
        "rows": rows_json,
    }))
}

/// 院校改名桥接: 高职升本(职业技术学院→职业技术大学 / 职业学院→职业大学)后,
/// 同一所学校历年录取数据被拆在新旧两个名字下(如"深圳职业技术大学"2023升本,
/// 2022 及以前挂在"深圳职业技术学院")。返回需要一起查的等价校名集合(去重, ≤2 个),
/// 让"近三年对比"能跨改名连起来。仅对"职业…"高职升本场景做替换, 不误并普通本科。
fn school_aliases(name: &str) -> Vec<String> {
    let mut set: Vec<String> = vec![name.to_string()];
    let swaps = [
        ("职业技术大学", "职业技术学院"),
        ("职业技术学院", "职业技术大学"),
        ("职业大学", "职业学院"),
        ("职业学院", "职业大学"),
    ];
    for (from, to) in swaps {
        if name.contains(from) {
            let alt = name.replace(from, to);
            if !set.contains(&alt) { set.push(alt); }
        }
    }
    set
}

/// 院校详情: flag + 本省该轨开设专业录取(去重, 跨改名近三年)
#[tauri::command]
pub fn gk_school_detail(args: serde_json::Value) -> Result<serde_json::Value, String> {
    let name = args.get("name").and_then(|v| v.as_str()).ok_or("missing name")?;
    let province = args.get("province").and_then(|v| v.as_str()).unwrap_or("");
    let track = args.get("track").and_then(|v| v.as_str()).unwrap_or("");
    let conn = open_db()?;

    let aliases = school_aliases(name);
    // IN (?,?...) 占位符, 用别名集合跨改名一起查
    let ph = (0..aliases.len()).map(|_| "?").collect::<Vec<_>>().join(",");

    // info: 优先精确命中请求名, 否则取任一别名(改名后 school 表可能只剩旧名)
    let info_sql = format!(
        "SELECT name, city, is_985, is_211, is_double_first, is_c9, is_central, school_type, dept
         FROM school WHERE name IN ({}) ORDER BY (name=?{}) DESC LIMIT 1",
        ph, aliases.len() + 1
    );
    let mut info_params: Vec<&dyn rusqlite::ToSql> = aliases.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    info_params.push(&name);
    let info = conn.query_row(
        &info_sql,
        info_params.as_slice(),
        |r| Ok(serde_json::json!({
            // 展示用请求名(改名后官方现名), 不暴露库里残留的旧校名
            "name": name,
            "city": r.get::<_, Option<String>>(1)?,
            "is985": r.get::<_, Option<i64>>(2)?.unwrap_or(0) == 1,
            "is211": r.get::<_, Option<i64>>(3)?.unwrap_or(0) == 1,
            "double_first": r.get::<_, Option<String>>(4)?,
            "is_c9": r.get::<_, Option<i64>>(5)?.unwrap_or(0) == 1,
            "is_central": r.get::<_, Option<i64>>(6)?.unwrap_or(0) == 1,
            "school_type": r.get::<_, Option<String>>(7)?,
            "dept": r.get::<_, Option<String>>(8)?,
        }))
    ).unwrap_or_else(|_| serde_json::json!({ "name": name }));

    let like = track_like(track);

    // 该校(含改名别名)该省该科类有数据的"年份"列表(倒序), 前端据此做年份切换 + 近三年对比
    // LENGTH(major_code)<=25: 滤掉抽取污染的超长专业名(整段培养说明被灌进专业名列, 全库约 6k 行)
    let years_sql = format!(
        "SELECT DISTINCT year FROM admission_min_rank
         WHERE school_code IN ({}) AND (?{}='' OR province=?{}) AND subject_group LIKE ?{} AND min_rank IS NOT NULL
           AND LENGTH(major_code)<=25
         ORDER BY year DESC",
        ph, aliases.len() + 1, aliases.len() + 1, aliases.len() + 2
    );
    let mut ystmt = conn.prepare(&years_sql).map_err(|e| e.to_string())?;
    let mut yp: Vec<&dyn rusqlite::ToSql> = aliases.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    yp.push(&province); yp.push(&like);
    let years: Vec<i64> = ystmt.query_map(yp.as_slice(), |r| r.get::<_, i64>(0))
        .map_err(|e| e.to_string())?.filter_map(|x| x.ok()).collect();
    // 只取最近三年, 避免老高考(理科/文科)混入新高考造成口径不一
    let recent: Vec<i64> = years.iter().take(3).cloned().collect();
    let year_min = recent.iter().cloned().min().unwrap_or(0);

    // 录取专业行: 带 year, 跨改名别名, 限定最近三年, 已在库层去重(ux_amr_dedupe), 这里再 DISTINCT 兜底
    let rows_sql = format!(
        "SELECT DISTINCT major_code, subject_group, min_rank, min_score, group_code, year
         FROM admission_min_rank
         WHERE school_code IN ({}) AND (?{}='' OR province=?{}) AND subject_group LIKE ?{}
           AND min_rank IS NOT NULL AND year>=?{} AND LENGTH(major_code)<=25
         ORDER BY year DESC, min_rank ASC LIMIT 600",
        ph, aliases.len() + 1, aliases.len() + 1, aliases.len() + 2, aliases.len() + 3
    );
    let mut stmt = conn.prepare(&rows_sql).map_err(|e| e.to_string())?;
    let mut rp: Vec<&dyn rusqlite::ToSql> = aliases.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    rp.push(&province); rp.push(&like); rp.push(&year_min);
    let majors: Vec<serde_json::Value> = stmt.query_map(rp.as_slice(), |r| Ok(serde_json::json!({
        "major": r.get::<_, String>(0)?,
        "subject_group": r.get::<_, Option<String>>(1)?,
        "min_rank": r.get::<_, Option<i64>>(2)?,
        "min_score": r.get::<_, Option<i64>>(3)?,
        "group_code": r.get::<_, Option<String>>(4)?,
        "year": r.get::<_, i64>(5)?,
    }))).map_err(|e| e.to_string())?.filter_map(|x| x.ok()).collect();

    Ok(serde_json::json!({ "info": info, "majors": majors, "years": recent }))
}

/// 组级下钻: 给定院校+专业组代码, 返回该组包含的专业(及招生计划数)。
/// 广东"院校专业组"投档是按组一条线, 进组后再分专业 → 组级投档线 + 组内专业构成。
/// 2025 投档数据只到组级, 专业构成取最近有招生计划的年份(广东 plan 覆盖 2021-2024),
/// 返回 plan_year 供前端标注"构成参考 N 年招生计划"。
#[tauri::command]
pub fn gk_group_majors(args: serde_json::Value) -> Result<serde_json::Value, String> {
    let name = args.get("name").and_then(|v| v.as_str()).ok_or("missing name")?;
    let group_code = args.get("group_code").and_then(|v| v.as_str()).unwrap_or("");
    let want_year = args.get("year").and_then(|v| v.as_i64()).unwrap_or(9999);
    if group_code.is_empty() {
        return Ok(serde_json::json!({ "majors": [], "plan_year": null }));
    }
    let conn = open_db()?;
    let aliases = school_aliases(name);
    let ph = (0..aliases.len()).map(|_| "?").collect::<Vec<_>>().join(",");

    // 专业构成取"≤目标年份里最近的一年"(无则取最近的一年), 让 2025 用 2024 构成兜底
    let year_sql = format!(
        "SELECT year FROM plan
         WHERE school_code IN ({}) AND group_code=?{} AND major_code IS NOT NULL
         ORDER BY (year<=?{}) DESC, ABS(year-?{}) ASC LIMIT 1",
        ph, aliases.len() + 1, aliases.len() + 2, aliases.len() + 2
    );
    let mut yp: Vec<&dyn rusqlite::ToSql> = aliases.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    yp.push(&group_code); yp.push(&want_year);
    let plan_year: Option<i64> = conn.query_row(&year_sql, yp.as_slice(), |r| r.get::<_, i64>(0)).ok();

    let majors: Vec<serde_json::Value> = if let Some(py) = plan_year {
        let m_sql = format!(
            "SELECT DISTINCT major_code, plan_count FROM plan
             WHERE school_code IN ({}) AND group_code=?{} AND year=?{}
               AND major_code IS NOT NULL AND LENGTH(major_code)<=25
             ORDER BY plan_count DESC, major_code",
            ph, aliases.len() + 1, aliases.len() + 2
        );
        let mut mp: Vec<&dyn rusqlite::ToSql> = aliases.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
        mp.push(&group_code); mp.push(&py);
        let mut stmt = conn.prepare(&m_sql).map_err(|e| e.to_string())?;
        let v: Vec<serde_json::Value> = stmt.query_map(mp.as_slice(), |r| Ok(serde_json::json!({
            "major": r.get::<_, String>(0)?,
            "plan_count": r.get::<_, Option<i64>>(1)?,
        }))).map_err(|e| e.to_string())?.filter_map(|x| x.ok()).collect();
        v
    } else { vec![] };

    Ok(serde_json::json!({ "majors": majors, "plan_year": plan_year }))
}

/// 健康检查(查表/库/路径)
#[tauri::command]
pub fn sql_tool_status() -> Result<serde_json::Value, String> {
    let path = db_path()?;
    if !path.exists() {
        return Ok(serde_json::json!({"ok": false, "err": "db not found", "path": path.display().to_string()}));
    }
    let conn = open_db()?;
    let stats: serde_json::Value = serde_json::json!({
        "school": conn.query_row("SELECT COUNT(*) FROM school", [], |r| r.get::<_, i64>(0)).unwrap_or(0),
        "major":  conn.query_row("SELECT COUNT(*) FROM major",  [], |r| r.get::<_, i64>(0)).unwrap_or(0),
        "admission_min_rank": conn.query_row("SELECT COUNT(*) FROM admission_min_rank", [], |r| r.get::<_, i64>(0)).unwrap_or(0),
        "plan":   conn.query_row("SELECT COUNT(*) FROM plan",   [], |r| r.get::<_, i64>(0)).unwrap_or(0),
        "tier_rule": conn.query_row("SELECT COUNT(*) FROM tier_rule", [], |r| r.get::<_, i64>(0)).unwrap_or(0),
        "score_rank": conn.query_row("SELECT COUNT(*) FROM score_rank", [], |r| r.get::<_, i64>(0)).unwrap_or(0),
    });
    Ok(serde_json::json!({"ok": true, "path": path.display().to_string(), "stats": stats}))
}
