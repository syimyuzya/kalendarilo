//! Calendar-independant date.

use std::ops::{Add, Sub};

/// A calendar-independant date.
///
/// Supported range begins from January 1, 4713 BC, proleptic Julian calendar.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Date {
    jdn: u32,
}

impl Date {
    /// Creates a `Date` with a Julian day number (JDN).
    pub fn from_jdn(jdn: u32) -> Self {
        Self { jdn }
    }
    /// Returns the Julian day number (JDN) of the date.
    pub fn jdn(&self) -> u32 {
        self.jdn
    }

    /// Creates a `Date` with a Gregorian calendar date.
    ///
    /// `year` should be an astronomical year number, i.e. 1 BC is `0`, 2
    /// BC is `-1`, etc.
    ///
    /// Returns `None` if the result date is out of supported range.
    ///
    /// # Example
    ///
    /// ```
    /// use kalendarilo::Date;
    ///
    /// let date = Date::from_gregorian(2000, 1, 1).unwrap();
    /// assert_eq!(2451545, date.jdn());
    /// ```
    pub fn from_gregorian(year: i32, month: u32, day: u32) -> Option<Self> {
        let (y, m, d) = (year as i64, month as i64, day as i64);
        u32::try_from(
            (1461 * (y + 4800 + (m - 14) / 12)) / 4 + (367 * (m - 2 - 12 * ((m - 14) / 12))) / 12
                - (3 * ((y + 4900 + (m - 14) / 12) / 100)) / 4
                + d
                - 32075,
        )
        .map(Self::from_jdn)
        .ok()
    }
    /// Represents the date in Gregorian calendar.
    ///
    /// Returns in `(year, month, day)` format.
    ///
    /// # Example
    ///
    /// ```
    /// use kalendarilo::Date;
    ///
    /// let date = Date::from_jdn(2451545);
    /// assert_eq!((2000, 1, 1), date.gregorian());
    /// ```
    pub fn gregorian(&self) -> (i32, u32, u32) {
        let jdn = self.jdn as i64;
        let f = jdn + 1401 + (((4 * jdn + 274277) / 146097) * 3) / 4 - 38;
        let e = 4 * f + 3;
        let g = (e % 1461) / 4;
        let h = 5 * g + 2;
        let day = (h % 153) / 5 + 1;
        let month = (h / 153 + 2) % 12 + 1;
        let year = e / 1461 - 4716 + (12 + 2 - month) / 12;
        // Safety: guaranteed by the range of jdn (within u32)
        (year as i32, month as u32, day as u32)
    }
    /// Formats the date in ISO 8601 format.
    ///
    /// # Example
    ///
    /// ```
    /// use kalendarilo::Date;
    ///
    /// let date = Date::from_gregorian(2000, 1, 1).unwrap();
    /// assert_eq!("2000-01-01", date.iso_gregorian());
    /// ```
    pub fn iso_gregorian(&self) -> String {
        let (y, m, d) = self.gregorian();
        match y {
            0..=9999 => format!("{:04}-{:02}-{:02}", y, m, d),
            _ => format!("{:+05}-{:02}-{:02}", y, m, d),
        }
    }

    /// Returns the day of week of the date, in ISO-8601 numbering (i.e.
    /// `1..=7` for Monday through Sunday)
    ///
    /// # Example
    ///
    /// ```
    /// use kalendarilo::Date;
    ///
    /// let date = Date::from_gregorian(2000, 1, 1).unwrap();
    /// assert_eq!(6, date.day_of_week()); // Saturday
    /// ```
    pub fn day_of_week(&self) -> u32 {
        self.jdn % 7 + 1
    }
    /// Returns the Chinese sexagenary day number of the date, numbered from 1
    /// (甲子) to 60 (癸亥).
    ///
    /// # Example
    ///
    /// ```
    /// use kalendarilo::Date;
    ///
    /// let date = Date::from_gregorian(2000, 1, 1).unwrap();
    /// assert_eq!(55, date.sexagenary()); // 戊午
    /// ```
    pub fn sexagenary(&self) -> u32 {
        (self.jdn % 60 + 49) % 60 + 1
    }

    /// Returns the ISO-8601 week number (with the year of that week) of the
    /// date, in `(year, week)` format.
    ///
    /// ```
    /// use kalendarilo::Date;
    ///
    /// let date = Date::from_gregorian(2000, 1, 1).unwrap();
    /// assert_eq!((1999, 52), date.year_week_gregorian()); // 1999-W52-6
    /// ```
    pub fn year_week_gregorian(&self) -> (i32, u32) {
        let (y, m, d) = self.gregorian();
        let y_type = YearType::from_gregorian(y);
        let y_is_leap = y_type.is_leap() as i32;
        let dn = ordinal_day_number(m, d, y_type) as i32;
        let dow = self.day_of_week() as i32;
        let dow1 = (dow - dn).rem_euclid(7) + 1;
        if dow1 > 4 && dow1 - 1 + dn <= 7 {
            use std::cmp::Ordering::*;
            return match dow1.cmp(&6) {
                Less => (y - 1, 53),
                Equal => (y - 1, 52 + YearType::from_gregorian(y - 1).is_leap() as u32),
                Greater => (y - 1, 52),
            };
        }
        let dow_last = (dow1 + 364 + y_is_leap - 1).rem_euclid(7) + 1;
        if dow_last < 4 && 365 + y_is_leap + 1 - dn <= dow_last {
            return (y + 1, 1);
        }

        (y, ((dow1 + dn - 2) / 7 + (dow1 <= 4) as i32) as u32)
    }

    pub fn checked_signed_diff(&self, rhs: Date) -> Option<i32> {
        self.jdn.checked_signed_diff(rhs.jdn)
    }
}

impl Add<i32> for Date {
    type Output = Date;
    fn add(self, rhs: i32) -> Self::Output {
        Date::from_jdn(self.jdn.strict_add_signed(rhs))
    }
}
impl Add<u32> for Date {
    type Output = Date;
    fn add(self, rhs: u32) -> Self::Output {
        Date::from_jdn(self.jdn + rhs)
    }
}
impl Sub<Date> for Date {
    type Output = u32;
    fn sub(self, rhs: Date) -> Self::Output {
        self.jdn - rhs.jdn
    }
}

/// Indicates whether a year is a leap year or common year.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum YearType {
    Common,
    Leap,
}

impl YearType {
    /// Determines if `year` is a leap year in Gregorian calendar.
    pub fn from_gregorian(year: i32) -> Self {
        if year % 4 == 0 && year % 100 != 0 || year % 400 == 0 {
            Self::Leap
        } else {
            Self::Common
        }
    }
    /// Returns `true` if `self` is `Leap`, otherwise `false`.
    pub fn is_leap(&self) -> bool {
        matches!(self, YearType::Leap)
    }
}

fn ordinal_day_number(month: u32, day: u32, year_type: YearType) -> u32 {
    day + match month {
        1 => 0,
        2 => 31,
        _ => (59 + (153 * (month as i32 - 3) + 2) / 5) as u32 + year_type.is_leap() as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let date = Date::from_jdn(2440588);
        assert_eq!(2440588, date.jdn());
    }

    #[test]
    fn from_gregorian() {
        let date = Date::from_gregorian(1970, 1, 1).unwrap();
        assert_eq!(2440588, date.jdn());
        let date = Date::from_gregorian(2021, 9, 8).unwrap();
        assert_eq!(2459466, date.jdn());
    }

    #[test]
    fn to_gregorian() {
        let date = Date::from_jdn(2440588);
        assert_eq!((1970, 1, 1), date.gregorian());
        let date = Date::from_jdn(2459466);
        assert_eq!((2021, 9, 8), date.gregorian());
        let date = Date::from_jdn(2451545);
        assert_eq!((2000, 1, 1), date.gregorian());
    }

    #[test]
    fn gregorian_near_bounds() {
        assert_eq!((-4713, 11, 24), Date::from_jdn(0).gregorian());
        assert_eq!((11754508, 12, 13), Date::from_jdn(u32::MAX).gregorian());
        assert_eq!(0, Date::from_gregorian(-4713, 11, 24).unwrap().jdn());
        assert_eq!(None, Date::from_gregorian(-4713, 11, 23));
        assert_eq!(
            u32::MAX,
            Date::from_gregorian(11754508, 12, 13).unwrap().jdn()
        );
        assert_eq!(None, Date::from_gregorian(11754508, 12, 14));
    }

    #[test]
    fn to_day_of_week() {
        let date = Date::from_gregorian(1970, 1, 1).unwrap();
        assert_eq!(4, date.day_of_week());
        let date = Date::from_gregorian(2021, 9, 8).unwrap();
        assert_eq!(3, date.day_of_week());
    }

    #[test]
    fn to_sexagenary() {
        let date = Date::from_gregorian(1970, 1, 1).unwrap();
        assert_eq!(18, date.sexagenary());
        let date = Date::from_gregorian(2021, 9, 8).unwrap();
        assert_eq!(56, date.sexagenary());
    }

    #[test]
    fn to_year_week() {
        for ((y, m, d), expected) in [
            ((1980, 12, 28), (1980, 52)),
            ((1980, 12, 31), (1981, 1)),
            ((1981, 1, 1), (1981, 1)),
            ((1981, 1, 4), (1981, 1)),
            ((1981, 1, 5), (1981, 2)),
            ((1981, 12, 31), (1981, 53)),
            ((1982, 1, 1), (1981, 53)),
        ] {
            let date = Date::from_gregorian(y, m, d).unwrap();
            assert_eq!(expected, date.year_week_gregorian(), "{y:04}-{m:02}-{d:02}");
        }
        for i in 6..=12 {
            let date = Date::from_gregorian(2021, 9, i).unwrap();
            assert_eq!((2021, 36), date.year_week_gregorian(), "2021-09-{:02}", i);
        }
        for (d, w) in [(12, 10), (13, 11)] {
            let date = Date::from_gregorian(2023, 3, d).unwrap();
            assert_eq!((2023, w), date.year_week_gregorian(), "2023-03-{d:02}");
        }
    }

    #[test]
    fn iso_format() {
        assert_eq!(
            "2021-09-08",
            Date::from_gregorian(2021, 9, 8).unwrap().iso_gregorian()
        );
    }

    #[test]
    fn iso_format_extended() {
        assert_eq!(
            "9999-12-31",
            Date::from_gregorian(9999, 12, 31).unwrap().iso_gregorian()
        );
        assert_eq!(
            "+10000-01-01",
            Date::from_gregorian(10000, 1, 1).unwrap().iso_gregorian()
        );
        assert_eq!(
            "0000-01-01",
            Date::from_gregorian(0, 1, 1).unwrap().iso_gregorian()
        );
        assert_eq!(
            "-0001-12-31",
            Date::from_gregorian(-1, 12, 31).unwrap().iso_gregorian()
        );
    }
}

#[cfg(test)]
mod tests_priv {
    use super::*;

    #[test]
    fn priv_ordinal_day_number() {
        use YearType::*;
        assert_eq!(1, ordinal_day_number(1, 1, Common));
        assert_eq!(256, ordinal_day_number(9, 13, Common));
        assert_eq!(366, ordinal_day_number(12, 31, Leap));
    }
}
