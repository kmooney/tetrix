use crate::shape::{Shape, Orientation, Point};
use crate::board::Board;
use crate::WIDTH;

pub enum Direction {
    Ccw, Cw
}

pub struct ShapeState {
    orientation: Orientation,
    position: Point,
    shape: Shape
}

impl ShapeState {
    pub fn new() -> ShapeState {
        let s = Shape::random();
        let position = match s {
            Shape::Eye => Point{x: 3, y: 21},
            _ => Point{x: 4, y:21},
        };

        ShapeState {
            orientation: Orientation::Up,
            position: position,
            shape: s        
        }
    }

    pub fn set_shape(&mut self, s: Shape) {
        self.shape = s;
    }

    pub fn set_position(&mut self, p: Point) {
        self.position = p;
    }

    pub fn set_orientation(&mut self, o: Orientation) {
        self.orientation = o;
    }

    pub fn shape(&self) -> &Shape {
        return &self.shape;
    }

    pub fn position(&self) -> &Point {
        return &self.position;
    }

    pub fn orientation(&self) -> &Orientation {
        return &self.orientation;
    }

    pub fn down(&mut self) {
        if self.position.y > 0 {
            self.position.y -= 1;
        }
    }

    pub fn left(&mut self, b: &Board) {
        if self.position.x > 0 {
            self.position.x -= 1;
        }
        if self.any_collide(b) {
            self.position.x += 1;
        }
    }

    pub fn right(&mut self, b: &Board) {
        if self.position.x <= WIDTH {
            self.position.x += 1;
        }
        if self.any_collide(b) {
            self.position.x -= 1;
        }
    }

    pub fn rotate(&mut self, d: Direction, b: &Board) {
        match d {
            Direction::Ccw => self.rotate_ccw(),
            Direction::Cw => self.rotate_cw()
        }

        loop {
            if !self.any_collide(b) { return }
            if self.position.x != 0 {
                self.position.x -= 1;
            }
            if !self.any_collide(b) { return }
            self.position.x += 1;
            self.position.y += 1;        
        }

    }

    pub fn rotate_cw(&mut self) {
        self.orientation = match self.orientation {
            Orientation::Up => Orientation::Right,
            Orientation::Right => Orientation::Down,
            Orientation::Down => Orientation::Left,
            Orientation::Left => Orientation::Up
        }
    }

    pub fn rotate_ccw(&mut self) {
        self.orientation = match self.orientation {
            Orientation::Up => Orientation::Left,
            Orientation::Left => Orientation::Down,
            Orientation::Down => Orientation::Right,
            Orientation::Right => Orientation::Up
        }
    }

    pub fn any_collide(&self, b: &Board) -> bool {
        if self.position.y == 0 {return true}
        let width = self.shape.width(&self.orientation);
        let position = &self.position;
        if position.x + width > WIDTH { return true }
        let mat = &self.shape.to_mat(&self.orientation);
        for my in 0..3 {
            for mx in 0..3 {
                if position.x + mx >= WIDTH || mat[3 - my][mx] && b.0[position.y + my][position.x + mx] {
                    return true
                }   
            }
        }

        return false
    }

    pub fn drop(&mut self, b: &Board) {
        loop {
            if self.position.y != 0 {
                self.position.y -= 1;
            }

            if self.any_collide(b) {
                self.position.y += 1;
                break;
            }
        }
    }
}