<script setup lang="ts">
import { ref, computed } from "vue";

type TagType = "red" | "gold" | "green";
interface Truth {
  name: string;
  cat: string;
  tag: string;
  tagType: TagType;
  fan: string;
  real: string;
  plan: string;
  pros: string;
  fit: string;
}

// 数据底座：载望升学资料·专业篇 / 就业方向篇 / 职业规划篇 / 答疑篇（真实案例）
const TRUTHS: Truth[] = [
  // —— 工科篇 ——
  { name: "计算机 / 软件工程", cat: "工科", tag: "就业面最宽", tagType: "green",
    fan: "高薪躺赚、35 岁财富自由。",
    real: "行业从普涨转入剧烈分化：大厂卡 985/211＋算法，普通院校多进中小厂/外包；“35 岁危机”真实存在；要持续刷题、造项目。",
    plan: "本科就刷 LeetCode、做开源/竞赛/实习；冲头部院校或考研提学历；学习不能停。",
    pros: "仍是工科里就业面最宽、薪资上限最高之一，但红利期已过，靠真实工程能力吃饭。",
    fit: "能自驱、抗压、真喜欢写代码者；不适合只图高薪、坐不住者。" },
  { name: "人工智能 / 数据科学", cat: "工科", tag: "风口·虚高", tagType: "gold",
    fan: "最火风口、起薪最高、一片蓝海。",
    real: "算法岗几乎只要硕博＋顶会论文；本科 AI 易“啥都学一点、啥都不精”；数据分析岗易被挤压。门槛远高于普通编程。",
    plan: "本科打牢数学(线代/概率/优化)＋编程，瞄准读研；做项目/竞赛/论文；否则不如踏实学计算机。",
    pros: "顶端薪资天花板极高、长期需求旺；红利只属于高学历高能力者。",
    fit: "数理极强、能读研深造者；劝退“听说火就报”的普通本科生。" },
  { name: "电子信息 / 微电子", cat: "工科", tag: "被低估", tagType: "green",
    fan: "和计算机一样轻松高薪。",
    real: "硬核难学(数电模电、信号与系统)，挂科率高；薪资略低于纯软件但更稳；芯片是战略风口但要求高学历。",
    plan: "建议读研(尤其微电子/集成电路)；打牢数理基础；瞄准芯片/通信头部企业。",
    pros: "国产替代＋芯片自主，长期前景好且稳定，是“被低估的优质工科”。",
    fit: "数理强、能啃硬课、愿读研者。" },
  { name: "自动化", cat: "工科", tag: "万金油", tagType: "gold",
    fan: "工科万金油、就业面广。",
    real: "“万金油”的另一面是样样通样样松，对口岗位界限模糊；与电气/电子/计算机都沾边却都不如其专；薪资中等。",
    plan: "在校选定一个方向深耕(嵌入式/控制/机器人)并补强；考研或考电网/考公是好出路。",
    pros: "进智能制造/机器人/电网有空间、稳定；纯本科泛而不精易平庸。",
    fit: "动手能力强、愿钻一个方向者。" },
  { name: "机械工程", cat: "工科", tag: "传统·稳", tagType: "gold",
    fan: "工业之母、永不失业。",
    real: "传统稳定但起薪低、涨薪慢；工作常在工厂/车间；“又脏又累钱少”是老牌劝退点；正向高端制造转型。",
    plan: "往智能制造/机器人/新能源汽车靠拢；考研或进国企央企；考注册机械工程师。",
    pros: "高端装备、新能源汽车带来新机会，进好平台稳定；低端机械前景平淡。",
    fit: "真喜欢机械、能下车间、求稳者。" },
  { name: "电气工程及其自动化", cat: "工科", tag: "国网铁饭碗", tagType: "green",
    fan: "进国家电网、铁饭碗。",
    real: "确属优质工科，但进国网/南网要对口院校＋校招笔试，普通院校竞争激烈；非电网方向去电力设计院/制造企业也不错；学习偏硬。",
    plan: "瞄准电网校招(大二三就准备)；或考研、进设计院/新能源；打好电路基础。",
    pros: "电力是刚需＋新能源风口，就业稳定质量高，被广泛认可。",
    fit: "数理好、想进体制/国企求稳者，性价比高。" },
  { name: "土木工程", cat: "工科", tag: "行业下行", tagType: "gold",
    fan: "基建狂魔、铁饭碗、包分配。",
    real: "行业进入下行周期，地产收缩、项目减少；工地常年漂泊、远离城市；薪资增长乏力。",
    plan: "考一级建造师/注册结构师等证书是核心竞争力；可转设计院/国企/考公(住建、规划)；做好驻外吃苦准备。",
    pros: "有证书＋进好平台(央企/设计院)仍稳定；但黄金时代已过。",
    fit: "能吃苦、愿驻工地、考证自驱者；劝退想留大城市坐办公室者。" },
  { name: "建筑学", cat: "工科", tag: "由热转冷", tagType: "red",
    fan: "建筑师、设计大楼、高大上有格调。",
    real: "随地产下行由热转冷；五年制、画图熬夜强度大；就业与薪资明显回落，设计院降薪裁员。",
    plan: "练好设计＋软件作品集；可转规划/景观/建筑表现/考公(规划局)；做好行业周期心理准备。",
    pros: "头部设计院/名校＋作品突出者仍有空间；普通院校前景趋紧。",
    fit: "真有设计天赋与热爱、能熬图者；劝退冲着“高大上”来的人。" },
  { name: "化学 / 材料 / 环境", cat: "工科", tag: "生化环材", tagType: "red",
    fan: "高大上的理工科、好就业。",
    real: "本科就业差、对口岗位少、薪资低、实验有损耗；材料“万金油却万不精”；环境受政策周期影响大。",
    plan: "基本要读研深造，或尽早转行/考公考编。",
    pros: "进头部企业研发、考公考编或转行尚可；纯本科对口前景弱。",
    fit: "立志科研深造者；劝退只看专业名字冲动报考的人。" },
  { name: "生物科学 / 生物工程", cat: "工科", tag: "天坑之首", tagType: "red",
    fan: "“21 世纪是生物的世纪”、生命科学高大上、进实验室搞科研很酷。",
    real: "本科就业极差、对口岗位少且薪资低；行业现实是“不读到博士基本没出路”，而读博周期长、青年科研岗内卷严重。靠“高大上”的想象报考，最易后悔。",
    plan: "只有两条清醒路：① 从大一就坚定走科研——读研读博、争取深造/出国，并接受长期低薪；② 或本科阶段尽早规划转码 / 考公 / 转行。最怕“走一步看一步”。",
    pros: "真爱科研、能读到博、进头部药企 / 测序 / 生物医药公司者，前景可观；纯本科留在本行，前景黯淡。",
    fit: "适合真心热爱科研、家庭能支撑长周期深造者。强烈劝退“分数不低、随便挑个听起来厉害的专业”的考生。" },
  // —— 医学篇 ——
  { name: "临床医学", cat: "医学", tag: "超长周期", tagType: "gold",
    fan: "白大褂受人尊敬、越老越吃香。",
    real: "“5+3+规培”至少 8–11 年才独立执业，本科几乎无法直接当医生；前期收入低、强度高、医患压力大；非顶尖院校就业下沉到县域。",
    plan: "入学即备考研(专硕规培)与执业医师；选好方向与导师；做好“30 岁仍在低薪规培”的准备。",
    pros: "确实越老越值钱、稳定、地位高——代价是十年青春＋持续高压。",
    fit: "家庭能撑长周期、真有奉献心与抗压力者；劝退想快回报者。" },
  { name: "口腔医学", cat: "医学", tag: "性价比高", tagType: "green",
    fan: "牙医赚钱、轻松体面。",
    real: "是医学里少有的就业好、能开私人诊所的方向，但同样要读研规培、好岗位卡学历；开诊所需资金与积累，并非“轻松”。",
    plan: "尽量读研(口腔专硕)；考执业医师；积累临床后再考虑独立执业。",
    pros: "需求大、可市场化、收入上限高，是医学里性价比高的方向。",
    fit: "分数够、能读研、想从医又看重回报者。" },
  { name: "护理学", cat: "医学", tag: "缺口大·辛苦", tagType: "gold",
    fan: "白衣天使、好就业、稳定。",
    real: "就业率高、缺口大，但三班倒、强度高、起薪低、晋升慢与社会偏见是现实；2024 年起调整为国控专业，门槛上升；本科护理比专科有优势。",
    plan: "考护士资格证；向护理管理/专科护士/学校医院发展；英语好可考虑国际护理。",
    pros: "老龄化下需求长期旺盛、稳定，适合求稳定就业者。",
    fit: "有耐心爱心、能接受倒班辛苦、求稳定者。" },
  { name: "药学", cat: "医学", tag: "需深造", tagType: "gold",
    fan: "进药企、白领、稳定高薪。",
    real: "本科药学就业一般，研发好岗位要硕博；销售岗多、研发岗少。注意区分“药学/临床药学/制药工程”。",
    plan: "多数要读研(药剂/药理/药物化学)；或走医院药师(考证)、药品监管考公。",
    pros: "医药行业长期向好，硕博进头部药企/医院药师前景好；纯本科普通。",
    fit: "化学生物基础好、愿深造者。" },
  // —— 财经管理篇 ——
  { name: "金融学", cat: "财经", tag: "极度分层", tagType: "gold",
    fan: "华尔街、操盘、年薪百万。",
    real: "光鲜的投行/券商研究只要顶尖院校硕士＋资源；普通本科金融多去银行柜员、销售、保险，与想象差距巨大。",
    plan: "本科只是起点，几乎必须读研(冲名校)；考 CFA/CPA；积累实习与人脉；定线(卖方/买方/银行/考公)。",
    pros: "天花板高但极度分层；非名校非硕士，回报远低于预期。",
    fit: "名校平台、家庭资源或极强自驱者；劝退“以为读金融就能进投行”的普通考生。" },
  { name: "会计 / 财务管理", cat: "财经", tag: "两极分化", tagType: "gold",
    fan: "铁饭碗、越老越吃香、考个 CPA 年薪百万。",
    real: "基础财务岗饱和、薪资低、易被财务共享/AI 替代；CPA 难考且非万能；“越老越吃香”只对高端(合伙人/CFO)成立。",
    plan: "必须考 CPA/CMA 等证书做区分；早进事务所积累或走管理会计；考公财政/税务也是好出路。",
    pros: "顶端(事务所/上市公司财务管理)稳定高薪，底端内卷＋替代风险高。",
    fit: "细心、能考证、目标明确者。" },
  { name: "工商管理 / 市场营销", cat: "财经", tag: "宽泛·空", tagType: "red",
    fan: "当管理者、做管理层、CEO 摇篮。",
    real: "本科没有“管理”岗，管理是干出来的不是学出来的；课程宽泛空泛、可替代性强；市场营销常被等同于销售。",
    plan: "考研细化(读 MBA 需先工作)，或辅修一门硬技能(数据/财务/编程)；早实习积累具体能力。",
    pros: "名校平台＋硬技能者有发展；纯本科“管理学”易空泛难就业。",
    fit: "有资源/平台、外向能跑业务者；劝退“以为读了就能当领导”的人。" },
  { name: "经济学", cat: "财经", tag: "偏理论", tagType: "gold",
    fan: "懂经济、进银行券商、高大上。",
    real: "偏理论、数学要求高；本科就业方向与金融重叠且更虚；好出路同样要读研＋名校＋实习。报考门槛：要么有分、要么家里有钱、要么考公。",
    plan: "打牢数学与计量；读研(经济/金融/数据)；或走考公(财经类岗位多)。",
    pros: "名校深造或考公前景好；普通本科泛理论、竞争力弱。",
    fit: "数理好、爱思辨、目标深造或考公者。" },
  // —— 人文社科 / 艺术 / 理基篇 ——
  { name: "法学", cat: "人文社科", tag: "连年红牌", tagType: "red",
    fan: "律政精英、西装、高薪。",
    real: "毕业生供给远超优质岗位；法考是硬门槛且通过率低；红圈所只要五院四系/名校；普通院校法学就业差、大量转行。",
    plan: "以通过法考为底线；尽量进名校或考研；早实习、定方向(诉讼/非诉/考公)。",
    pros: "对口考公岗位多(法检、公务员)，名校＋法考＋名所路径前景好；“普通学校法学”是典型高风险。",
    fit: "擅记忆与逻辑表达、能啃法条、目标明确者。" },
  { name: "新闻传播 / 广告", cat: "人文社科", tag: "红牌常客", tagType: "red",
    fan: "当记者、媒体人、光鲜自由。",
    real: "传统媒体萎缩、岗位锐减；新媒体门槛低、可替代性强；薪资偏低、加班多。",
    plan: "在校就产出作品集、做自媒体/实习，练真本事(拍剪写运营)；名校光环帮助大。",
    pros: "会内容＋会运营＋会数据者在互联网/品牌方仍有机会；“读了新闻就能当大记者”是幻想。",
    fit: "表达欲强、有内容创作天赋与执行力者。" },
  { name: "英语 / 翻译", cat: "人文社科", tag: "AI 冲击", tagType: "gold",
    fan: "当翻译、外企白领、走遍世界。",
    real: "英语正从“专业”退化为“人人必备的工具”；翻译岗受 AI 冲击大；纯语言对口岗减少、薪资走低。",
    plan: "必须“英语＋第二技能”(法律/经贸/计算机/小语种)；考教师编、外贸、考公(外事)；练口译等高端能力。",
    pros: "高端口译/小语种＋复合背景者仍有空间；纯英语前景趋弱。",
    fit: "语言天赋强且愿叠加第二专业者。" },
  { name: "心理学", cat: "人文社科", tag: "需深造", tagType: "gold",
    fan: "当心理咨询师、看透人心、高端职业。",
    real: "本科基本无法独立做咨询；国内咨询市场不成熟、收入不稳；学术路线要读博；多数转行 HR/教育/用研。",
    plan: "想做咨询要读研＋长期督导积累；或转向用户研究、HR、教育等应用方向；早定“学术 or 应用”。",
    pros: "用户研究、教育、HR 等应用方向有空间；纯咨询创业难。",
    fit: "真有兴趣并准备深造者；劝退“觉得有趣就随便选”的人。" },
  { name: "师范 / 汉语言文学", cat: "人文社科", tag: "求稳之选", tagType: "green",
    fan: "当老师、稳定、有寒暑假。",
    real: "教师考编竞争激烈(非师范也能考)；非公费师范无包分配；大城市编制极难、岗位多在县域；汉语言“万金油”对口有限。",
    plan: "考教师资格证＋考编是核心；尽量考公费师范或名校；接受地域下沉。",
    pros: "稳定、社会认可，适合求安稳者；公费师范/部属师范前景好。",
    fit: "真心想当老师、求稳定、能接受地域下沉者。" },
  { name: "设计学 / 美术", cat: "人文社科", tag: "投入高·内卷", tagType: "gold",
    fan: "当设计师、自由有创意、接单赚钱。",
    real: "艺考＋学费投入大；行业内卷、甲方虐、加班多、薪资两极；UI/平面同质化严重，易被模板和 AI 冲击。",
    plan: "死磕作品集与专精方向(交互/工业/三维/品牌)；积累真实项目；培养审美＋软件＋沟通综合力。",
    pros: "头部＋专精＋有商业理解者收入高；普通设计师替代性强。",
    fit: "真有天赋热爱、能持续产出作品、抗压者。" },
  { name: "数学 / 基础理科(强基)", cat: "人文社科", tag: "深造跳板", tagType: "gold",
    fan: "数学是万能钥匙、好就业。",
    real: "本科数学/物理直接就业窄，本质是“深造跳板”：读研转金融/计算机/AI 才有出路；强基计划锁定基础学科、转专业受限，需想清楚。",
    plan: "明确“以数学为基底深造转应用”路径；尽早定转码/金工/科研方向；强基生要接受长期学术路线。",
    pros: "转入 AI/金融/计算机后竞争力极强；坚持纯科研者顶尖院校前景好；纯本科留本行窄。",
    fit: "数理天赋拔尖、目标深造、想清楚再报强基者。" },
  { name: "农学", cat: "人文社科", tag: "政策友好", tagType: "green",
    fan: "田园、轻松、没前途(常被误解)。",
    real: "实则考公考编友好(选调/三支一扶/农业农村部门)、考研容易、分数性价比高；但市场化岗位薪资一般、部分要下基层。",
    plan: "走考公考编/选调/科研深造；或进种业/食品/农业科技企业；用好涉农政策红利。",
    pros: "乡村振兴＋粮食安全长期利好，体制内出路稳，适合求稳。",
    fit: "分数有限想稳上岸体制、能接受涉农基层者。" },

  // —— 名不副实篇（资料库《最具欺骗性的10大专业》直采，望文生义最易踩坑）——
  { name: "信息与计算科学", cat: "名不副实", tag: "其实是数学", tagType: "gold",
    fan: "听着像培养程序员、就是计算机/IT。",
    real: "实为数学学科，与计算机差很远——是“数学＋信息＋管理”交叉，核心课是数值代数/数值逼近等。",
    plan: "想进 IT 需自学编程或读研转向；天然适合走数学/金融数据/教研。",
    pros: "数理扎实，转金融/软件/教研有路；纯本科对口窄。",
    fit: "冲着“计算机”三个字报的最该当心。" },
  { name: "生物医学工程", cat: "名不副实", tag: "工科非医", tagType: "gold",
    fan: "以为学医、将来当医生。",
    real: "不归医学管，是不折不扣的工科（医疗器械/电子/信号/计算机）。",
    plan: "走医疗仪器研发/影像设备/临床工程，读研更稳。",
    pros: "高端医疗器械国产化，前景不错。",
    fit: "想当医生的别选它；爱工科又关注医疗的合适。" },
  { name: "精算（精算数学）", cat: "名不副实", tag: "金融非数学", tagType: "gold",
    fan: "以为是数学专业。",
    real: "属金融保险学科，精算师门槛极高、就业薪资亮眼，在银行/保险/投资/监管做风险定价。",
    plan: "考精算师系列证书，数理＋英语都要强，是长跑。",
    pros: "顶端稀缺、收入高、前程可观。",
    fit: "数理强且能长期考证者。" },
  { name: "信息资源管理", cat: "名不副实", tag: "图书档案", tagType: "red",
    fan: "听着像计算机/大数据。",
    real: "另一个名字是“图书情报与档案管理”，因招生不好才改了高大上的名，对口多是图书管理员。",
    plan: "谨慎报考；走名校＋考公(档案/图情)尚可，否则就业弱。",
    pros: "名校＋体制内出路尚可；普通院校前景差。",
    fit: "望文生义最易踩的坑，看清再报。" },
  { name: "汉语国际教育", cat: "名不副实", tag: "更考英语", tagType: "gold",
    fan: "以为专研汉语、轻松好学。",
    real: "更要求英语好，主攻对外汉语教学；国内不吃香，出路多在涉外/出国。",
    plan: "走国际中文教师/出国教学，或转传播/外贸。",
    pros: "国际中文热＋复合背景者有空间。",
    fit: "英语好、愿涉外、想教学者。" },
  { name: "地球物理学", cat: "名不副实", tag: "野外勘探", tagType: "gold",
    fan: "以为出入高端实验室搞地球科学。",
    real: "介于物理/地质/大气，毕业多去矿产能源勘探、工程与环境探测，常驻野外。",
    plan: "读研深造或进能源/勘探院所。",
    pros: "资源能源领域稳定、刚需。",
    fit: "能接受野外、真爱地学者。" },
  { name: "基础医学", cat: "名不副实", tag: "科研非临床", tagType: "gold",
    fan: "看到“医学”以为能当临床医生。",
    real: "研究生命与疾病本质，毕业去医学院校/科研机构做研究，本科不当临床医生。",
    plan: "坚定读研读博走科研路线。",
    pros: "医学科研、教研前景好。",
    fit: "爱临床别选、爱科研就选。" },
  { name: "数学与应用数学", cat: "名不副实", tag: "深造跳板", tagType: "gold",
    fan: "以为很难就业。",
    real: "是连接各学科的桥梁，本科窄但深造后可转金融/计算机/编程，很多去大厂写程序。",
    plan: "以数学为底深造转应用，尽早定方向。",
    pros: "转码/金工竞争力极强。",
    fit: "数理拔尖、目标深造者。" },
  { name: "哲学", cat: "名不副实", tag: "考公金饭碗", tagType: "green",
    fan: "以为是找不到工作的专业。",
    real: "毕业多去研究所/国家机关，考公友好，是“家长眼中的金饭碗”。",
    plan: "走考公/读研/教研/出版。",
    pros: "考公稳、就业并不差。",
    fit: "爱思辨、目标考公考编者。" },
  { name: "数字媒体 · 艺术 vs 技术", cat: "名不副实", tag: "一字之差两条路", tagType: "gold",
    fan: "以为“数字媒体艺术/技术”是同一个专业。",
    real: "艺术＝美工设计(动漫/游戏/动画)，技术＝编程开发(VR/图形学/特效)，差别极大、招生方式也不同。",
    plan: "看清是艺考美术还是工科编程再报志愿。",
    pros: "技术方向偏 IT 更稳；艺术方向看作品与天赋。",
    fit: "别只看名字，认准学位与课程表。" },
];

const CATS = ["工科", "医学", "财经", "人文社科", "名不副实"];
const activeCat = ref("全部");

const list = computed(() =>
  activeCat.value === "全部" ? TRUTHS : TRUTHS.filter((t) => t.cat === activeCat.value)
);

const ROWS = [
  { key: "fan", lab: "幻想", cls: "fan" },
  { key: "real", lab: "真相", cls: "real" },
  { key: "plan", lab: "规划", cls: "plan" },
  { key: "pros", lab: "前景", cls: "pros" },
  { key: "fit", lab: "适合", cls: "fit" },
] as const;
</script>

<template>
  <div class="page">
    <header class="ph">
      <div class="eyebrow">★ 反幻想护栏 · 专业真相栏</div>
      <h1>专业真相栏</h1>
      <p>不替你造梦，也不贩卖焦虑——只把“招生话术里的光鲜”和“毕业五年后的真实出路”摆在一起。</p>
    </header>

    <div class="cat-tabs">
      <button :class="{ on: activeCat === '全部' }" @click="activeCat = '全部'">全部</button>
      <button v-for="c in CATS" :key="c" :class="{ on: activeCat === c }" @click="activeCat = c">{{ c }}</button>
    </div>

    <div class="count-line">共 {{ list.length }} 张真相卡 · 证据源自载望升学资料（专业篇/就业方向篇/职业规划篇/答疑篇）</div>

    <div class="card-grid">
      <article v-for="t in list" :key="t.name" class="truth">
        <div class="th">
          <span class="th-name">{{ t.name }}</span>
          <span class="pill" :class="t.tagType">{{ t.tag }}</span>
        </div>
        <div class="bd">
          <p v-for="r in ROWS" :key="r.key">
            <span class="k" :class="r.cls">{{ r.lab }}</span>{{ (t as any)[r.key] }}
          </p>
        </div>
      </article>
    </div>

    <div class="three-truth">
      <div class="tt-card"><h4>① 别被名字骗</h4><p>“生物医学工程”不是医生、“信息与计算科学”是数学、“精算”门槛极高——名字光鲜 ≠ 出路光鲜。先看真实就业，再谈兴趣。</p></div>
      <div class="tt-card"><h4>② 看清“要不要读研”</h4><p>很多专业的真相是本科只是半成品(医学、生物、心理、金融、药学、数学)。报考时就要算清：你和家庭能否支撑这条长路？</p></div>
      <div class="tt-card"><h4>③ 兴趣要有证据</h4><p>“我喜欢”常是幻想，“我做过/查过/能坚持”才是兴趣——没有证据的热爱，撑不过四年硬课。</p></div>
    </div>

    <div class="warn-band">
      <b>“生化环材”四大天坑 + 红牌预警：</b>生物、化学、材料、环境本科直接就业普遍偏弱，默认按“要读研读博”或“早做转行规划”对待；法学、新闻传播、绘画、应用心理学、工商管理等是近年红牌常客。系统会在冲稳保里选到这些专业时自动浮出真相卡二次提醒，但绝不替你否决——讲清代价，把选择权留给你。
    </div>
  </div>
</template>

<style scoped>
.page { height: 100vh; overflow-y: auto; max-width: 1000px; margin: 0 auto; padding: 30px 28px 60px; }
.eyebrow { font-family: var(--mono); font-size: 11px; color: var(--gold-deep); font-weight: 700; letter-spacing: .2em; }
.ph h1 { font-family: var(--serif); font-size: 27px; margin: 6px 0 0; color: var(--ink); }
.ph p { color: var(--text-2); font-size: 13.5px; margin: 8px 0 20px; max-width: 760px; }

.cat-tabs { display: flex; gap: 8px; margin-bottom: 14px; }
.cat-tabs button { border: 1px solid var(--border); background: var(--panel); border-radius: 999px; padding: 6px 16px; font-size: 13px; color: var(--text-2); font-weight: 600; }
.cat-tabs button.on { background: var(--grad); color: #fff; border-color: transparent; box-shadow: 0 4px 12px -4px rgba(210,58,44,.5); }

.count-line { font-size: 12px; color: var(--muted); margin-bottom: 14px; }

.card-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 14px; }
.truth { background: var(--panel); border: 1px solid var(--border); border-radius: 14px; overflow: hidden; display: flex; flex-direction: column; box-shadow: var(--shadow-sm); transition: transform .18s, box-shadow .18s; }
.truth:hover { transform: translateY(-2px); box-shadow: var(--shadow-lg); }
.th { padding: 11px 15px; background: var(--bg-soft); font-weight: 800; color: var(--ink); display: flex; justify-content: space-between; align-items: center; gap: 8px; border-bottom: 1px solid var(--border); }
.th-name { font-size: 14.5px; font-family: var(--serif); }
.pill { font-size: 10px; font-weight: 700; padding: 2px 9px; border-radius: 999px; white-space: nowrap; border: 1px solid transparent; }
.pill.red { background: #fde6e3; color: var(--red-deep); border-color: #f2c1ba; }
.pill.gold { background: var(--gold-soft); color: var(--gold-deep); border-color: #ecd5a3; }
.pill.green { background: var(--green-soft); color: var(--green); border-color: #c4e2cf; }
.bd { padding: 12px 15px; font-size: 12.8px; line-height: 1.7; flex: 1; color: var(--text-2); }
.bd p { margin: 6px 0; }
.bd .k { display: inline-block; min-width: 38px; font-weight: 800; margin-right: 6px; }
.bd .k.fan { color: var(--gold-deep); }
.bd .k.real { color: var(--red-deep); }
.bd .k.plan { color: var(--blue); }
.bd .k.pros { color: var(--green); }
.bd .k.fit { color: var(--purple); }

.three-truth { display: grid; grid-template-columns: repeat(3, 1fr); gap: 14px; margin-top: 24px; }
.tt-card { background: var(--panel); border: 1px solid var(--border); border-radius: 14px; padding: 16px 18px; box-shadow: var(--shadow-sm); }
.tt-card h4 { margin: 0 0 7px; color: var(--blue-deep); font-size: 14px; }
.tt-card p { margin: 0; font-size: 12.8px; color: var(--text-2); line-height: 1.75; }

.warn-band { margin-top: 18px; background: linear-gradient(135deg, #fdebe7, #fbe2dc); border: 1px solid #f3ccc4; border-left: 4px solid var(--red); border-radius: 12px; padding: 14px 18px; color: #8a2c22; font-size: 13px; line-height: 1.8; }
.warn-band b { color: var(--red-deep); }
@media (max-width: 820px) { .card-grid { grid-template-columns: 1fr; } .three-truth { grid-template-columns: 1fr; } }
</style>
