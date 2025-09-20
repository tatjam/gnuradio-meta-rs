use std::ops::{Add, Bound, Sub};

#[derive(PartialEq, Copy, Clone)]
pub struct RxTime {
    /// Maybe negative, but you will never find such a value in a GNU Radio file, only
    /// by using offset.
    /// If the RxTime was a real number, this corresponds to trunc(RxTime)
    sec: i64,
    /// If the RxTime was a real number, this corresponds to RxTime - trunc(RxTime)
    frac: f64,
}

impl RxTime {
    fn new(sec: i64, frac: f64) -> RxTime {
        debug_assert_eq!(sec.signum(), frac.signum() as i64);
        RxTime { sec, frac }
    }
    fn from_secs(sec: f64) -> RxTime {
        RxTime {
            sec: sec.trunc() as i64,
            frac: sec - sec.trunc(),
        }
    }

    /// Could have some rounding error if the number of seconds is large,
    /// or if the RxTime is not relative to 0, but to a given epoch (say UNIX timestamp).
    fn total_secs(self) -> f64 {
        return self.sec as f64 + self.frac;
    }

    /// Returns true if self and b represent the same timestamp, up to
    /// the precision (in seconds) stated in the argument
    fn is_same_as(self, b: RxTime, tol: f64) -> bool {
        self.sec == b.sec && (self.frac - b.frac).abs() <= tol
    }
}

impl Add for RxTime {
    type Output = RxTime;

    fn add(self, other: RxTime) -> RxTime {
        todo!("Implement");
    }
}

impl Sub for RxTime {
    type Output = RxTime;

    fn sub(self, other: RxTime) -> RxTime {
        todo!("Implement");
    }
}

#[cfg(test)]
mod test {
    const TOLERANCE: f64 = 1e-9; // 1ns error is allowed in these tests
    use super::RxTime;

    #[test]
    fn rxtime_arithmetic_small() {
        let a = RxTime::new(4, 0.5);
        let b = RxTime::new(1, 0.5);
        let c = a + b;
        assert!(c.is_same_as(RxTime::new(6, 0.0), TOLERANCE));
        let d = c - b;
        assert!(d.is_same_as(a, TOLERANCE));
        let e = c - a;
        assert!(e.is_same_as(b, TOLERANCE));
    }
    #[test]
    fn rxtime_arithmetic_big() {
        // UNIX timestamp: 2025-09-20T13:05:03+0000
        let start = RxTime::new(1758373503, 0.0);

        let a = start + RxTime::new(4, 0.5);
        let b = start + RxTime::new(1, 0.5);
        let c = a + b;
        assert!(c.is_same_as(start + RxTime::new(6, 0.0), TOLERANCE));
        let d = c - b;
        assert!(d.is_same_as(start + a, TOLERANCE));
        let e = c - a;
        assert!(e.is_same_as(start + b, TOLERANCE));
        let diff = b - a;
        assert!(diff.is_same_as(RxTime::new(-3, 0.0), TOLERANCE));
    }
    #[test]
    fn rxtime_add_negative() {}
}
