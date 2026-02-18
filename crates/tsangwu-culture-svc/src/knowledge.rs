use serde::Serialize;

/// 朝代信息
#[derive(Debug, Clone, Serialize)]
pub struct DynastyInfo {
    pub name: &'static str,
    pub start_year: i32,
    pub end_year: i32,
    pub keywords: &'static [&'static str],
}

/// 风格条目
#[derive(Debug, Clone, Serialize)]
pub struct StyleEntry {
    pub name: &'static str,
    pub keywords: &'static [&'static str],
    pub description: &'static str,
}

/// 校验规则
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub id: &'static str,
    pub category: &'static str,
    pub dynasty: &'static str,
    pub forbidden: &'static [&'static str],
    pub message: &'static str,
}

/// 模拟素材条目
#[derive(Debug, Clone, Serialize)]
pub struct AssetEntry {
    pub id: &'static str,
    pub name: &'static str,
    pub tags: &'static [&'static str],
    pub dynasty: Option<&'static str>,
    pub asset_type: &'static str,
}

/// 内置朝代列表
pub const DYNASTIES: &[DynastyInfo] = &[
    DynastyInfo { name: "秦", start_year: -221, end_year: -206, keywords: &["秦朝", "秦代", "始皇", "兵马俑", "长城"] },
    DynastyInfo { name: "汉", start_year: -206, end_year: 220, keywords: &["汉朝", "汉代", "西汉", "东汉", "汉服"] },
    DynastyInfo { name: "唐", start_year: 618, end_year: 907, keywords: &["唐朝", "唐代", "盛唐", "大唐", "唐装", "唐三彩"] },
    DynastyInfo { name: "宋", start_year: 960, end_year: 1279, keywords: &["宋朝", "宋代", "北宋", "南宋", "宋瓷"] },
    DynastyInfo { name: "元", start_year: 1271, end_year: 1368, keywords: &["元朝", "元代", "蒙古", "元青花"] },
    DynastyInfo { name: "明", start_year: 1368, end_year: 1644, keywords: &["明朝", "明代", "大明", "明式家具"] },
    DynastyInfo { name: "清", start_year: 1644, end_year: 1912, keywords: &["清朝", "清代", "大清", "旗装", "马褂", "旗袍"] },
];

/// 风格词典
pub const STYLES: &[StyleEntry] = &[
    StyleEntry { name: "工笔", keywords: &["工笔", "工笔画", "细笔"], description: "精细写实的传统绘画技法" },
    StyleEntry { name: "写意", keywords: &["写意", "写意画", "泼墨"], description: "注重意境的自由绘画风格" },
    StyleEntry { name: "水墨", keywords: &["水墨", "水墨画", "墨色"], description: "以墨色浓淡表现的绘画" },
    StyleEntry { name: "青绿山水", keywords: &["青绿", "青绿山水", "金碧"], description: "以石青石绿为主色的山水画" },
    StyleEntry { name: "白描", keywords: &["白描", "线描"], description: "纯用线条勾勒的绘画技法" },
    StyleEntry { name: "没骨", keywords: &["没骨", "没骨画"], description: "不用墨线勾勒直接用色彩渲染" },
];

/// 校验规则集：朝代-服饰/器物匹配
pub const VALIDATION_RULES: &[ValidationRule] = &[
    ValidationRule {
        id: "costume-tang-no-qing",
        category: "服饰",
        dynasty: "唐",
        forbidden: &["马褂", "旗装", "旗袍", "长袍马褂"],
        message: "唐代不应出现清代服饰（马褂/旗装）",
    },
    ValidationRule {
        id: "costume-song-no-qing",
        category: "服饰",
        dynasty: "宋",
        forbidden: &["马褂", "旗装", "旗袍"],
        message: "宋代不应出现清代服饰",
    },
    ValidationRule {
        id: "costume-han-no-qing",
        category: "服饰",
        dynasty: "汉",
        forbidden: &["马褂", "旗装", "旗袍", "唐装"],
        message: "汉代不应出现后世服饰",
    },
    ValidationRule {
        id: "prop-tang-no-yuanqinghua",
        category: "器物",
        dynasty: "唐",
        forbidden: &["元青花", "青花瓷"],
        message: "唐代不应出现元代青花瓷",
    },
    ValidationRule {
        id: "prop-song-no-yuanqinghua",
        category: "器物",
        dynasty: "宋",
        forbidden: &["元青花"],
        message: "宋代不应出现元代青花瓷",
    },
    ValidationRule {
        id: "color-ming-no-yellow",
        category: "颜色禁忌",
        dynasty: "明",
        forbidden: &["明黄", "正黄"],
        message: "明代明黄色为皇家专用，平民场景不宜使用",
    },
    ValidationRule {
        id: "color-qing-no-yellow",
        category: "颜色禁忌",
        dynasty: "清",
        forbidden: &["明黄", "正黄"],
        message: "清代明黄色为皇家专用，平民场景不宜使用",
    },
];

/// 模拟素材库
pub const ASSET_LIBRARY: &[AssetEntry] = &[
    AssetEntry { id: "asset-001", name: "唐代仕女图背景", tags: &["唐", "仕女", "工笔", "人物"], dynasty: Some("唐"), asset_type: "background" },
    AssetEntry { id: "asset-002", name: "宋代山水长卷", tags: &["宋", "山水", "水墨", "长卷"], dynasty: Some("宋"), asset_type: "background" },
    AssetEntry { id: "asset-003", name: "明代园林场景", tags: &["明", "园林", "建筑", "青绿"], dynasty: Some("明"), asset_type: "background" },
    AssetEntry { id: "asset-004", name: "清代宫廷内景", tags: &["清", "宫廷", "建筑", "工笔"], dynasty: Some("清"), asset_type: "background" },
    AssetEntry { id: "asset-005", name: "汉服女装模板", tags: &["汉", "汉服", "服饰", "女装"], dynasty: Some("汉"), asset_type: "template" },
    AssetEntry { id: "asset-006", name: "唐装男装模板", tags: &["唐", "唐装", "服饰", "男装"], dynasty: Some("唐"), asset_type: "template" },
    AssetEntry { id: "asset-007", name: "水墨笔刷效果", tags: &["水墨", "笔刷", "效果"], dynasty: None, asset_type: "effect" },
    AssetEntry { id: "asset-008", name: "青绿山水色调", tags: &["青绿", "山水", "色调"], dynasty: None, asset_type: "effect" },
    AssetEntry { id: "asset-009", name: "工笔花鸟素材", tags: &["工笔", "花鸟", "素材"], dynasty: None, asset_type: "element" },
    AssetEntry { id: "asset-010", name: "古典建筑构件", tags: &["建筑", "构件", "古典"], dynasty: None, asset_type: "element" },
];

/// 文化元素关键词（用于文本扫描）
pub const ELEMENT_KEYWORDS: &[(&str, &str)] = &[
    ("服饰", "汉服"), ("服饰", "唐装"), ("服饰", "旗袍"), ("服饰", "马褂"),
    ("服饰", "旗装"), ("服饰", "长袍"), ("服饰", "凤冠"), ("服饰", "霞帔"),
    ("器物", "青花瓷"), ("器物", "元青花"), ("器物", "唐三彩"), ("器物", "宋瓷"),
    ("器物", "玉器"), ("器物", "铜镜"), ("器物", "香炉"), ("器物", "屏风"),
    ("建筑", "亭台"), ("建筑", "楼阁"), ("建筑", "园林"), ("建筑", "宫殿"),
    ("建筑", "牌坊"), ("建筑", "石桥"), ("建筑", "庙宇"), ("建筑", "塔"),
    ("自然", "山水"), ("自然", "竹林"), ("自然", "梅花"), ("自然", "荷花"),
    ("自然", "松柏"), ("自然", "云雾"),
];
