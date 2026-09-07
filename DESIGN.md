# Design notes

## Shape

    stdin -> parse -> World::execute -> Display -> stdout

`main.rs` reads stdin, drives the iterator and writes stdout. `parse.rs` is
the only code that reads text. `Orientation`, `Position`, `Grid`, `Command`
and `Robot` are `Copy` values. `World` holds the grid and one scent flag per
cell, which is the only mutable state. `simulate()` in `lib.rs` chains the
stages as an iterator so the binary streams, and `run()` collects that
iterator into a `String` so tests can treat the program as a function from
text to text.

## Where each rule lives

| Rule from the brief                                 | File             |
| --------------------------------------------------- | ---------------- |
| North is (x, y) to (x, y + 1); L and R turn 90°      | `orientation.rs` |
| Lower-left is (0, 0), upper-right inclusive, max 50 | `grid.rs`        |
| L, R, F, with room for more                         | `command.rs`     |
| What a command does to a robot                      | `robot.rs`       |
| Off the edge is lost; scents; lost robots stop      | `world.rs`       |
| `x y O` and `x y O LOST`                            | `world.rs`       |
| Input format, blank lines, under 100 instructions   | `parse.rs`       |

## Decisions

**Commands are an enum, not a trait.** The command set is closed and known at
compile time, so an enum with exhaustive `match` gives the strongest
guarantee for the least code: add a variant and the compiler lists every
place that needs a new arm. A trait object would only pay off if commands had
to be registered at runtime or from another crate, which the brief does not
ask for.

**The robot is pure, the world judges.** `Robot::apply` knows nothing about
the grid and can return an off-grid position. `World::execute` checks the
result: on the grid, move; off the grid from a scented cell, ignore the
command; off the grid from a clean cell, leave a scent and stop. Any future
command that moves a robot gets the edge and scent rules without extra code.

**Scents are keyed by cell.** The brief says a scent stops robots "dropping
off the world at the same grid point". Keying by cell and direction is a
stricter reading that also passes the sample. The two differ in corners:
after a robot is lost from `0 0 S`, a second robot at `0 0 S` running `FRF`
ends at `0 0 W` under the cell reading and is lost on the westward step under
the directional one. Switching is a small change in `World` plus the test
that pins it.

**Scents are one flag per cell.** The grid is at most 51 by 51, so `World`
keeps a `Vec<bool>` indexed by `Grid::cell`. A lookup is an array read and
the largest possible world is under 3 KB. A `HashSet<Position>` is the
general answer; the bound in the brief makes the specific one simpler and
faster.

**Input and output stream.** Robots are read one at a time into a reused
buffer, run, and written to a buffered stdout, so memory does not depend on
the number of robots. A release build handles a million robots (56 MB of
input) in about 0.7 seconds with a peak of 1.6 MB. Output is flushed in 8 KB
chunks, so a tiny input prints all at once at the end. A closed pipe, as with
`| head`, ends the program quietly.

**A lost robot reports its last cell and stops.** Both follow from `execute`
returning early with the robot as it was before the fatal move.

**Blank lines separate robots, except directly after a robot line.** The
sample uses blank separators, so blank lines are skipped where they are
optional. The instruction line is never optional, so a blank one there is a
robot with no instructions.

**Validation is strict and line-numbered.** Every rule in the brief has its
own `InputError` variant carrying the line it was found on, for example
`line 4: unknown instruction 'X'`. The run stops at the first bad line.
Robots before it have already been printed, and the error goes to stderr with
exit code 1. Trailing whitespace and CRLF are accepted.

**No dependencies, no persistence, no UI.** Parsing is `split_whitespace` and
`str::parse`, the property tests use a small linear congruential generator,
and errors are a plain enum. Nothing outlives the process, so nothing is
stored. A clean build takes about a second.

**Robots run sequentially.** Scents written by earlier robots are read by
later ones, and the brief requires it. Separate missions share nothing and
could run in parallel without changing this code.

## Extending it

To add `B`, step backwards without turning:

1. `command.rs`: add `Backward` to the enum and `'B' => Ok(Self::Backward)`
   to `TryFrom<char>`.
2. `robot.rs`: add an arm to `apply` that steps in `self.facing.left().left()`.
3. Add a unit test for the arm and a world test that `B` off an edge is lost
   and scented like any other move.

Nothing else changes, and the build fails between steps 1 and 2 until the
new arm exists.

## Testing

Unit tests sit inside the module they exercise. `tests/sample.rs` compares
the brief's sample byte for byte and proves the third robot survives only
because of the second robot's scent. `tests/properties.rs` runs a few hundred
random missions and checks that every reported position is on the grid, that
no two robots are ever lost from the same cell, and that a robot that only
turns never moves. The unit cases target the ways I expected to get this
wrong: the scent on the wrong cell, the lost robot reporting the off-grid
cell or carrying on, a blocked robot treated as lost, the inclusive edge, the
sign of north, and the 100-character limit (99 accepted, 100 rejected).

## Language

Every domain type is a couple of integers or an enum, so everything is `Copy`
and nothing is cloned. Exhaustive `match` makes a missed case a compile
error. Errors are `Result` values threaded with `?`, and there is no `unwrap`
in the crate. The core is generic over `BufRead`, so tests feed it byte slices
and the binary feeds it stdin.
