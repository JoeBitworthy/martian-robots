//! Martian Robots: a bounded grid, robots that follow instructions, and the
//! scent that lost robots leave behind. This crate holds the simulation with
//! no I/O; the binary in `main.rs` is the only thing that touches stdin/stdout.

pub mod command;
pub mod grid;
pub mod orientation;
pub mod parse;
pub mod robot;
pub mod world;

use std::io::BufRead;

use parse::{Deployments, InputError};
use world::{Outcome, World};

/// A mission in progress. Each call to `next` reads one robot, runs it and
/// returns its outcome. A robot finishes before the next one is read, so its
/// scent is in place, and only the current robot is held in memory.
pub struct Simulation<R> {
    world: World,
    deployments: Deployments<R>,
}

impl<R: BufRead> Iterator for Simulation<R> {
    type Item = Result<Outcome, InputError>;

    fn next(&mut self) -> Option<Self::Item> {
        let deployment = self.deployments.next()?;
        Some(deployment.map(|d| self.world.execute(d.robot, &d.commands)))
    }
}

/// Check the grid line and start the mission. Robots are read and run as the
/// iterator is consumed.
pub fn simulate<R: BufRead>(input: R) -> Result<Simulation<R>, InputError> {
    let (grid, deployments) = parse::parse(input)?;
    Ok(Simulation {
        world: World::new(grid),
        deployments,
    })
}

/// The whole program as a pure function: input text in, output text out.
pub fn run(input: &str) -> Result<String, InputError> {
    simulate(input.as_bytes())?
        .map(|outcome| outcome.map(|outcome| format!("{outcome}\n")))
        .collect()
}
