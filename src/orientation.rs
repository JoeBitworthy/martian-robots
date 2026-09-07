use std::fmt;

/// A compass direction. The variants are ordered clockwise so that a turn is
/// simply a step to the neighbouring variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

impl Orientation {
    /// Turn 90 degrees anticlockwise.
    pub fn left(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
        }
    }

    /// Turn 90 degrees clockwise.
    pub fn right(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    /// The `(dx, dy)` of one step forward. North goes from `(x, y)` to
    /// `(x, y + 1)`, as the brief defines it.
    pub fn delta(self) -> (i32, i32) {
        match self {
            Self::North => (0, 1),
            Self::East => (1, 0),
            Self::South => (0, -1),
            Self::West => (-1, 0),
        }
    }
}

impl TryFrom<char> for Orientation {
    type Error = char;

    fn try_from(letter: char) -> Result<Self, Self::Error> {
        match letter {
            'N' => Ok(Self::North),
            'E' => Ok(Self::East),
            'S' => Ok(Self::South),
            'W' => Ok(Self::West),
            other => Err(other),
        }
    }
}

impl fmt::Display for Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let letter = match self {
            Self::North => 'N',
            Self::East => 'E',
            Self::South => 'S',
            Self::West => 'W',
        };
        write!(f, "{letter}")
    }
}

#[cfg(test)]
mod tests {
    use super::Orientation::{self, East, North, South, West};

    const ALL: [Orientation; 4] = [North, East, South, West];

    #[test]
    fn left_turns_anticlockwise() {
        assert_eq!(North.left(), West);
        assert_eq!(West.left(), South);
        assert_eq!(South.left(), East);
        assert_eq!(East.left(), North);
    }

    #[test]
    fn right_turns_clockwise() {
        assert_eq!(North.right(), East);
        assert_eq!(East.right(), South);
        assert_eq!(South.right(), West);
        assert_eq!(West.right(), North);
    }

    #[test]
    fn left_then_right_is_identity() {
        for orientation in ALL {
            assert_eq!(orientation.left().right(), orientation);
            assert_eq!(orientation.right().left(), orientation);
        }
    }

    #[test]
    fn four_turns_come_full_circle() {
        for orientation in ALL {
            assert_eq!(orientation.left().left().left().left(), orientation);
            assert_eq!(orientation.right().right().right().right(), orientation);
        }
    }

    #[test]
    fn north_increases_y() {
        assert_eq!(North.delta(), (0, 1));
        assert_eq!(East.delta(), (1, 0));
        assert_eq!(South.delta(), (0, -1));
        assert_eq!(West.delta(), (-1, 0));
    }

    #[test]
    fn parses_each_compass_letter() {
        assert_eq!(Orientation::try_from('N'), Ok(North));
        assert_eq!(Orientation::try_from('E'), Ok(East));
        assert_eq!(Orientation::try_from('S'), Ok(South));
        assert_eq!(Orientation::try_from('W'), Ok(West));
    }

    #[test]
    fn rejects_unknown_letter() {
        assert_eq!(Orientation::try_from('X'), Err('X'));
        assert_eq!(Orientation::try_from('n'), Err('n'));
    }

    #[test]
    fn displays_as_single_letter() {
        let letters: Vec<String> = ALL.iter().map(ToString::to_string).collect();
        assert_eq!(letters, ["N", "E", "S", "W"]);
    }
}
