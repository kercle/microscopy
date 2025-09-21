#![no_std]

pub enum Axes {
    X,
    Y,
    Z,
}

pub enum Commands {
    MoveSteps { axis: Axes, steps: i32 },
}

pub fn test() {}
