//! Chinese calendar
//!
//! Note: 為方便處理諸多術語，本模塊文檔用中文。
//!
//! 本程序採用預製好的天文曆表資料編算夏曆，見 [`ephemeris`]。

use crate::date::Date;
use crate::time_scales::{Tdb, Ut};

pub mod ephemeris;
pub mod fmt;

/// 「歲」，相鄰兩冬至間的時段，或自冬至所在月（十一月）至下一冬至前月（十月或閏十月）的時段。
///
/// 支持的年份取決於曆表數據，見 [`ephemeris`]。
///
/// 注意：「歲」與「年」在曆法上不同，年以正月為首，但曆法編算須以兩冬至間的「歲」為基礎，本程序亦以「歲」編排，並依日期計算所在「年」。
///
/// # 用例
///
/// ```
/// use kalendarilo::Date;
/// use kalendarilo::chinese::{Annus, Month::*};
///
/// let date = Date::from_gregorian(2000, 1, 1).unwrap();
/// let annus = Annus::from_date(date).unwrap();
///
/// assert_eq!(Ok((1999, Common(11), 25)), annus.ymd_for(date));
/// ```
#[derive(Debug, Clone)]
pub struct Annus {
    /// 序號，為該歲大部分時段所在的公元年
    pub annus: i32,
    /// 該歲的曆表
    pub ephemeris: &'static ephemeris::Annus,
    /// 全部月首，包括次一歲首月用以標記本歲最末日
    pub months: Vec<NewMoon>,
}
/// 月首信息
#[derive(Debug, Copy, Clone)]
pub struct NewMoon {
    /// 月名
    pub month: Month,
    /// 月首所在日期
    pub date: Date,
}
/// 月名，`Common` 為平月，`Leap` 為閏月。
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Month {
    Common(u32),
    Leap(u32),
}
impl Month {
    /// 取得月序號，無論平閏。
    pub fn num(&self) -> u32 {
        use Month::*;
        *match self {
            Common(v) | Leap(v) => v,
        }
    }
    /// 閏月為 `true`，平月為 `false`
    pub fn is_leap(&self) -> bool {
        matches!(self, Self::Leap(_))
    }
    /// 取得月名的文本形式，十一、十二月稱「冬月」「臘月」。
    pub fn name(&self) -> String {
        fmt::month(*self)
    }
}

impl Annus {
    /// 取得與公元 `annus` 年對應的歲。
    ///
    /// 若曆表無該歲資料則返回 `None`。
    ///
    /// # 用例
    ///
    /// ```
    /// use kalendarilo::chinese::Annus;
    ///
    /// let annus = Annus::new(2000).unwrap();
    /// ```
    pub fn new(annus: i32) -> Option<Self> {
        use Month::*;

        let ephemeris = ephemeris::Annus::get(annus)?;
        let new_moon_dates: Vec<_> = ephemeris
            .moon_phase
            .iter()
            .map(|arr| date_cst(arr[0]))
            .collect();
        let ws = date_cst(ephemeris.solar_term[0]);
        let ws_next = date_cst(ephemeris.solar_term[24]);
        let m11_idx = new_moon_dates.partition_point(|date| date <= &ws) - 1;
        let m11n_idx = new_moon_dates.partition_point(|date| date < &ws_next) - 1;
        let mut needs_leap = match m11n_idx - m11_idx {
            12 => false,
            13 => true,
            _ => panic!("{} months between winter solstices", m11n_idx - m11_idx),
        };

        let mut months = Vec::with_capacity(m11n_idx - m11_idx);
        let mut month = 10;
        let mut term = 0;
        for i in m11_idx..=m11n_idx {
            if needs_leap && new_moon_dates[i + 1] <= date_cst(ephemeris.solar_term[term]) {
                months.push(NewMoon {
                    month: Leap(month),
                    date: new_moon_dates[i],
                });
                needs_leap = false;
                continue;
            }
            month = month % 12 + 1;
            months.push(NewMoon {
                month: Common(month),
                date: new_moon_dates[i],
            });
            term += 2;
        }
        assert!(!needs_leap);

        Some(Annus {
            annus,
            ephemeris,
            months,
        })
    }
    /// 依特定日期取得其所在歲。
    ///
    /// 若曆表無該歲資料則返回 `None`。
    ///
    /// # 用例
    ///
    /// ```
    /// use kalendarilo::Date;
    /// use kalendarilo::chinese::Annus;
    ///
    /// let date = Date::from_gregorian(1999, 12, 31).unwrap();
    /// let annus = Annus::from_date(date).unwrap();
    ///
    /// assert_eq!(2000, annus.annus);
    /// ```
    pub fn from_date(date: Date) -> Option<Self> {
        let mut y = date.gregorian().0;
        loop {
            let annus = Self::new(y)?;

            let start = annus.months[0].date;
            let end = annus.months.last().unwrap().date;

            if (start..end).contains(&date) {
                return Some(annus);
            }

            y += if date < start { -1 } else { 1 };
        }
    }

    /// 取得給定日期在該歲的年月日，返回格式為 `(年, 月, 日)`。
    ///
    /// 若所給日期不在該歲，則回報 `Err` 並指出該日期在該歲之前還是之後。
    ///
    /// # 用例
    ///
    /// ```
    /// use kalendarilo::Date;
    /// use kalendarilo::chinese::{Annus, Month::*};
    ///
    /// let date = Date::from_gregorian(2000, 1, 1).unwrap();
    /// let annus = Annus::from_date(date).unwrap();
    ///
    /// assert_eq!(Ok((1999, Common(11), 25)), annus.ymd_for(date));
    /// ```
    pub fn ymd_for(&self, date: Date) -> Result<(i32, Month, u32), OtherAnnus> {
        let begin = self.months[0].date;
        let end = self.months.last().unwrap().date;

        if date < begin {
            return Err(OtherAnnus::Before);
        } else if date >= end {
            return Err(OtherAnnus::After);
        }

        let m = self
            .months
            .iter()
            .take_while(|m| m.date <= date)
            .last()
            .unwrap();
        let d = date.jdn() - m.date.jdn() + 1;
        let y = if m.month.num() >= 11 {
            self.annus - 1
        } else {
            self.annus
        };
        Ok((y, m.month, d))
    }

    /// 取得給定日期所在節氣信息，若當日並無交節，則給出該日相對其前一個交節的日數差。返回值格式如下：
    ///
    /// - `.0`：取得的節氣所在歲（前一歲大雪可能落在該歲，故須回報所在歲）
    /// - `.1`：該節氣序號，1..=24 對應立春到大寒
    /// - `.2`：所給 `date` 在該節交節後第幾日，為 0 則表示當日交節
    ///
    /// 本方法支持自該歲首日至次歲冬至前日的區間。
    ///
    /// 若給定日期不在該歲，或曆表無法取得前一歲節氣數據，則回報 `Err`。
    ///
    /// # 用例
    ///
    /// ```
    /// use kalendarilo::Date;
    /// use kalendarilo::chinese::{Annus, Month::*};
    ///
    /// let date = Date::from_gregorian(2000, 1, 1).unwrap();
    /// let annus = Annus::from_date(date).unwrap();
    ///
    /// assert_eq!(Ok((2000, 22, 10)), annus.solar_term_for(date)); // 冬至過後第 10 天
    /// ```
    pub fn solar_term_for(&self, date: Date) -> Result<(i32, u32, u32), SolarTermErr> {
        use self::OtherAnnus::*;
        use SolarTermErr::*;
        if date < self.months[0].date {
            return Err(OtherAnnus(Before));
        } else if date >= date_cst(self.ephemeris.solar_term[24]) {
            return Err(OtherAnnus(After));
        }
        if date < date_cst(self.ephemeris.solar_term[0]) {
            let last_annus = ephemeris::Annus::get(self.annus - 1).ok_or(NoData)?;
            for (idx, &tdb) in (22..24).zip(&last_annus.solar_term[22..24]).rev() {
                let term_start = date_cst(tdb);
                if date >= term_start {
                    return Ok((self.annus - 1, (idx + 21) % 24 + 1, date - term_start));
                }
            }
            panic!("incorrect data for annus {}", self.annus - 1);
        }
        let idx = self.ephemeris.solar_term[..24].partition_point(|&tdb| date_cst(tdb) <= date) - 1;
        let off = date - date_cst(self.ephemeris.solar_term[idx]);
        Ok((self.annus, (idx as u32 + 21) % 24 + 1, off))
    }
}

/// 表示給定日期不在該歲，並指出其在前還是在後。
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OtherAnnus {
    Before,
    After,
}

/// 表示給定日期不在該歲，或曆表無法取得節氣數據。
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SolarTermErr {
    NoData,
    OtherAnnus(OtherAnnus),
}

/// 將給定曆表時間轉為北京時間（UTC+8）日期。
pub fn date_cst(tdb: Tdb) -> Date {
    Ut::convert(tdb).date_in_timezone(480)
}

/// 取得所給公元年的干支。
///
/// # 用例
///
/// ```
/// use kalendarilo::chinese::sexagenary_for_year;
///
/// assert_eq!(1, sexagenary_for_year(-2696));
/// ```
pub fn sexagenary_for_year(year: i32) -> u32 {
    (year.rem_euclid(60) as u32 + 2696) % 60 + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_date() {
        let dataset = [
            (2017, (2017, 1, 27)),
            (2017, (2017, 12, 17)),
            (2018, (2017, 12, 18)),
        ];
        for (std, (y, m, d)) in dataset {
            assert_eq!(
                Some(std),
                Annus::from_date(Date::from_gregorian(y, m, d).unwrap()).map(|a| a.annus)
            );
        }
    }

    #[test]
    fn months() {
        let annus = Annus::new(2000).unwrap();
        let stds = [
            (11, "1999-12-08"),
            (12, "2000-01-07"),
            (1, "2000-02-05"),
            (2, "2000-03-06"),
            (3, "2000-04-05"),
            (4, "2000-05-04"),
            (5, "2000-06-02"),
            (6, "2000-07-02"),
            (7, "2000-07-31"),
            (8, "2000-08-29"),
            (9, "2000-09-28"),
            (10, "2000-10-27"),
            (11, "2000-11-26"),
        ];
        assert_eq!(stds.len(), annus.months.len());
        for (std, month) in stds.iter().zip(&annus.months) {
            assert_eq!(Month::Common(std.0), month.month);
            assert_eq!(std.1, month.date.iso_gregorian());
        }
    }

    #[test]
    fn leap_months() {
        let stds = [
            (11, "2016-11-29"),
            (12, "2016-12-29"),
            (1, "2017-01-28"),
            (2, "2017-02-26"),
            (3, "2017-03-28"),
            (4, "2017-04-26"),
            (5, "2017-05-26"),
            (6, "2017-06-24"),
            (-6, "2017-07-23"),
            (7, "2017-08-22"),
            (8, "2017-09-20"),
            (9, "2017-10-20"),
            (10, "2017-11-18"),
            (11, "2017-12-18"),
        ];
        let annus = Annus::new(2017).unwrap();
        for (std, month) in stds.iter().zip(&annus.months) {
            let std_month = if std.0 > 0 {
                Month::Common(std.0 as u32)
            } else {
                Month::Leap(-std.0 as u32)
            };
            assert_eq!(
                (std_month, std.1.into()),
                (month.month, month.date.iso_gregorian())
            );
        }
        assert_eq!(stds.len(), annus.months.len());
    }

    #[test]
    fn dates() {
        use Month::*;
        use OtherAnnus::*;
        let data = [
            ((2016, 11, 29), Ok((2016, Common(11), 1))),
            ((2017, 1, 27), Ok((2016, Common(12), 30))),
            ((2017, 1, 28), Ok((2017, Common(1), 1))),
            ((2017, 7, 22), Ok((2017, Common(6), 29))),
            ((2017, 7, 23), Ok((2017, Leap(6), 1))),
            ((2017, 12, 17), Ok((2017, Common(10), 30))),
            ((2016, 11, 28), Err(Before)),
            ((2017, 12, 18), Err(After)),
        ];
        let annus = Annus::new(2017).unwrap();
        for ((y, m, d), std) in data {
            assert_eq!(std, annus.ymd_for(Date::from_gregorian(y, m, d).unwrap()));
        }
    }

    #[test]
    fn solar_terms() {
        use self::OtherAnnus::*;
        use SolarTermErr::*;
        let dataset = [
            ((2016, 11, 28), Err(OtherAnnus(Before))),
            ((2016, 11, 29), Ok((2016, 20, 7))),
            ((2016, 12, 7), Ok((2016, 21, 0))),
            ((2016, 12, 21), Ok((2017, 22, 0))),
            ((2016, 12, 22), Ok((2017, 22, 1))),
            ((2017, 1, 20), Ok((2017, 24, 0))),
            ((2017, 2, 3), Ok((2017, 1, 0))),
            ((2017, 12, 7), Ok((2017, 21, 0))),
            ((2017, 12, 17), Ok((2017, 21, 10))),
            ((2017, 12, 18), Ok((2017, 21, 11))),
            ((2017, 12, 21), Ok((2017, 21, 14))),
            ((2017, 12, 22), Err(OtherAnnus(After))),
        ];
        let annus = Annus::new(2017).unwrap();
        for ((y, m, d), std) in dataset {
            assert_eq!(
                std,
                annus.solar_term_for(Date::from_gregorian(y, m, d).unwrap())
            );
        }
    }

    #[test]
    fn year_sexagenary() {
        for (std, year) in [(60, -2697), (1, -2696), (17, 2000)] {
            assert_eq!(std, sexagenary_for_year(year));
        }
    }
}
