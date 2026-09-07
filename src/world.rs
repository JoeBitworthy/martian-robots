use crate::command::Command;
use crate::grid::{Grid, Position};
use crate::robot::Robot;

/// How a robot's instructions ended. Both carry the robot's final state; a
/// lost robot reports the last cell it stood on before leaving the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Completed(Robot),
    Lost(Robot),
}

/// The grid plus the scent each lost robot leaves on the cell it fell from.
/// The scents are the only state shared between robots, which is why robots
/// run one after another.
///
/// The grid is at most 51 by 51, so scents are one flag per cell rather than
/// a hash set: a lookup is an array read.
#[derive(Debug)]
pub struct World {
    grid: Grid,
    scents: Vec<bool>,
}

impl World {
    pub fn new(grid: Grid) -> Self {
        Self {
            grid,
            scents: vec![false; grid.cell_count()],
        }
    }

    /// Run one robot's commands in order.
    ///
    /// A command that would take the robot off the grid is ignored if an
    /// earlier robot was lost from the same cell; otherwise the robot is lost,
    /// leaves its scent there, and carries out no further commands.
    pub fn execute(&mut self, start: Robot, commands: &[Command]) -> Outcome {
        let mut robot = start;
        for &command in commands {
            let next = robot.apply(command);
            if self.grid.contains(next.position) {
                robot = next;
            } else if !self.is_scented(robot.position) {
                self.add_scent(robot.position);
                return Outcome::Lost(robot);
            }
        }
        Outcome::Completed(robot)
    }

    pub fn is_scented(&self, position: Position) -> bool {
        self.grid
            .cell(position)
            .is_some_and(|cell| self.scents[cell])
    }

    fn add_scent(&mut self, position: Position) {
        if let Some(cell) = self.grid.cell(position) {
            self.scents[cell] = true;
        }
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
    fn a_lost_robot_leaves_a_scent_where_it_fell() {
        let mut world = world();
        world.execute(robot(3, 3, North), &[F]);
        assert!(world.is_scented(Position { x: 3, y: 3 }));
        assert!(
            !world.is_scented(Position { x: 3, y: 4 }),
            "not off the grid"
        );
        assert!(
            !world.is_scented(Position { x: 3, y: 2 }),
            "not the cell before"
        );
    }

    #[test]
    fn a_completed_robot_leaves_no_scent() {
        let mut world = world();
        world.execute(robot(1, 1, East), &[F, F]);
        assert!(!world.is_scented(Position { x: 1, y: 1 }));
        assert!(!world.is_scented(Position { x: 3, y: 1 }));
    }

    #[test]
    fn a_scented_cell_makes_the_fatal_move_a_no_op() {
        let mut world = world();
        world.execute(robot(3, 3, North), &[F]);

        let outcome = world.execute(robot(3, 3, North), &[F]);
        assert_eq!(outcome, Outcome::Completed(robot(3, 3, North)));
    }

    #[test]
    fn a_rescued_robot_carries_on_with_its_remaining_instructions() {
        let mut world = world();
        world.execute(robot(3, 3, North), &[F]);

        let outcome = world.execute(robot(3, 2, North), &[F, F, L, F, L, F]);
        assert_eq!(outcome, Outcome::Completed(robot(2, 2, South)));
    }

    #[test]
    fn a_scent_protects_the_cell_in_every_direction() {
        let mut world = world();
        world.execute(robot(5, 3, North), &[F]);

        let outcome = world.execute(robot(5, 3, East), &[F, R, F]);
        assert_eq!(outcome, Outcome::Completed(robot(5, 2, South)));
    }

    #[test]
    fn a_scent_only_protects_its_own_cell() {
        let mut world = world();
        world.execute(robot(3, 3, North), &[F]);

        let outcome = world.execute(robot(2, 3, North), &[F]);
        assert_eq!(outcome, Outcome::Lost(robot(2, 3, North)));
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
