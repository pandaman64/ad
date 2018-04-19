extern crate typed_arena;

use std::collections::HashMap;

#[derive(Debug)]
pub enum NodeType<'a> {
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

#[derive(Debug)]
pub struct NodeData<'a> {
    type_: NodeType<'a>
}

impl<'a> std::convert::From<NodeType<'a>> for NodeData<'a> {
    fn from(type_: NodeType<'a>) -> Self {
        NodeData { type_ }
    }
}

type Node<'a> = &'a NodeData<'a>;
type Arena<'a> = typed_arena::Arena<NodeData<'a>>;

pub fn constant<'a>(arena: &'a Arena<'a>, value: f32) -> Node<'a> {
    arena.alloc(NodeType::Const(value).into())
}

pub fn var<'a>(arena: &'a Arena<'a>, name: String) -> Node<'a> {
    arena.alloc(NodeType::Var(name).into())
}

pub fn add<'a>(arena: &'a Arena<'a>, lhs: Node<'a>, rhs: Node<'a>) -> Node<'a> {
    arena.alloc(NodeType::Add(lhs, rhs).into())
}

pub fn sub<'a>(arena: &'a Arena<'a>, lhs: Node<'a>, rhs: Node<'a>) -> Node<'a> {
    arena.alloc(NodeType::Sub(lhs, rhs).into())
}

pub fn mul<'a>(arena: &'a Arena<'a>, lhs: Node<'a>, rhs: Node<'a>) -> Node<'a> {
    arena.alloc(NodeType::Mul(lhs, rhs).into())
}

pub fn div<'a>(arena: &'a Arena<'a>, lhs: Node<'a>, rhs: Node<'a>) -> Node<'a> {
    arena.alloc(NodeType::Div(lhs, rhs).into())
}

pub fn pow<'a>(arena: &'a Arena<'a>, lhs: Node<'a>, rhs: f32) -> Node<'a> {
    arena.alloc(NodeType::Pow(lhs, rhs).into())
}

pub fn sin<'a>(arena: &'a Arena<'a>, value: Node<'a>) -> Node<'a> {
    arena.alloc(NodeType::Sin(value).into())
}

pub fn cos<'a>(arena: &'a Arena<'a>, value: Node<'a>) -> Node<'a> {
    arena.alloc(NodeType::Cos(value).into())
}

pub fn forward<'a>(node: Node<'a>, assignment: &HashMap<String, f32>) -> Option<f32> { 
    use NodeType::*;

    match node.type_ {
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

