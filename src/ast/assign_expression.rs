use crate::ast::*;

#[derive(Debug)]
pub struct AssignExpression {
    left: Box<dyn Expression>,
    right: Box<dyn Expression>,
}

impl AssignExpression {
    pub fn new(lhs: Box<dyn Expression>, rhs: Box<dyn Expression>) -> Self {
        Self {
            left: lhs,
            right: rhs,
        }
    }

    pub fn left(&self) -> &dyn Expression {
        self.left.as_ref()
    }

    pub fn right(&self) -> &dyn Expression {
        self.right.as_ref()
    }
}

impl AstNode for AssignExpression {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_assign_expression(self)
    }

    fn accept_mut(&mut self, visitor: &mut dyn AstVisitorMut) {
        visitor.visit_assign_expression_mut(self)
    }
}

impl Expression for AssignExpression {}
