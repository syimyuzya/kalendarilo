//! Utilities for converting between dates in different calendars.
//!
//! Currently, this crate mainly supports conversion into Chinese lunisolar
//! calendar (modern version) since 1973, using modern astronomical data for
//! accurate prediction.
//!
//! # Examples
//!
//! Basic usage with [`Date`]:
//!
//! ```
//! use kalendarilo::Date;
//!
//! let date = Date::from_gregorian(2000, 1, 1).unwrap();
//!
//! assert_eq!(6, date.day_of_week()); // Saturday
//! assert_eq!(2451545, date.jdn());
//! ```
//!
//! Chinese lunisolar calendar:
//!
//! ```
//! use kalendarilo::Date;
//! use kalendarilo::chinese::{Annus, Month::*};
//!
//! let date = Date::from_gregorian(2000, 1, 1).unwrap();
//! let annus = Annus::from_date(date).unwrap();
//!
//! assert_eq!(Ok((1999, Common(11), 25)), annus.ymd_for(date));
//! ```
//!
//! # Planned features
//!
//! - Gregorian computus (for calculating date of Easter)
//!     - (Possibly) full Gregorian lunisolar calendar
//! - Timezone-neutrual version of Chinese calendar (differs slightly from the
//!   standard version in some corner cases)
//! - Chinese calendar before 1973
//!
//! I wrote this primarily for my own use, so the design and development of
//! this crate will depend heavily on my personal need.

pub mod chinese;
pub mod date;
pub mod time_scales;

pub use date::{Date, YearType};
