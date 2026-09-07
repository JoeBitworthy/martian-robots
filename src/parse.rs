use std::fmt;
use std::io::BufRead;

use crate::command::Command;
use crate::grid::{Grid, InvalidCoordinate, MAX_COORDINATE, Position};
use crate::orientation::Orientation;
use crate::robot::Robot;

/// Instruction strings must be shorter than this, from the brief.
pub const MAX_INSTRUCTIONS: usize = 100;

/// One robot and the commands it has been sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deployment {
    pub robot: Robot,
    pub commands: Vec<Command>,
}

/// Why the input could not be accepted. Every variant carries the 1-based
/// line number it was found on, except when there was no input at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputError {
    MissingGrid,
    Unreadable { line: usize, reason: String },
    Malformed { line: usize, expected: &'static str },
    InvalidCoordinate { line: usize, value: i64 },
    UnknownOrientation { line: usize, found: char },
    StartOutsideGrid { line: usize },
    MissingInstructions { line: usize },
    InstructionsTooLong { line: usize, len: usize },
    UnknownInstruction { line: usize, found: char },
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingGrid => {
                write!(
                    f,
                    "no input: expected the grid's upper-right coordinates on line 1"
                )
            }
            Self::Unreadable { line, reason } => {
                write!(f, "line {line}: could not read input: {reason}")
            }
            Self::Malformed { line, expected } => write!(f, "line {line}: expected {expected}"),
            Self::InvalidCoordinate { line, value } => {
                write!(f, "line {line}: {}", InvalidCoordinate(*value))
            }
            Self::UnknownOrientation { line, found } => {
                write!(
                    f,
                    "line {line}: unknown orientation '{found}', expected N, E, S or W"
                )
            }
            Self::StartOutsideGrid { line } => {
                write!(f, "line {line}: robot starts outside the grid")
            }
            Self::MissingInstructions { line } => {
                write!(f, "line {line}: robot has no instruction line after it")
            }
            Self::InstructionsTooLong { line, len } => write!(
                f,
                "line {line}: {len} instructions, but fewer than {MAX_INSTRUCTIONS} are allowed"
            ),
            Self::UnknownInstruction { line, found } => {
                write!(f, "line {line}: unknown instruction '{found}'")
            }
        }
    }
}

impl std::error::Error for InputError {}

/// Numbered, trimmed lines read one at a time into a single reused buffer.
/// Trailing whitespace and Windows line endings are tolerated, and the numbers
/// always refer to the original input.
struct Lines<R> {
    reader: R,
    buffer: String,
    number: usize,
}

impl<R: BufRead> Lines<R> {
    fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: String::new(),
            number: 0,
        }
    }

    /// The next line, blank or not, or `None` at the end.
    fn next_line(&mut self) -> Option<Result<(usize, &str), InputError>> {
        match self.read()? {
            Ok(number) => Some(Ok((number, self.buffer.trim()))),
            Err(error) => Some(Err(error)),
        }
    }

    /// The next non-blank line, or `None` at the end.
    fn next_non_blank(&mut self) -> Option<Result<(usize, &str), InputError>> {
        loop {
            match self.read()? {
                Ok(_) if self.buffer.trim().is_empty() => {}
                Ok(number) => return Some(Ok((number, self.buffer.trim()))),
                Err(error) => return Some(Err(error)),
            }
        }
    }

    fn read(&mut self) -> Option<Result<usize, InputError>> {
        self.buffer.clear();
        match self.reader.read_line(&mut self.buffer) {
            Ok(0) => None,
            Ok(_) => {
                self.number += 1;
                Some(Ok(self.number))
            }
            Err(error) => Some(Err(InputError::Unreadable {
                line: self.number + 1,
                reason: error.to_string(),
            })),
        }
    }
}

/// The robots in the input, read one at a time, so memory does not depend on
/// how many there are. Blank lines between robots are skipped, but the line
/// directly after a robot is always its instructions, so a blank one means no
/// instructions. Stops after the first error.
pub struct Deployments<R> {
    grid: Grid,
    lines: Lines<R>,
    failed: bool,
}

impl<R: BufRead> Iterator for Deployments<R> {
    type Item = Result<Deployment, InputError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        let deployment = read_deployment(self.grid, &mut self.lines)?;
        self.failed = deployment.is_err();
        Some(deployment)
    }
}

fn read_deployment<R: BufRead>(
    grid: Grid,
    lines: &mut Lines<R>,
) -> Option<Result<Deployment, InputError>> {
    let (line, text) = match lines.next_non_blank()? {
        Ok(found) => found,
        Err(error) => return Some(Err(error)),
    };
    let robot = match parse_robot(grid, line, text) {
        Ok(robot) => robot,
        Err(error) => return Some(Err(error)),
    };
    let commands = match lines.next_line() {
        None => Err(InputError::MissingInstructions { line }),
        Some(Err(error)) => Err(error),
        Some(Ok((line, text))) => parse_commands(line, text),
    };
    Some(commands.map(|commands| Deployment { robot, commands }))
}

/// Validate the grid line and hand back the robots as a lazy iterator over
/// the rest of the input.
pub fn parse<R: BufRead>(reader: R) -> Result<(Grid, Deployments<R>), InputError> {
    let mut lines = Lines::new(reader);
    let (line, text) = lines.next_non_blank().ok_or(InputError::MissingGrid)??;
    let grid = parse_grid(line, text)?;
    Ok((
        grid,
        Deployments {
            grid,
            lines,
            failed: false,
        },
    ))
}

fn parse_grid(line: usize, text: &str) -> Result<Grid, InputError> {
    const EXPECTED: &str = "two coordinates, like `5 3`";
    let malformed = InputError::Malformed {
        line,
        expected: EXPECTED,
    };
    let [max_x, max_y] = parse_coordinates(line, text, &malformed)?;
    Grid::new(max_x, max_y)
        .map_err(|InvalidCoordinate(value)| InputError::InvalidCoordinate { line, value })
}

fn parse_robot(grid: Grid, line: usize, text: &str) -> Result<Robot, InputError> {
    const EXPECTED: &str = "a position and orientation, like `1 1 E`";
    let malformed = InputError::Malformed {
        line,
        expected: EXPECTED,
    };
    let (coordinates, orientation) = text
        .rsplit_once(char::is_whitespace)
        .ok_or_else(|| malformed.clone())?;
    let [x, y] = parse_coordinates(line, coordinates, &malformed)?;

    let mut letters = orientation.chars();
    let facing = match (letters.next(), letters.next()) {
        (Some(letter), None) => Orientation::try_from(letter)
            .map_err(|found| InputError::UnknownOrientation { line, found })?,
        _ => return Err(malformed),
    };

    let position = Position { x, y };
    if !grid.contains(position) {
        return Err(InputError::StartOutsideGrid { line });
    }
    Ok(Robot { position, facing })
}

fn parse_commands(line: usize, text: &str) -> Result<Vec<Command>, InputError> {
    let len = text.chars().count();
    if len >= MAX_INSTRUCTIONS {
        return Err(InputError::InstructionsTooLong { line, len });
    }
    text.chars()
        .map(|letter| {
            Command::try_from(letter)
                .map_err(|found| InputError::UnknownInstruction { line, found })
        })
        .collect()
}

/// Two whitespace-separated integers. Anything that is not two integers is
/// malformed; an integer that cannot be a coordinate is reported as such.
fn parse_coordinates(
    line: usize,
    text: &str,
    malformed: &InputError,
) -> Result<[i32; 2], InputError> {
    let mut numbers = text.split_whitespace().map(str::parse::<i64>);
    let [x, y] = match (numbers.next(), numbers.next(), numbers.next()) {
        (Some(Ok(x)), Some(Ok(y)), None) => [x, y],
        _ => return Err(malformed.clone()),
    };
    let coordinate = |value: i64| {
        i32::try_from(value)
            .ok()
            .filter(|value| (0..=MAX_COORDINATE).contains(value))
            .ok_or(InputError::InvalidCoordinate { line, value })
    };
    Ok([coordinate(x)?, coordinate(y)?])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command::{Forward as F, Left as L, Right as R};
    use crate::orientation::Orientation::{East, North, West};

    fn robot(x: i32, y: i32, facing: Orientation) -> Robot {
        Robot {
            position: Position { x, y },
            facing,
        }
    }

    /// Drain the lazy parser so tests can look at everything at once.
    fn parse_all(input: &str) -> Result<(Grid, Vec<Deployment>), InputError> {
        let (grid, deployments) = parse(input.as_bytes())?;
        let deployments = deployments.collect::<Result<Vec<_>, _>>()?;
        Ok((grid, deployments))
    }

    #[test]
    fn parses_the_sample_input() {
        let input = "5 3\n1 1 E\nRFRFRFRF\n\n3 2 N\nFRRFLLFFRRFLL\n\n0 3 W\nLLFFFLFLFL\n";
        let (grid, deployments) = parse_all(input).expect("sample input is valid");

        assert_eq!(grid, Grid::new(5, 3).expect("valid grid"));
        assert_eq!(deployments.len(), 3);
        assert_eq!(
            deployments[0],
            Deployment {
                robot: robot(1, 1, East),
                commands: vec![R, F, R, F, R, F, R, F],
            }
        );
        assert_eq!(deployments[1].robot, robot(3, 2, North));
        assert_eq!(deployments[2].robot, robot(0, 3, West));
        assert_eq!(deployments[2].commands, vec![L, L, F, F, F, L, F, L, F, L]);
    }

    #[test]
    fn a_grid_with_no_robots_is_valid() {
        let (_, deployments) = parse_all("5 3\n").expect("a lonely grid is fine");
        assert!(deployments.is_empty());
    }

    #[test]
    fn robots_are_parsed_lazily_and_the_iterator_stops_after_an_error() {
        let (_, mut deployments) =
            parse("5 3\n1 1 E\nF\n2 2 N\nX\n3 3 S\nF\n".as_bytes()).expect("grid is valid");
        assert!(deployments.next().is_some_and(|first| first.is_ok()));
        assert!(deployments.next().is_some_and(|second| second.is_err()));
        assert!(
            deployments.next().is_none(),
            "nothing after the first error"
        );
    }

    #[test]
    fn rejects_empty_input() {
        assert_eq!(parse_all(""), Err(InputError::MissingGrid));
        assert_eq!(parse_all("\n  \n"), Err(InputError::MissingGrid));
    }

    #[test]
    fn rejects_a_malformed_grid_line() {
        for input in ["5", "5 x", "5 3 1", "five three"] {
            assert!(
                matches!(parse_all(input), Err(InputError::Malformed { line: 1, .. })),
                "{input:?}"
            );
        }
    }

    #[test]
    fn rejects_a_coordinate_outside_the_allowed_range() {
        assert_eq!(
            parse_all("51 3"),
            Err(InputError::InvalidCoordinate { line: 1, value: 51 })
        );
        assert_eq!(
            parse_all("5 -1"),
            Err(InputError::InvalidCoordinate { line: 1, value: -1 })
        );
        assert_eq!(
            parse_all("5 3\n99999999999 1 E\nF"),
            Err(InputError::InvalidCoordinate {
                line: 2,
                value: 99_999_999_999
            })
        );
    }

    #[test]
    fn a_blank_line_straight_after_a_robot_means_no_instructions() {
        let (_, deployments) = parse_all("5 3\n1 1 E\n\n2 2 N\nF\n").expect("valid");
        assert_eq!(deployments.len(), 2);
        assert!(deployments[0].commands.is_empty());
        assert_eq!(deployments[1].commands, vec![F]);
    }

    #[test]
    fn rejects_a_malformed_robot_line() {
        for input in [
            "5 3\n1 1\nF",
            "5 3\n1 E\nF",
            "5 3\n1 1 E extra\nF",
            "5 3\n1 1 EE\nF",
        ] {
            assert!(
                matches!(parse_all(input), Err(InputError::Malformed { line: 2, .. })),
                "{input:?}"
            );
        }
    }

    #[test]
    fn rejects_an_unknown_orientation() {
        assert_eq!(
            parse_all("5 3\n1 1 X\nF"),
            Err(InputError::UnknownOrientation {
                line: 2,
                found: 'X'
            })
        );
    }

    #[test]
    fn rejects_a_robot_starting_off_the_grid() {
        for input in ["5 3\n6 1 E\nF", "5 3\n1 4 E\nF", "5 3\n5 4 E\nF"] {
            assert_eq!(
                parse_all(input),
                Err(InputError::StartOutsideGrid { line: 2 }),
                "{input:?}"
            );
        }
    }

    #[test]
    fn rejects_a_robot_with_no_instructions() {
        assert_eq!(
            parse_all("5 3\n1 1 E"),
            Err(InputError::MissingInstructions { line: 2 })
        );
        assert_eq!(
            parse_all("5 3\n1 1 E\nF\n\n2 2 N"),
            Err(InputError::MissingInstructions { line: 5 })
        );
    }

    #[test]
    fn accepts_ninety_nine_instructions_but_not_one_hundred() {
        let ok = format!("5 3\n1 1 E\n{}", "L".repeat(99));
        assert!(parse_all(&ok).is_ok());

        let too_long = format!("5 3\n1 1 E\n{}", "L".repeat(100));
        assert_eq!(
            parse_all(&too_long),
            Err(InputError::InstructionsTooLong { line: 3, len: 100 })
        );
    }

    #[test]
    fn rejects_an_unknown_instruction_with_its_line_number() {
        assert_eq!(
            parse_all("5 3\n1 1 E\nF\n\n2 2 N\nFXF"),
            Err(InputError::UnknownInstruction {
                line: 6,
                found: 'X'
            })
        );
    }

    #[test]
    fn tolerates_blank_lines_crlf_and_trailing_whitespace() {
        let tidy = parse_all("5 3\n1 1 E\nRF\n2 2 N\nLF\n").expect("valid");
        let messy = parse_all("\r\n5 3 \r\n\r\n1 1 E\t\r\nRF \r\n\r\n\r\n2 2 N\r\nLF\r\n\r\n")
            .expect("valid");
        assert_eq!(tidy, messy);
    }

    #[test]
    fn reports_input_that_is_not_text_with_its_line_number() {
        let input: &[u8] = b"5 3\n1 1 E\nF\n\xff\n";
        let error = parse_all_bytes(input).expect_err("0xff is not UTF-8");
        assert_eq!(
            error,
            InputError::Unreadable {
                line: 4,
                reason: "stream did not contain valid UTF-8".to_string()
            }
        );
    }

    fn parse_all_bytes(input: &[u8]) -> Result<Vec<Deployment>, InputError> {
        parse(input)?.1.collect()
    }

    #[test]
    fn errors_read_as_plain_sentences() {
        let error = InputError::UnknownInstruction {
            line: 4,
            found: 'X',
        };
        assert_eq!(error.to_string(), "line 4: unknown instruction 'X'");

        let error = InputError::InvalidCoordinate { line: 1, value: 51 };
        assert_eq!(error.to_string(), "line 1: coordinate 51 is outside 0..=50");
    }
}
