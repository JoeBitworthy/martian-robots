/// An instruction a robot can carry out.
///
/// The brief says more instruction types may be needed later. The set is
/// closed and known at compile time, so it is an enum rather than a trait:
/// adding a variant makes the compiler point at every `match` that still has
/// to handle it, which is exactly the provision the brief asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Left,
    Right,
    Forward,
}

impl TryFrom<char> for Command {
    type Error = char;

    fn try_from(letter: char) -> Result<Self, Self::Error> {
        match letter {
            'L' => Ok(Self::Left),
            'R' => Ok(Self::Right),
            'F' => Ok(Self::Forward),
            other => Err(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Command::{self, Forward, Left, Right};

    #[test]
    fn parses_each_instruction_letter() {
        assert_eq!(Command::try_from('L'), Ok(Left));
        assert_eq!(Command::try_from('R'), Ok(Right));
        assert_eq!(Command::try_from('F'), Ok(Forward));
    }

    #[test]
    fn rejects_unknown_letters() {
        assert_eq!(Command::try_from('X'), Err('X'));
        assert_eq!(Command::try_from('f'), Err('f'));
        assert_eq!(Command::try_from(' '), Err(' '));
    }
}
