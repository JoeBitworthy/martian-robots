//! Randomised checks of the rules that must hold for any input, not just the
//! sample. A small seeded generator keeps the runs reproducible without
//! pulling in a dependency.

use std::collections::HashSet;

use martian_robots::command::Command;
use martian_robots::grid::{Grid, MAX_COORDINATE, Position};
use martian_robots::orientation::Orientation;
use martian_robots::robot::Robot;
use martian_robots::world::{Outcome, World};

const RUNS: usize = 500;
const ORIENTATIONS: [Orientation; 4] = [
    Orientation::North,
    Orientation::East,
    Orientation::South,
    Orientation::West,
];
const COMMANDS: [Command; 3] = [Command::Left, Command::Right, Command::Forward];

/// A linear congruential generator: enough randomness for a test, and the
/// same sequence on every machine.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next()).expect("u64 >> 33 fits in usize") % bound
    }

    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len())]
    }

    /// A coordinate anywhere in the allowed range.
    fn coordinate(&mut self) -> i32 {
        let choices = usize::try_from(MAX_COORDINATE + 1).expect("MAX_COORDINATE is positive");
        i32::try_from(self.below(choices)).expect("no larger than MAX_COORDINATE")
    }

    fn grid(&mut self) -> Grid {
        Grid::new(self.coordinate(), self.coordinate()).expect("generated within bounds")
    }

    fn robot_on(&mut self, grid: Grid) -> Robot {
        loop {
            let position = Position {
                x: self.coordinate(),
                y: self.coordinate(),
            };
            if grid.contains(position) {
                return Robot {
                    position,
                    facing: self.pick(&ORIENTATIONS),
                };
            }
        }
    }

    fn commands(&mut self) -> Vec<Command> {
        let len = self.below(100);
        (0..len).map(|_| self.pick(&COMMANDS)).collect()
    }
}

#[test]
fn every_robot_is_reported_at_a_cell_inside_the_grid() {
    let mut rng = Rng(2024);
    for _ in 0..RUNS {
        let grid = rng.grid();
        let mut world = World::new(grid);
        for _ in 0..rng.below(8) {
            let robot = rng.robot_on(grid);
            let reported = match world.execute(robot, &rng.commands()) {
                Outcome::Completed(robot) | Outcome::Lost(robot) => robot,
            };
            assert!(grid.contains(reported.position), "{reported:?} on {grid:?}");
        }
    }
}

#[test]
fn no_two_robots_are_ever_lost_from_the_same_cell() {
    let mut rng = Rng(7);
    for _ in 0..RUNS {
        let grid = rng.grid();
        let mut world = World::new(grid);
        let mut lost_from = HashSet::new();
        for _ in 0..rng.below(20) {
            let robot = rng.robot_on(grid);
            if let Outcome::Lost(lost) = world.execute(robot, &rng.commands()) {
                assert!(
                    lost_from.insert(lost.position),
                    "second robot lost from {:?} on {grid:?}",
                    lost.position
                );
            }
        }
    }
}

#[test]
fn a_robot_that_only_turns_never_moves() {
    let mut rng = Rng(99);
    for _ in 0..RUNS {
        let grid = rng.grid();
        let mut world = World::new(grid);
        let robot = rng.robot_on(grid);
        let turns: Vec<Command> = (0..rng.below(100))
            .map(|_| rng.pick(&[Command::Left, Command::Right]))
            .collect();
        match world.execute(robot, &turns) {
            Outcome::Completed(finished) => assert_eq!(finished.position, robot.position),
            Outcome::Lost(lost) => panic!("turning lost a robot: {lost:?}"),
        }
    }
}
