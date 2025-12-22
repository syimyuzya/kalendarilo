//! 格式化日期相關功能

/// 漢數字，第 `1..=9` 項分別為「一」到「九」。為便於格式化日期，第 0 項為「十」。
pub const NUM_CHINESE: &[&str] = &["十", "一", "二", "三", "四", "五", "六", "七", "八", "九"];

/// 干支序號轉為文本形式。
///
/// # 用例
///
/// ```
/// use kalendarilo::chinese;
///
/// assert_eq!("乙巳", chinese::fmt::sexagenary(42));
/// ```
pub fn sexagenary(num: u32) -> String {
    static NAME1: &[&str] = &["癸", "甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬"];
    static NAME2: &[&str] = &[
        "亥", "子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌",
    ];
    NAME1[num.rem_euclid(10) as usize].to_owned() + NAME2[num.rem_euclid(12) as usize]
}

/// 取得月名（含「月」字）。十一、十二月稱「冬月」「臘月」。
///
/// # 用例
///
/// ```
/// use kalendarilo::chinese::{self, Month::*};
///
/// assert_eq!("冬月", chinese::fmt::month(Common(11)));
/// assert_eq!("閏正月", chinese::fmt::month(Leap(1)));
/// ```
///
/// # Panics
///
/// 若月序號不在 `1..=12` 間則 panic。
pub fn month(m: super::Month) -> String {
    let mut rt = String::new();
    if m.is_leap() {
        rt += "閏";
    }
    let num = m.num();
    rt += match num {
        1 => "正",
        2..=9 => NUM_CHINESE[num as usize],
        10 => "十",
        11 => "冬",
        12 => "臘",
        _ => panic!("month {} not in 1..=12", num),
    };
    rt += "月";
    rt
}

/// 取得日名，前十日為「初一」到「初十」，第 21 至 29 日為「廿一」到「廿九」。
///
/// # 用例
///
/// ```
/// use kalendarilo::chinese;
///
/// assert_eq!("初十", chinese::fmt::day(10));
/// assert_eq!("廿五", chinese::fmt::day(25));
/// assert_eq!("三十", chinese::fmt::day(30));
/// ```
///
/// # Panics
///
/// 若日序號不在 `1..=30` 間則 panic。
pub fn day(d: u32) -> String {
    match d {
        1..=10 => "初",
        11..=19 => "十",
        20 => "二",
        21..=29 => "廿",
        30 => "三",
        _ => panic!("day {} not in 1..=30", d),
    }
    .to_owned()
        + NUM_CHINESE[(d % 10) as usize]
}

/// 節氣序號轉為名稱。`1..=24` 分別為立春到大寒。
///
/// # 用例
///
/// ```
/// use kalendarilo::chinese;
///
/// assert_eq!("穀雨", chinese::fmt::solar_term(6));
/// ```
pub fn solar_term(term: u32) -> &'static str {
    const NAMES: &[&str] = &[
        "大寒", "立春", "雨水", "驚蟄", "春分", "清明", "穀雨", "立夏", "小滿", "芒種", "夏至",
        "小暑", "大暑", "立秋", "處暑", "白露", "秋分", "寒露", "霜降", "立冬", "小雪", "大雪",
        "冬至", "小寒",
    ];
    NAMES[term.rem_euclid(24) as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sexagenary() {
        for (std, num) in [("甲子", 1), ("庚寅", 27), ("癸亥", 60)] {
            assert_eq!(std, sexagenary(num));
        }
    }

    #[test]
    fn test_day() {
        for (std, d) in [
            ("初一", 1),
            ("初十", 10),
            ("十一", 11),
            ("二十", 20),
            ("廿一", 21),
            ("三十", 30),
        ] {
            assert_eq!(std, day(d));
        }
    }
}
