use crate::{WIDTH, HEIGHT};
use crate::shape_controller::ShapeController;

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

    pub fn occupy(&mut self, c: &ShapeController) {
        let m = c.shape().to_mat(c.orientation());
        let p = c.position();
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

    pub fn vacate(&mut self, c: &ShapeController) {
        let m = &c.shape().to_mat(c.orientation());
        let p = c.position();
        for y in 0..4 {
            for x in 0..4 {
                if m[3-y][x] && self.0[y+p.y][x+p.x]  {
                    self.0[y+p.y][x+p.x] = false;
                }
            }
        }
    }
}