use rand::Rng;
use std::marker::Copy;


#[derive(Debug)]
pub struct Point {
    pub x: usize, 
    pub y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Point {
        Point {x: x, y: y}
    }
}


pub type ShapeMat = [[bool; 4]; 4];

#[derive(Debug)]
pub enum Orientation {
    Up, Down, Left, Right,
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Shape {
    Eye, El, ElInv, Square, Zee, ZeeInv, Tee
}

impl Shape {
    pub fn random() -> Shape {
        match rand::thread_rng().gen_range(0, 7) {
            0 => Shape::Eye,
            1 => Shape::El,
            2 => Shape::ElInv,
            3 => Shape::Square,
            4 => Shape::Zee,
            5 => Shape::ZeeInv,
            6 => Shape::Tee,
            _ => panic!("wtf value is out of range")
        }	
    }

    pub fn width(&self, o: &Orientation) -> usize {
        match self {
            Shape::Eye => match o {
                Orientation::Left | Orientation::Right => 4,
                Orientation::Up | Orientation::Down => 1,
            },
            Shape::Square => 2,
            Shape::ElInv | Shape::El | Shape::ZeeInv | Shape::Zee => match o {
                Orientation::Left | Orientation::Right => 3,
                Orientation::Up | Orientation:: Down => 2,
            },
            Shape::Tee => {
                match o {
                    Orientation::Up | Orientation::Down => 3,
                    Orientation::Left | Orientation::Right => 2, 
                }
            }
        }
    }

    pub fn to_mat(&self, o: &Orientation) -> ShapeMat {
        match self {
            Shape::Tee => match o {
                Orientation::Up => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [false, true, false, false],
                    [true, true, true, false],
                ],
                Orientation::Down => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [true, true, true, false],
                    [false, true, false, false],
                ],
                Orientation::Left => [
                    [false, false, false, false],
                    [false, true, false, false],
                    [true, true, false, false],
                    [false, true, false, false],
                ],
                Orientation::Right => [
                    [false, false, false, false],
                    [true, false, false, false],
                    [true, true, false, false],
                    [true, false, false, false],
                ],
            },
            Shape::Eye => match o {
                Orientation::Left | Orientation::Right => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [false, false, false, false],
                    [true,  true,  true,  true],
                ],
                Orientation::Up | Orientation::Down => [
                    [true, false, false, false],
                    [true, false, false, false],
                    [true, false, false, false],
                    [true, false, false, false],
                ]
            },
            Shape::El => match o {
                Orientation::Up => [
                    [false, false, false, false],
                    [true,  false, false, false],
                    [true,  false, false, false],
                    [true,  true,  false, false],
                ],
                Orientation::Left => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [false, false, true, false],
                    [true,  true,  true, false],
                ],
                Orientation::Down => [
                    [true, true, false, false],
                    [false, true, false, false],
                    [false, true, false, false],
                    [false,  true,  false, false],
                ],
                Orientation::Right => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [true, true, true, false],
                    [true,  false,  false, false],
                ],
            },
            Shape::ElInv => match o {
                Orientation::Up => [
                    [false, false, false, false],
                    [false,  true, false, false],
                    [false,  true, false, false],
                    [true,  true,  false, false],
                ],
                Orientation::Left => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [true, true, true, false],
                    [false,  false,  true, false],
                ],
                Orientation::Down => [
                    [false, false, false, false],
                    [true, true, false, false],
                    [true, false, false, false],
                    [true,  false,  false, false],
                ],
                Orientation::Right => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [true, false, false, false],
                    [true,  true,  true, false],
                ],
            },
            Shape::Square => match o {
                Orientation::Up | Orientation::Down | Orientation::Left | Orientation::Right => [
                    [false, false, false, false],
                    [false,  false, false, false],
                    [true,  true, false, false],
                    [true,  true,  false, false],
                ]
            },
            Shape::Zee => match o {
                Orientation::Up | Orientation::Down => [
                    [false, false, false, false],
                    [true,  false, false, false],
                    [true,  true, false, false],
                    [false,  true,  false, false],
                ],
                Orientation::Left | Orientation::Right => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [false, true, true, false],
                    [true,  true,  false, false],
                ]
            }
            Shape::ZeeInv => match o {
                Orientation::Up | Orientation::Down => [
                    [false, false, false, false],
                    [false,  true, false, false],
                    [true,  true, false, false],
                    [true,  false,  false, false],
                ],
                Orientation::Left | Orientation::Right => [
                    [false, false, false, false],
                    [false, false, false, false],
                    [true, true, false, false],
                    [false,  true,  true, false],
                ]
            }
        }
    }
} 