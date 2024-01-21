use std::cmp::Ordering;

use crate::Side;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Turn {
    pub side: Side,
    pub number: u32,
}

impl Turn {
    pub fn new(side: Side, number: u32) -> Self {
        Turn { side, number }
    }

    pub fn next(&self) -> Turn {
        match self.side {
            Side::Axis => Turn::new(Side::Allies, self.number),
            Side::Allies => Turn::new(Side::Axis, self.number + 1),
        }
    }

    pub fn previous(&self) -> Turn {
        match self.side {
            Side::Axis => Turn::new(Side::Allies, self.number - 1),
            Side::Allies => Turn::new(Side::Axis, self.number),
        }
    }
}

impl Ord for Turn {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.number.cmp(&other.number) {
            Ordering::Equal => self.side.cmp(&other.side),
            order => order,
        }
    }
}

impl PartialOrd for Turn {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turn_comparison() {
        let axis_45 = Turn::new(Side::Axis, 45);
        let allies_45 = Turn::new(Side::Allies, 45);
        let axis_46 = Turn::new(Side::Axis, 46);

        assert!(allies_45 > axis_45);
        assert!(allies_45 < axis_46);
        assert_eq!(allies_45, allies_45);
    }

    #[test]
    fn turn_next() {
        let axis_45 = Turn::new(Side::Axis, 45);
        let allies_45 = Turn::new(Side::Allies, 45);
        let axis_46 = Turn::new(Side::Axis, 46);

        assert!(allies_45.next() > axis_45);
        assert_eq!(allies_45.next(), axis_46);
        assert_eq!(axis_45.next(), allies_45);
    }

    #[test]
    fn turn_previous() {
        let axis_45 = Turn::new(Side::Axis, 45);
        let allies_45 = Turn::new(Side::Allies, 45);
        let axis_46 = Turn::new(Side::Axis, 46);

        assert!(allies_45.next().previous() == allies_45);
        assert_eq!(axis_46.previous(), allies_45);
        assert_eq!(allies_45.previous(), axis_45);
    }
}
