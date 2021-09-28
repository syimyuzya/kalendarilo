//! 月相節氣曆表數據
//!
//! [數據取自該 Github 項目](https://github.com/ytliu0/ChineseCalendar)。

use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::sync::Once;

use crate::time_scales::Tdb;

/// 保存一歲的曆表數據
#[derive(Debug)]
pub struct Annus {
    /// 序號，為該歲大部分時段所在公元年
    pub annus: i32,
    /// 從冬至開始的各節氣時刻，亦含次歲冬至以便計算末日
    pub solar_term: [Tdb; 25],
    /// 月相時刻，列出從冬至前一朔開始的十五個月，內層 `0..=3` 分別為朔、上弦、望、下弦
    pub moon_phase: [[Tdb; 4]; 15],
}

static mut DATA: Vec<Annus> = Vec::new();
static INIT: Once = Once::new();

impl Annus {
    /// 取得公元 `annus` 年對應的歳的曆表。
    ///
    /// 無數據則返回 `None`。
    pub fn get(annus: i32) -> Option<&'static Self> {
        INIT.call_once(|| {
            let res = parse_raw_data()
                .unwrap_or_else(|e| panic!("error parsing ephemeris data: {:?}", e));
            unsafe {
                DATA = res;
            }
        });
        unsafe {
            DATA.binary_search_by_key(&annus, |an| an.annus)
                .ok()
                .map(|i| &DATA[i])
        }
    }
}

static RAW_DATA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/TDBtimes.txt"));

fn parse_raw_data() -> Result<Vec<Annus>, RawDataError> {
    let mut res = Vec::new();
    for (line_num, line) in (1usize..).zip(RAW_DATA.lines()).skip(1) {
        let mut it = line.split_whitespace();
        let annus: i32 = match it.next() {
            None => continue,
            Some(s) => s
                .parse()
                .map_err(|e| RawDataError::new(line_num, 1, ErrorType::InvalidInt(e)))?,
        };
        // XXX dumtempe limita por pli rapida pravalorizado
        if !(1970..=2050).contains(&annus) {
            continue;
        }
        let jd0 = require_next_f64(&mut it, line_num, 2)?;
        let mut annus_rec = Annus {
            annus,
            solar_term: [Tdb(0.0); 25],
            moon_phase: [[Tdb(0.0); 4]; 15],
        };
        for i in 0..25 {
            let jd_diff = require_next_f64(&mut it, line_num, 3 + i)?;
            annus_rec.solar_term[i] = Tdb(jd0 + jd_diff);
        }
        for i in 0..15 {
            for j in 0..4 {
                let jd_diff = require_next_f64(&mut it, line_num, 28 + i * 4 + j)?;
                annus_rec.moon_phase[i][j] = Tdb(jd0 + jd_diff);
            }
        }
        res.push(annus_rec);
    }
    Ok(res)
}

fn require_next_f64<I: Iterator<Item = &'static str>>(
    it: &mut I,
    line_num: usize,
    field_num: usize,
) -> Result<f64, RawDataError> {
    use ErrorType::*;
    it.next()
        .ok_or_else(|| RawDataError::new(line_num, field_num, MissingField))?
        .parse()
        .map_err(|e| RawDataError::new(line_num, field_num, InvalidFloat(e)))
}

#[derive(Debug)]
struct RawDataError {
    pub line_num: usize,
    pub field_num: usize,
    pub reason: ErrorType,
}

impl RawDataError {
    fn new(line_num: usize, field_num: usize, reason: ErrorType) -> Self {
        Self {
            line_num,
            field_num,
            reason,
        }
    }
}

#[derive(Debug)]
enum ErrorType {
    InvalidInt(ParseIntError),
    InvalidFloat(ParseFloatError),
    MissingField,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::date::Date;
    use crate::time_scales::Ut;

    fn date_cst(tdb: Tdb) -> Date {
        Ut::convert(tdb).date_in_timezone(480)
    }

    #[test]
    fn solar_terms() {
        let annus = Annus::get(2000).unwrap();
        let date = date_cst(annus.solar_term[0]);
        assert_eq!("1999-12-22", date.iso_gregorian());
        let date = date_cst(annus.solar_term[24]);
        assert_eq!("2000-12-21", date.iso_gregorian());
    }

    #[test]
    fn new_moons() {
        let annus = Annus::get(2000).unwrap();
        let date = date_cst(annus.moon_phase[0][0]);
        assert_eq!("1999-12-08", date.iso_gregorian());
    }
}
