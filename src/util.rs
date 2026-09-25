//! 小型工具函数（std-only，不引入时间库）。

/// 把 Unix 秒时间戳格式化为 UTC ISO 8601（如 `2026-09-25T21:41:50Z`）。
///
/// 日期换算采用 Howard Hinnant 的 civil_from_days 算法（公历历法，
/// 含闰年/世纪规则），仅依赖整数运算；单元测试锚定若干已知时间点。
pub fn format_utc_iso(epoch_secs: u64) -> String {
    let secs = epoch_secs as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, h, m, s
    )
}

/// 天数（自 1970-01-01）→ (年, 月, 日)。Hinnant, "chrono-Compatible Low-Level Date Algorithms"。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (y + if m <= 2 { 1 } else { 0 }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_formats() {
        assert_eq!(format_utc_iso(0), "1970-01-01T00:00:00Z");
        // 两个广为人知的时间锚点
        assert_eq!(format_utc_iso(1_000_000_000), "2001-09-09T01:46:40Z");
        assert_eq!(format_utc_iso(2_000_000_000), "2033-05-18T03:33:20Z");
    }

    #[test]
    fn leap_year_and_rollover() {
        // 闰日：2024-02-29T00:00:00Z = 1709164800
        assert_eq!(format_utc_iso(1_709_164_800), "2024-02-29T00:00:00Z");
        // 年末跨日：2023-12-31T23:59:59Z = 1704067199
        assert_eq!(format_utc_iso(1_704_067_199), "2023-12-31T23:59:59Z");
        assert_eq!(format_utc_iso(1_704_067_200), "2024-01-01T00:00:00Z");
    }
}
