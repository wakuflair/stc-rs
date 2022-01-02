use crate::ast::*;
use std::rc::Rc;

#[derive(Debug)]
pub struct AssignExpression {
    left: Expression,
    right: Expression,
    ty: Option<Rc<Box<dyn Type>>>,
}

impl AssignExpression {
    pub fn new(lhs: Expression, rhs: Expression) -> Self {
        Self {
            left: lhs,
            right: rhs,
            ty: None,
        }
    }

    pub fn left(&self) -> &Expression {
        &self.left
    }

    pub fn left_mut(&mut self) -> &mut Expression {
        &mut self.left
    }

    pub fn right(&self) -> &Expression {
        &self.right
    }

    pub fn right_mut(&mut self) -> &mut Expression {
        &mut self.right
    }

    pub fn ty(&self) -> Option<Rc<Box<dyn Type>>> {
        self.ty.clone()
    }

    pub fn set_ty(&mut self, ty: Option<Rc<Box<dyn Type>>>) {
        self.ty = ty
    }
}

// impl AstNode for AssignExpression {
//     fn as_any(&self) -> &dyn Any {
//         self
//     }
//
//     fn accept(&self, visitor: &mut dyn AstVisitor) {
//         visitor.visit_assign_expression(self)
//     }
//
//     fn accept_mut(&mut self, visitor: &mut dyn AstVisitorMut) {
//         visitor.visit_assign_expression_mut(self)
//     }
// }
//
// impl Expression for AssignExpression {}
