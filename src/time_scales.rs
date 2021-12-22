//! Deals with different time scales, specifically, conversion from TDB/TT into
//! UT (UTC or UT1).
//!
//! Only conversions necessary for other computations in this crate are
//! included for now.
//!
//! # Planned
//!
//! - Supoort UT1 before 1972 (inter- & extrapolation)

use crate::date::Date;

/// [Barycentric dynamic time](https://en.wikipedia.org/wiki/Barycentric_Dynamical_Time),
/// represented in Julian date (JD).
///
/// Ephemeris data are typically computed in this
/// time scale, and should be converted to UT when calculating dates.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Tdb(pub f64);

/// [Terristrial time](https://en.wikipedia.org/wiki/Terrestrial_Time),
/// represented in Julian date (JD).
///
/// Note: Because TT differs no more than centisenconds from TDB during
/// thousands of years, they are treated numerically the same in this crate for
/// calendar calculation.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Tt(pub f64);

impl From<Tdb> for Tt {
    fn from(tdb: Tdb) -> Tt {
        Tt(tdb.0)
    }
}

impl From<Tai> for Tt {
    fn from(tai: Tai) -> Tt {
        Tt(tai.0 + 32.184 / 86400.0)
    }
}

/// [International atomic time](https://en.wikipedia.org/wiki/International_Atomic_Time),
/// represented in Julian date (JD).
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Tai(pub f64);

impl From<Tt> for Tai {
    fn from(tt: Tt) -> Tai {
        Tai(tt.0 - 32.184 / 86400.0)
    }
}

impl From<Tdb> for Tai {
    fn from(tdb: Tdb) -> Tai {
        Tt::from(tdb).into()
    }
}

/// [Universal time](https://en.wikipedia.org/wiki/Universal_Time), the actual
/// civil time used for determining the actual date at a given time point.
///
/// This can be either UTC (coordinated universal time, based on TAI with leap
/// seconds) or UT1 (mean solar time on the Prime Meridian) depending on the
/// year. Specifically, UTC from 1972-01-01T00:00Z to the latest known leap
/// second, UT1 otherwise.
///
/// Due to irregularity of Earth's rotation, conversion from TAI to UT1 relies
/// on data points with inter-/extrapolation with
/// [a method described here](https://astro.ukho.gov.uk/nao/lvm/).
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Ut(pub f64);

impl Ut {
    // XXX Liveri `Enum`-n por distingi inter UTC & UT1
    /// Tries to convert a TAI (or other time scale easily convertible to TAI)
    /// into UT.
    ///
    /// # Example
    ///
    /// ```
    /// use kalendarilo::time_scales::{Tdb, Ut};
    /// let tdb = Tdb(2451543.166666667);
    /// let ut = Ut::convert(tdb);
    /// assert_eq!((1999, 12, 30), ut.date_in_timezone(0).gregorian());
    /// ```
    ///
    /// # Panics
    ///
    /// Does not currently support time before 1972-01-01 and will panic.
    /// Working on it.
    pub fn convert<T>(time: T) -> Self
    where
        T: Into<Tai>,
    {
        let tai = time.into();
        let &leap_seconds::Data {
            starts,
            ref leap_seconds,
            expires,
            c2,
        } = leap_seconds::data();

        if tai < starts {
            todo!("UT before UTC (1972-01-01)");
        } else if tai > expires {
            let diff = leap_seconds::estimate(tai) + c2;
            return Ut(tai.0 - diff / 86400.0); // NOTE UT1, ne UTC
        }

        let ls = match leap_seconds.partition_point(|ls| ls.tai <= tai) {
            0 => return Ut(tai.0 - 10.0 / 86400.0),
            i => &leap_seconds[i - 1],
        };
        let leap = (tai.0 - ls.tai.0).min(2.0) / 2.0;
        Ut(tai.0 - (ls.delta_secs as f64 + leap) / 86400.0)
    }
    /// Returns the date at the time point in timezone ahead (east) of UTC by
    /// `tz_offset_minutes`minutes.
    ///
    /// For Beijing time (UTC+8), `tz_offset_minutes` should be +480.
    ///
    /// # Example
    ///
    /// ```
    /// use kalendarilo::time_scales::{Tdb, Ut};
    /// let tdb = Tdb(2451543.166666667);
    /// let ut = Ut::convert(tdb);
    /// assert_eq!((1999, 12, 30), ut.date_in_timezone(480).gregorian());
    /// ```
    pub fn date_in_timezone(&self, tz_offset_minutes: i32) -> Date {
        let jdn = (self.0 + tz_offset_minutes as f64 / 1440.0).round() as u32;
        Date::from_jdn(jdn)
    }
}

mod leap_seconds {
    use super::{Tai, Tt};
    use crate::date::Date;
    use std::sync::Once;

    pub const DATES: &[(i32, i32, i32)] = &[
        (1972, 6, 30),
        (1972, 12, 31),
        (1973, 12, 31),
        (1974, 12, 31),
        (1975, 12, 31),
        (1976, 12, 31),
        (1977, 12, 31),
        (1978, 12, 31),
        (1979, 12, 31),
        (1981, 6, 30),
        (1982, 6, 30),
        (1983, 6, 30),
        (1985, 6, 30),
        (1987, 12, 31),
        (1989, 12, 31),
        (1990, 12, 31),
        (1992, 6, 30),
        (1993, 6, 30),
        (1994, 6, 30),
        (1995, 12, 31),
        (1997, 6, 30),
        (1998, 12, 31),
        (2005, 12, 31),
        (2008, 12, 31),
        (2012, 6, 30),
        (2015, 6, 30),
        (2016, 12, 31),
    ];
    pub const DATE_EXPIRES: (i32, i32, i32) = (2021, 12, 31);

    #[derive(Debug, Clone)]
    pub struct Data {
        pub starts: Tai,
        pub leap_seconds: Vec<LeapSecond>,
        pub expires: Tai,
        pub c2: f64,
    }
    #[derive(Debug, Clone)]
    pub struct LeapSecond {
        pub tai: Tai,
        pub delta_secs: i32,
    }

    static mut COMPUTED: Data = Data {
        starts: Tai(0.0),
        leap_seconds: vec![],
        expires: Tai(0.0),
        c2: 0.0,
    };
    static INIT: Once = Once::new();

    pub fn data() -> &'static Data {
        INIT.call_once(|| {
            let starts =
                Tai(Date::from_gregorian(1972, 1, 1).unwrap().jdn() as f64 + 10.0 / 86400.0);
            unsafe {
                COMPUTED.starts = starts;
                COMPUTED.leap_seconds.reserve_exact(DATES.len());
            }
            for (delta_secs, &(y, m, d)) in (10..).zip(DATES) {
                let jdn = Date::from_gregorian(y, m, d)
                    .unwrap_or_else(|| panic!("date not recognized: {:?}", (y, m, d)))
                    .jdn();
                let tai = Tai(jdn as f64 + (43199 + delta_secs) as f64 / 86400.0);
                unsafe {
                    COMPUTED.leap_seconds.push(LeapSecond { tai, delta_secs });
                }
            }
            let (y, m, d) = DATE_EXPIRES;
            let jdn = Date::from_gregorian(y, m, d)
                .unwrap_or_else(|| panic!("date not recognized: {:?}", (y, m, d)))
                .jdn();
            let tai = Tai(jdn as f64 + (43199 + 10 + DATES.len()) as f64 / 86400.0);
            let c2 = (DATES.len() + 10) as f64 - estimate(tai);
            unsafe {
                COMPUTED.expires = tai;
                COMPUTED.c2 = c2;
            }
        });
        unsafe { &COMPUTED }
    }

    pub fn estimate<T: Into<Tt>>(tt: T) -> f64 {
        use std::f64::consts::PI;
        let tt = tt.into();
        let y = (tt.0 - 2451544.5) / 365.2425 + 2000.0;
        let t = (y - 1825.0) / 100.0;
        31.4115 * t * t + 284.8435805251424 * (2.0 * PI * (t + 0.75) / 14.0).cos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tdb_to_ut() {
        let tdb = Tdb(2451543.166666667);
        let ut = Ut::convert(tdb);
        assert_eq!((1999, 12, 30), ut.date_in_timezone(0).gregorian());
        assert_eq!((1999, 12, 30), ut.date_in_timezone(480).gregorian());
        let ut_midnight = Ut(ut.0 + (32.0 + 32.184) / 86400.0);
        let ut_before_midnight = Ut(ut_midnight.0 - 1.0 / 86400.0);
        assert_eq!(
            (1999, 12, 30),
            ut_before_midnight.date_in_timezone(480).gregorian()
        );
        assert_eq!(
            (1999, 12, 31),
            ut_midnight.date_in_timezone(480).gregorian()
        );
    }

    #[test]
    fn playing_with() {
        let tdb = Tdb(2462501.166666667 + 5.647029454550371); // 2030 小寒
        let ut = Ut::convert(tdb);
        assert!((ut.0 - 2462506.81319).abs() <= 30.0 / 86400.0);
    }
}
