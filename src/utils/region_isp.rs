// region code列表
pub const REGION_CODE_LIST: [&str; 38] = [
    "china",
    "global",
    "oversea",
    "shandong",
    "jiangsu",
    "zhejiang",
    "anhui",
    "shanghai",
    "fujian",
    "jiangxi",
    "guangdong",
    "guangxi",
    "hainan",
    "henan",
    "hunan",
    "hubei",
    "beijing",
    "tianjin",
    "hebei",
    "shanxi",
    "neimenggu",
    "ningxia",
    "qinghai",
    "gansu",
    "shaanxi",
    "sichuan",
    "guizhou",
    "xinjiang",
    "yunnan",
    "chongqing",
    "xizang",
    "liaoning",
    "jilin",
    "heilongjiang",
    "hongkong",
    "macau",
    "taiwan",
    "unknown",
];

// region名称列表
pub const REGION_NAME_LIST: [&str; 38] = [
    "中国",
    "全球",
    "海外",
    "山东",
    "江苏",
    "浙江",
    "安徽",
    "上海",
    "福建",
    "江西",
    "广东",
    "广西",
    "海南",
    "河南",
    "湖南",
    "湖北",
    "北京",
    "天津",
    "河北",
    "山西",
    "内蒙古",
    "宁夏",
    "青海",
    "甘肃",
    "陕西",
    "四川",
    "贵州",
    "新疆",
    "云南",
    "重庆",
    "西藏",
    "辽宁",
    "吉林",
    "黑龙江",
    "香港",
    "澳门",
    "台湾",
    "未知地区",
];

pub fn get_region_name_from_code(region_code: &str) -> &str {
    for (i, r) in REGION_CODE_LIST.iter().enumerate() {
        if *r == region_code {
            return REGION_NAME_LIST.get(i).unwrap();
        }
    }
    "未知地区"
}

// 运营商代码
pub const ISP_CODES: [&str; 8] = [
    "all", "telecom", "unicom", "mobile", "drpeng", "tietong", "cernet", "others",
];

// 运营商名称
pub const ISP_NAME: [&str; 8] = [
    "全部",
    "电信",
    "联通",
    "移动",
    "鹏博士",
    "铁通",
    "教育网",
    "其他",
];

pub fn get_isp_name_from_code(isp_code: &str) -> &str {
    for (i, r) in ISP_CODES.iter().enumerate() {
        if *r == isp_code {
            return ISP_NAME.get(i).unwrap();
        }
    }
    "其他"
}
