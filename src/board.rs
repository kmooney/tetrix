use crate::{WIDTH, HEIGHT};
use crate::shape::{Point, ShapeMat};
use std::marker::Copy;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Board(pub [[bool; WIDTH]; HEIGHT]);

impl Board {

    pub fn new() -> Board {
        return Board([[false; WIDTH]; HEIGHT]);
    }

    pub fn report(&self) -> String {
        let mut board_report = String::new();
        board_report.push_str(&format!("[  ]----{:02}----\n", HEIGHT));
        for y in (0..HEIGHT).rev() {
            let row = self.0[y];
            board_report.push_str(&format!("{:02} ", y));
            for cell in row.iter() {
                board_report.push_str(match cell {
                    true => "x",
                    false => "_",
                })
            }
            board_report.push_str("\n");
        }
        board_report.push_str("-------------\n");
        board_report.push_str("  |0123456789\n");
        board_report
    }

    pub fn occupy(&mut self, m: &ShapeMat, p: &Point) {
        for y in 0..4 {
            for x in 0..4 {
                // 0 is the bottom of the board
                // so invert the y coordinate for this
                // shape
                if m[3-y][x] {
                    self.0[y+p.y][x+p.x] = m[3-y][x];
                }
            }
        }
    }

    pub fn vacate(&mut self, m: &ShapeMat, p: &Point) {
        for y in 0..4 {
            for x in 0..4 {
                if m[3-y][x] && self.0[y+p.y][x+p.x]  {
                    self.0[y+p.y][x+p.x] = false;
                }
            }
        }
    }

    pub fn reset(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.0[y][x] = false;
            }
        }
    }

    pub fn setup(&mut self, config: Vec<Vec<bool>>, position: Point, overwrite: bool) {
        let mut x;
        let mut y = 0;
        let config_height = config.len();
        for row in config.iter() {
            x = 0;
            for cell in row.iter() {
                if overwrite {
                    self.0[config_height - y + position.y - 1][x + position.x] = *cell;
                } else {
                    self.0[config_height - y + position.y - 1][x + position.x] |= *cell;
                }
                x += 1;
            }
            y += 1;
        }
    }

}