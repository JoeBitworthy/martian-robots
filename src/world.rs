use crate::command::Command;
use crate::grid::Grid;
use crate::robot::Robot;

/// How a robot's instructions ended. Both carry the robot's final state; a
/// lost robot reports the last cell it stood on before leaving the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Completed(Robot),
    Lost(Robot),
}

/// The grid the robots move on.
#[derive(Debug)]
pub struct World {
    grid: Grid,
}

impl World {
    pub fn new(grid: Grid) -> Self {
        Self { grid }
    }

    /// Run one robot's commands in order. A command that would take the robot
    /// off the grid loses it, and a lost robot carries out no further commands.
    pub fn execute(&mut self, start: Robot, commands: &[Command]) -> Outcome {
        let mut robot = start;
        for &command in commands {
            let next = robot.apply(command);
            if !self.grid.contains(next.position) {
                return Outcome::Lost(robot);
            }
            robot = next;
        }
        Outcome::Completed(robot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command::{Forward as F, Left as L, Right as R};
    use crate::grid::Position;
    use crate::orientation::Orientation::{self, East, North, South, West};

    fn world() -> World {
        World::new(Grid::new(5, 3).expect("5 3 is a valid grid"))
    }

    fn robot(x: i32, y: i32, facing: Orientation) -> Robot {
        Robot {
            position: Position { x, y },
            facing,
        }
    }

    #[test]
    fn empty_instructions_leave_the_robot_unchanged() {
        let start = robot(1, 1, East);
        assert_eq!(world().execute(start, &[]), Outcome::Completed(start));
    }

    #[test]
    fn turning_never_moves_the_robot() {
        let outcome = world().execute(robot(1, 1, East), &[L, L, R, L, R, R, R]);
        assert_eq!(outcome, Outcome::Completed(robot(1, 1, South)));
    }

    #[test]
    fn a_square_walk_returns_to_the_start() {
        let outcome = world().execute(robot(1, 1, East), &[R, F, R, F, R, F, R, F]);
        assert_eq!(outcome, Outcome::Completed(robot(1, 1, East)));
    }

    #[test]
    fn a_robot_may_stand_on_every_edge_cell() {
        let outcome = world().execute(robot(0, 0, East), &[F, F, F, F, F, L, F, F, F]);
        assert_eq!(outcome, Outcome::Completed(robot(5, 3, North)));
    }

    #[test]
    fn stepping_off_each_edge_loses_the_robot_at_its_last_cell() {
        let cases = [
            (robot(2, 3, North), "north"),
            (robot(5, 1, East), "east"),
            (robot(2, 0, South), "south"),
            (robot(0, 1, West), "west"),
        ];
        for (start, edge) in cases {
            assert_eq!(world().execute(start, &[F]), Outcome::Lost(start), "{edge}");
        }
    }

    #[test]
    fn a_lost_robot_ignores_its_remaining_instructions() {
        let outcome = world().execute(robot(3, 3, North), &[F, R, F, F]);
        assert_eq!(outcome, Outcome::Lost(robot(3, 3, North)));
    }

    #[test]
    fn a_corner_robot_facing_out_is_lost_immediately() {
        assert_eq!(
            world().execute(robot(0, 0, South), &[F]),
            Outcome::Lost(robot(0, 0, South))
        );
        assert_eq!(
            world().execute(robot(5, 3, East), &[F]),
            Outcome::Lost(robot(5, 3, East))
        );
    }
}
