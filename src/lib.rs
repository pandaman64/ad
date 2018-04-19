extern crate typed_arena;

use std::cell::Cell;
use std::cell::RefCell;
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
    type_: NodeType<'a>,
    value: Cell<f32>,
    grads: RefCell<HashMap<String, f32>>,
}

impl<'a> std::convert::From<NodeType<'a>> for NodeData<'a> {
    fn from(type_: NodeType<'a>) -> Self {
        NodeData {
            type_,
            value: Cell::new(0f32), // bad idea
            grads: RefCell::new(HashMap::new()),
        }
    }
}

impl<'a> NodeData<'a> {
    pub fn reset_grads(&self) {
        use NodeType::*;

        self.grads.borrow_mut().clear();
        match self.type_ {
            Const(_) | Var(_) => {},
            Neg(value) | Pow(value, _) | Sin(value) | Cos(value) => value.reset_grads(),
            Add(lhs, rhs) | Sub(lhs, rhs) | Mul(lhs, rhs) | Div(lhs, rhs) => {
                lhs.reset_grads();
                rhs.reset_grads();
            },
        }
    }

    pub fn backward_ad(&self, variables: &[&str]) {
        use NodeType::*;

        self.reset_grads();

        // we can detect cyclic dependencies if this borrow_mut fails
        let mut grads = self.grads.borrow_mut();

        // we already set gradients
        if grads.len() != 0 {
            return;
        }

        // we may be able to denote grad of x is 0 by just leaving grads[x] empty
        match self.type_ {
            Const(_) => {
                for v in variables {
                    grads.insert(v.to_string(), 0f32);
                }
            },
            Var(ref this) => {
                for v in variables {
                    if this == v {
                        grads.insert(v.to_string(), 1f32);
                    } else {
                        grads.insert(v.to_string(), 0f32);
                    }
                }
            },
            Neg(value) => {
                value.backward_ad(variables);

                for v in variables {
                    grads.insert(v.to_string(), -value.grads.borrow()[*v]);
                }
            },
            Add(lhs, rhs) => {
                lhs.backward_ad(variables);
                rhs.backward_ad(variables);

                for v in variables {
                    grads.insert(v.to_string(), lhs.grads.borrow()[*v] + rhs.grads.borrow()[*v]);
                }
            },
            Sub(lhs, rhs) => {
                lhs.backward_ad(variables);
                rhs.backward_ad(variables);

                for v in variables {
                    grads.insert(v.to_string(), lhs.grads.borrow()[*v] - rhs.grads.borrow()[*v]);
                }
            },
            Mul(lhs, rhs) => {
                lhs.backward_ad(variables);
                rhs.backward_ad(variables);

                for v in variables {
                    grads.insert(v.to_string(), lhs.grads.borrow()[*v] * rhs.value.get() + lhs.value.get() * rhs.grads.borrow()[*v]);
                }
            },
            Div(lhs, rhs) => {
                lhs.backward_ad(variables);
                rhs.backward_ad(variables);

                for v in variables {
                    grads.insert(v.to_string(), (lhs.grads.borrow()[*v] * rhs.value.get() - lhs.value.get() * rhs.grads.borrow()[*v]) / (rhs.value.get().powf(2f32)));
                }
            },
            Pow(lhs, rhs) => {
                lhs.backward_ad(variables);

                for v in variables {
                    grads.insert(v.to_string(), rhs * self.value.get().powf(rhs - 1f32) * lhs.grads.borrow()[*v]);
                }
            },
            Sin(value) => {
                value.backward_ad(variables);

                for v in variables {
                    grads.insert(v.to_string(), self.value.get().cos() * value.grads.borrow()[*v]);
                }
            },
            Cos(value) => {
                value.backward_ad(variables);

                for v in variables {
                    grads.insert(v.to_string(), -self.value.get().sin() * value.grads.borrow()[*v]);
                }
            },
        }
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

    let x = var(arena, "x".to_string());
    let y = var(arena, "y".to_string());

    let mul = mul(arena, x, y);
    let div = div(arena, x, y);
    let add = add(arena, mul, div);
    let sub = sub(arena, mul, div);

    let assignment = {
        let mut assignment = HashMap::new();
        assignment.insert("x".to_string(), 8f32);
        assignment.insert("y".to_string(), 4f32);
        assignment
    };

    assert_eq!(forward(add, &assignment), Some(34f32));
    assert_eq!(forward(sub, &assignment), Some(30f32));
}

#[test]
fn basic_backward_ad() {
    let arena = Arena::new();
    let arena = &arena;

    let x = var(arena, "x".to_string());
    let y = var(arena, "y".to_string());

    let mul = mul(arena, x, y);
    let div = div(arena, x, y);
    let add = add(arena, mul, div);
    let sub = sub(arena, mul, div);

    x.value.set(8f32);
    y.value.set(4f32);

    add.backward_ad(&["x", "y"]);
    sub.backward_ad(&["x", "y"]);

    assert_eq!(add.grads.borrow()["x"], 4.25);
    assert_eq!(add.grads.borrow()["y"], 7.5);

    assert_eq!(sub.grads.borrow()["x"], 3.75);
    assert_eq!(sub.grads.borrow()["y"], 8.5);
}

