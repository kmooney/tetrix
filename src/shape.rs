use rand::Rng;
use std::marker::Copy;


#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: usize, 
    pub y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Point {
        Point {x: x, y: y}
    }

}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

pub type ShapeMat = [[Option<Shape>; 4]; 4];

#[derive(Clone, Copy, PartialEq, Debug)]
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

    pub fn to_mat(&self, o: Orientation) -> ShapeMat {
        match self {
            Shape::Tee => match o {
                Orientation::Up => [
                    [None, None, None, None],
                    [None, None, None, None],
                    [None, Some(Shape::Tee), None, None],
                    [Some(Shape::Tee), Some(Shape::Tee), Some(Shape::Tee), None],
                ],
                Orientation::Down => [
                    [None, None, None, None],
                    [None, None, None, None],
                    [Some(Shape::Tee), Some(Shape::Tee), Some(Shape::Tee), None],
                    [None, Some(Shape::Tee), None, None],
                ],
                Orientation::Left => [
                    [None, None, None, None],
                    [None, Some(Shape::Tee), None, None],
                    [Some(Shape::Tee), Some(Shape::Tee), None, None],
                    [None, Some(Shape::Tee), None, None],
                ],
                Orientation::Right => [
                    [None, None, None, None],
                    [Some(Shape::Tee), None, None, None],
                    [Some(Shape::Tee), Some(Shape::Tee), None, None],
                    [Some(Shape::Tee), None, None, None],
                ],
            },
            Shape::Eye => match o {
                Orientation::Left | Orientation::Right => [
                    [None, None, None, None],
                    [None, None, None, None],
                    [None, None, None, None],
                    [Some(Shape::Eye),  Some(Shape::Eye),  Some(Shape::Eye),  Some(Shape::Eye)],
                ],
                Orientation::Up | Orientation::Down => [
                    [Some(Shape::Eye), None, None, None],
                    [Some(Shape::Eye), None, None, None],
                    [Some(Shape::Eye), None, None, None],
                    [Some(Shape::Eye), None, None, None],
                ]
            },
            Shape::El => match o {
                Orientation::Up => [
                    [None, None, None, None],
                    [Some(Shape::El),  None, None, None],
                    [Some(Shape::El),  None, None, None],
                    [Some(Shape::El),  Some(Shape::El),  None, None],
                ],
                Orientation::Left => [
                    [None, None, None, None],
                    [None, None, None, None],
                    [None, None, Some(Shape::El), None],
                    [Some(Shape::El),  Some(Shape::El),  Some(Shape::El), None],
                ],
                Orientation::Down => [
                    [Some(Shape::El), Some(Shape::El), None, None],
                    [None, Some(Shape::El), None, None],
                    [None, Some(Shape::El), None, None],
                    [None,  None,  None, None],
                ],
                Orientation::Right => [
                    [None, None, None, None],
                    [None, None, None, None],
                    [Some(Shape::El), Some(Shape::El), Some(Shape::El), None],
                    [Some(Shape::El),  None,  None, None],
                ],
            },
            Shape::ElInv => match o {
                Orientation::Up => [
                    [None, None, None, None],
                    [None,  Some(Shape::ElInv), None, None],
                    [None,  Some(Shape::ElInv), None, None],
                    [Some(Shape::ElInv),  Some(Shape::ElInv),  None, None],
                ],
                Orientation::Left => [
                    [None, None, None, None],
                    [None, None, None, None],
                    [Some(Shape::ElInv), Some(Shape::ElInv), Some(Shape::ElInv), None],
                    [None,  None,  Some(Shape::ElInv), None],
                ],
                Orientation::Down => [
                    [None, None, None, None],
                    [Some(Shape::ElInv), Some(Shape::ElInv), None, None],
                    [Some(Shape::ElInv), None, None, None],
                    [Some(Shape::ElInv),  None,  None, None],
                ],
                Orientation::Right => [
                    [None, None, None, None],
                    [None, None, None, None],
                    [Some(Shape::ElInv), None, None, None],
                    [Some(Shape::ElInv),  Some(Shape::ElInv),  Some(Shape::ElInv), None],
                ],
            },
            Shape::Square => match o {
                Orientation::Up | Orientation::Down | Orientation::Left | Orientation::Right => [
                    [None, None, None, None],
                    [None,  None, None, None],
                    [Some(Shape::Square),  Some(Shape::Square), None, None],
                    [Some(Shape::Square),  Some(Shape::Square),  None, None],
                ]
            },
            Shape::ZeeInv => match o {
                Orientation::Up | Orientation::Down => [
                    [None, None, None, None],
                    [Some(Shape::ZeeInv),  None, None, None],
                    [Some(Shape::ZeeInv),  Some(Shape::ZeeInv), None, None],
                    [None,  Some(Shape::ZeeInv),  None, None],
                ],
                Orientation::Left | Orientation::Right => [
                    [None, None, None, None],
                    [None, None, None, None],
                    [None, Some(Shape::ZeeInv), Some(Shape::ZeeInv), None],
                    [Some(Shape::ZeeInv),  Some(Shape::ZeeInv),  None, None],
                ]
            }
            Shape::Zee => match o {
                Orientation::Up | Orientation::Down => [
                    [None, None, None, None],
                    [None,  Some(Shape::Zee), None, None],
                    [Some(Shape::Zee),  Some(Shape::Zee), None, None],
                    [Some(Shape::Zee),  None,  None, None],
                ],
                Orientation::Left | Orientation::Right => [
                    [None, None, None, None],
                    [None, None, None, None],
                    [Some(Shape::Zee), Some(Shape::Zee), None, None],
                    [None,  Some(Shape::Zee),  Some(Shape::Zee), None],
                ]
            }
        }
    }
} 