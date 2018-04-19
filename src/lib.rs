extern crate typed_arena;
use typed_arena::Arena;

use std::collections::HashMap;

#[derive(Debug)]
pub enum NodeData<'a> {
    Const(f32),
    Var(String),
    Neg(Node<'a>),
    Add(Node<'a>, Node<'a>),
    Sub(Node<'a>, Node<'a>),
    Mul(Node<'a>, Node<'a>),
    Div(Node<'a>, Node<'a>),
    Pow(Node<'a>, f32),
    Sin(Node<'a>),
    Cos(Node<'a>),
}
type Node<'a> = &'a NodeData<'a>;

pub fn constant<'a>(arena: &'a Arena<NodeData<'a>>, value: f32) -> Node<'a> {
    arena.alloc(NodeData::Const(value))
}

pub fn var<'a>(arena: &'a Arena<NodeData<'a>>, name: String) -> Node<'a> {
    arena.alloc(NodeData::Var(name))
}

pub fn add<'a>(arena: &'a Arena<NodeData<'a>>, lhs: Node<'a>, rhs: Node<'a>) -> Node<'a> {
    arena.alloc(NodeData::Add(lhs, rhs))
}

pub fn sub<'a>(arena: &'a Arena<NodeData<'a>>, lhs: Node<'a>, rhs: Node<'a>) -> Node<'a> {
    arena.alloc(NodeData::Sub(lhs, rhs))
}

pub fn mul<'a>(arena: &'a Arena<NodeData<'a>>, lhs: Node<'a>, rhs: Node<'a>) -> Node<'a> {
    arena.alloc(NodeData::Mul(lhs, rhs))
}

pub fn div<'a>(arena: &'a Arena<NodeData<'a>>, lhs: Node<'a>, rhs: Node<'a>) -> Node<'a> {
    arena.alloc(NodeData::Div(lhs, rhs))
}

pub fn pow<'a>(arena: &'a Arena<NodeData<'a>>, lhs: Node<'a>, rhs: f32) -> Node<'a> {
    arena.alloc(NodeData::Pow(lhs, rhs))
}

pub fn sin<'a>(arena: &'a Arena<NodeData<'a>>, value: Node<'a>) -> Node<'a> {
    arena.alloc(NodeData::Sin(value))
}

pub fn cos<'a>(arena: &'a Arena<NodeData<'a>>, value: Node<'a>) -> Node<'a> {
    arena.alloc(NodeData::Cos(value))
}

pub fn forward<'a>(node: Node<'a>, assignment: &HashMap<String, f32>) -> Option<f32> { 
    use NodeData::*;

    match *node {
        Const(v) => Some(v),
        Var(ref name) => assignment.get(name).cloned(),
        Neg(value) => forward(value, assignment).map(|value| -value),
        Add(lhs, rhs) => forward(lhs, assignment).and_then(|lhs| forward(rhs, assignment).map(|rhs| lhs + rhs)),
        Sub(lhs, rhs) => forward(lhs, assignment).and_then(|lhs| forward(rhs, assignment).map(|rhs| lhs - rhs)),
        Mul(lhs, rhs) => forward(lhs, assignment).and_then(|lhs| forward(rhs, assignment).map(|rhs| lhs * rhs)),
        Div(lhs, rhs) => forward(lhs, assignment).and_then(|lhs| forward(rhs, assignment).map(|rhs| lhs / rhs)),
        Pow(lhs, rhs) => forward(lhs, assignment).map(|lhs| lhs.powf(rhs)),
        Sin(value) => forward(value, assignment).map(f32::sin),
        Cos(value) => forward(value, assignment).map(f32::cos),
    }
}

#[test]
fn basic_forward() {
    let arena = Arena::new();
    let arena = &arena;

    let x = var(arena, "x".into());
    let y = var(arena, "y".into());

    let mul = mul(arena, x, y);
    let div = div(arena, x, y);
    let add = add(arena, mul, div);
    let sub = sub(arena, mul, div);

    let assignment = {
        let mut assignment = HashMap::new();
        assignment.insert("x".into(), 8f32);
        assignment.insert("y".into(), 4f32);
        assignment
    };

    assert_eq!(forward(add, &assignment), Some(34f32));
    assert_eq!(forward(sub, &assignment), Some(30f32));
}

