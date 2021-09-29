# Kalendarilo

Calculate Chinese lunisolar calendar using accurate astronomical data (`data/TDBtimes.txt` from https://github.com/ytliu0/ChineseCalendar).

## Example

```rust
use kalendarilo::Date;
use kalendarilo::chinese::{Annus, Month::*};

let date = Date::from_gregorian(2000, 1, 1).unwrap();

assert_eq!(6, date.day_of_week()); // Saturday
assert_eq!(2451545, date.jdn());

let annus = Annus::from_date(date).unwrap();

assert_eq!(Ok((1999, Common(11), 25)), annus.ymd_for(date)); // 冬月廿五
```
