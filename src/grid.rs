use std::fmt;

use crate::orientation::Orientation;

/// The largest value any coordinate may take, from the brief.
pub const MAX_COORDINATE: i32 = 50;

/// A point on the grid. Signed so that a step off the bottom or left edge is
/// representable and can be rejected, rather than wrapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    /// The position one step away in the given direction. Pure geometry: it
    /// does not know or care whether the result is on the grid.
    pub fn step(self, facing: Orientation) -> Self {
        let (dx, dy) = facing.delta();
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

/// The rectangular surface of Mars. The lower-left corner is always `(0, 0)`
/// and the upper-right corner is inclusive: a `5 3` grid has six columns and
/// four rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grid {
    upper_right: Position,
}

/// A coordinate outside `0..=MAX_COORDINATE`. Wide enough to echo back any
/// number the parser could not fit into a coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidCoordinate(pub i64);

impl fmt::Display for InvalidCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "coordinate {} is outside 0..={MAX_COORDINATE}", self.0)
    }
}

impl std::error::Error for InvalidCoordinate {}

impl Grid {
    pub fn new(max_x: i32, max_y: i32) -> Result<Self, InvalidCoordinate> {
        for value in [max_x, max_y] {
            if !(0..=MAX_COORDINATE).contains(&value) {
                return Err(InvalidCoordinate(value.into()));
            }
        }
        Ok(Self {
            upper_right: Position { x: max_x, y: max_y },
        })
    }

    pub fn contains(self, position: Position) -> bool {
        (0..=self.upper_right.x).contains(&position.x)
            && (0..=self.upper_right.y).contains(&position.y)
    }

    /// How many cells the grid has, for anything that stores one value per cell.
    pub fn cell_count(self) -> usize {
        self.cell(self.upper_right).map_or(0, |last| last + 1)
    }

    /// The row-major index of a cell, or `None` if the position is off the grid.
    pub fn cell(self, position: Position) -> Option<usize> {
        if !self.contains(position) {
            return None;
        }
        let width = self.upper_right.x + 1;
        usize::try_from(position.y * width + position.x).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orientation::Orientation::{East, North, South, West};

    fn grid() -> Grid {
        Grid::new(5, 3).expect("5 3 is a valid grid")
    }

    #[test]
    fn contains_every_corner() {
        for (x, y) in [(0, 0), (5, 0), (0, 3), (5, 3)] {
            assert!(grid().contains(Position { x, y }), "({x}, {y})");
        }
    }

    #[test]
    fn excludes_one_step_past_each_edge() {
        for (x, y) in [(-1, 1), (6, 1), (2, -1), (2, 4)] {
            assert!(!grid().contains(Position { x, y }), "({x}, {y})");
        }
    }

    #[test]
    fn single_cell_grid_contains_only_origin() {
        let tiny = Grid::new(0, 0).expect("0 0 is a valid grid");
        assert!(tiny.contains(Position { x: 0, y: 0 }));
        assert!(!tiny.contains(Position { x: 1, y: 0 }));
        assert!(!tiny.contains(Position { x: 0, y: 1 }));
    }

    #[test]
    fn accepts_the_maximum_coordinate() {
        assert!(Grid::new(50, 50).is_ok());
    }

    #[test]
    fn rejects_coordinates_over_the_maximum() {
        assert_eq!(Grid::new(51, 3), Err(InvalidCoordinate(51)));
        assert_eq!(Grid::new(3, 51), Err(InvalidCoordinate(51)));
    }

    #[test]
    fn rejects_negative_coordinates() {
        assert_eq!(Grid::new(-1, 3), Err(InvalidCoordinate(-1)));
        assert_eq!(Grid::new(3, -1), Err(InvalidCoordinate(-1)));
    }

    #[test]
    fn cells_are_numbered_row_by_row_from_the_origin() {
        assert_eq!(grid().cell(Position { x: 0, y: 0 }), Some(0));
        assert_eq!(grid().cell(Position { x: 5, y: 0 }), Some(5));
        assert_eq!(grid().cell(Position { x: 0, y: 1 }), Some(6));
        assert_eq!(grid().cell(Position { x: 5, y: 3 }), Some(23));
        assert_eq!(grid().cell_count(), 24);
    }

    #[test]
    fn off_grid_positions_have_no_cell() {
        assert_eq!(grid().cell(Position { x: 6, y: 0 }), None);
        assert_eq!(grid().cell(Position { x: 0, y: -1 }), None);
    }

    #[test]
    fn step_moves_one_cell_in_the_facing_direction() {
        let origin = Position { x: 2, y: 2 };
        assert_eq!(origin.step(North), Position { x: 2, y: 3 });
        assert_eq!(origin.step(East), Position { x: 3, y: 2 });
        assert_eq!(origin.step(South), Position { x: 2, y: 1 });
        assert_eq!(origin.step(West), Position { x: 1, y: 2 });
    }

    #[test]
    fn invalid_coordinate_explains_the_bound() {
        assert_eq!(
            InvalidCoordinate(51).to_string(),
            "coordinate 51 is outside 0..=50"
        );
    }
}
