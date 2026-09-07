use crate::command::Command;
use crate::grid::Position;
use crate::orientation::Orientation;

/// Where a robot is and which way it faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Robot {
    pub position: Position,
    pub facing: Orientation,
}

impl Robot {
    /// The robot after carrying out one command, ignoring the world entirely.
    /// Whether the resulting position is actually on the grid is the world's
    /// decision, not the robot's.
    pub fn apply(self, command: Command) -> Self {
        match command {
            Command::Left => Self {
                facing: self.facing.left(),
                ..self
            },
            Command::Right => Self {
                facing: self.facing.right(),
                ..self
            },
            Command::Forward => Self {
                position: self.position.step(self.facing),
                ..self
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command::{Forward, Left, Right};
    use crate::orientation::Orientation::{East, North, South, West};

    fn robot(x: i32, y: i32, facing: Orientation) -> Robot {
        Robot {
            position: Position { x, y },
            facing,
        }
    }

    #[test]
    fn turning_changes_facing_but_not_position() {
        assert_eq!(robot(1, 1, North).apply(Left), robot(1, 1, West));
        assert_eq!(robot(1, 1, North).apply(Right), robot(1, 1, East));
    }

    #[test]
    fn forward_moves_one_cell_in_each_direction() {
        assert_eq!(robot(2, 2, North).apply(Forward), robot(2, 3, North));
        assert_eq!(robot(2, 2, East).apply(Forward), robot(3, 2, East));
        assert_eq!(robot(2, 2, South).apply(Forward), robot(2, 1, South));
        assert_eq!(robot(2, 2, West).apply(Forward), robot(1, 2, West));
    }

    #[test]
    fn forward_keeps_facing() {
        for facing in [North, East, South, West] {
            assert_eq!(robot(2, 2, facing).apply(Forward).facing, facing);
        }
    }

    #[test]
    fn apply_is_pure_and_may_leave_the_grid() {
        assert_eq!(robot(0, 0, South).apply(Forward), robot(0, -1, South));
    }
}
