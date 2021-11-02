use glam::*;
use std::f32::INFINITY;

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect{x,y,w,h}
    }
    pub fn centroid(&self) -> Vec2 {
        Vec2::new(self.x + self.w/2.0, self.y + self.h/2.0)
    }
    pub fn new_centered(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(x-w/2.0, y-h/2.0, w, h)
    }
    pub fn translate(&self, v: Vec2) -> Rect {
        return Rect::new(self.x + v.x, self.y + v.y, self.w, self.h);
    }
    pub fn dilate(&self, d: f32) -> Rect {
        return Rect::new(self.x - d, self.y - d, self.w + 2.0*d, self.h + 2.0*d);
    }
    pub fn left(self) -> f32 {
        self.x
    }
    pub fn right(self) -> f32 {
        self.x + self.w
    }
    pub fn top(self) -> f32 {
        self.y
    }
    pub fn bot(self) -> f32 {
        self.y + self.h
    }
}

#[test]
fn test_intersection() {
    assert_eq!(rect_intersection(Rect::new(0.0, 0.0, 1.0, 1.0,), Rect::new(0.5, 0.0, 1.0, 1.0)), true);
    assert_eq!(rect_intersection(Rect::new(0.0, 0.0, 1.0, 1.0,), Rect::new(-0.5, 0.0, 1.0, 1.0)), true);
    assert_eq!(rect_intersection(Rect::new(0.0, 0.0, 1.0, 1.0,), Rect::new(0.0, 0.5, 1.0, 1.0)), true);
    assert_eq!(rect_intersection(Rect::new(0.0, 0.0, 1.0, 1.0,), Rect::new(0.0, -0.5, 1.0, 1.0)), true);

    assert_eq!(rect_intersection(Rect::new(0.0, 0.0, 1.0, 1.0,), Rect::new(0.5, -0.05, 0.1, 0.1)), true);
}

// not overlapping if sides kiss
fn overlap_1d(a1: f32, a2: f32, b1: f32, b2: f32) -> bool {
    (b1 > a1 && b1 < a2) ||
    (a1 > b1 && a1 < b2) ||
    a1 == b1 && a2 == b2
}

pub fn rect_intersection(a: Rect, b: Rect) -> bool {
    overlap_1d(a.left(), a.right(), b.left(), b.right()) &&
    overlap_1d(a.top(), a.bot(), b.top(), b.bot())
}

// 5 cases: both a in b, both b in a, a left in b, b left in a, no overlap
fn overlap_amount(a1: f32, a2: f32, b1: f32, b2: f32) -> f32 {
    let a1_in_b = a1 > b1 && a1 < b2;
    let a2_in_b = a2 > b1 && a2 < b2;
    let b1_in_a = b1 > a1 && b1 < a2;
    let b2_in_a = b2 > a1 && b2 < a2;

    if !a1_in_b && !a2_in_b && !b1_in_a && !b2_in_a {return 0.0;} // no overlap
    if a1_in_b && a2_in_b {return a2 - a1;} // a fully within b // maybe better to do distance to outside still
    if b1_in_a && b2_in_a {return b2 - b1;} // b fully within a
    if a1_in_b {return b2 - a1;} // a to right of b
    if b1_in_a {return -(a2 - b1);} // b to right of a

    panic!("unhandled overlap case");
}

pub fn least_penetration(a: Rect, b: Rect) -> Vec2 {
    let x_overlap = overlap_amount(a.left(), a.right(), b.left(), b.right());
    let y_overlap = overlap_amount(a.top(), a.bot(), b.top(), b.bot());

    if x_overlap.abs() < y_overlap.abs() {
        Vec2::new(x_overlap, 0.0)
    } else {
        Vec2::new(0.0, y_overlap)
    }
}

#[test]
fn test_least_pen() {
    let r1 = Rect::new(0.0, 0.0, 1.0, 1.0);
    let r2 = Rect::new(0.9, 0.8, 1.0, 1.0);

    assert_eq!(least_penetration(r1, r2), Vec2::new(-0.1, 0.0));
    assert_eq!(least_penetration(r2, r1), Vec2::new(0.1, 0.0));
}

