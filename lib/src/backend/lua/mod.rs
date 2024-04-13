/// Lua ByteCode object wrapper
mod bytecode;
use bytecode::*;

pub mod dump;
mod register;
mod utils;
mod vm;

use crate::backend::lua::register::{RegisterId, RegisterManager};
use crate::backend::lua::utils::*;
use crate::backend::*;
use crate::parser::{LiteralValue, Operator};
use crate::prelude::*;

use indexmap::IndexSet;
use log::*;
use smallvec::{smallvec, SmallVec};
use std::mem;
use std::rc::Rc;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct LuaAccessMode: u32 {
        const NONE              = 0b0000_0000_0000_0000;
        const READ              = 0b0000_0000_0000_0001;
        const WRITE             = 0b0000_0000_0000_0010;
        const PARAMETER         = 0b0000_0000_0000_0100;
        const CALL              = 0b0000_0000_0000_1000;
    }
}

#[derive(Clone)]
pub struct LuaBackendStates {
    variable: Option<Rc<Variable>>,
    register: Option<RegisterId>,
    constant_index: Option<usize>,
    scope: Option<Scope>,
    error: bool,
    access_mode: LuaAccessMode,
}

impl Default for LuaBackendStates {
    fn default() -> Self {
        Self {
            variable: None,
            register: None,
            scope: None,
            error: false,
            access_mode: LuaAccessMode::NONE,
            constant_index: None,
        }
    }
}

pub struct LuaBackend {
    mgr: UnitsManager,
    app: ModuleContext,
    byte_codes: Vec<LuaByteCode>,
    states: SmallVec<[LuaBackendStates; 32]>,
    local_function: Option<Function>,
    local_proto: Option<Prototype>,
    constants: IndexSet<LuaConstants>,
    reg_mgr: RegisterManager,
    module_upvalues: SmallVec<[LuaConstants; 32]>,
}

impl LuaBackend {
    fn push_code(&mut self, code: LuaByteCode) {
        debug!("LuaBackend: Push Code {:?}", code);
        self.byte_codes.push(code)
    }

    fn current_application(&self) -> ModuleContext {
        self.app.clone()
    }

    fn module_upvalues(&self) -> &SmallVec<[LuaConstants; 32]> {
        &self.module_upvalues
    }

    fn push_attribute_with_scope(&mut self, scope: Scope) {
        let attr = LuaBackendStates {
            scope: Some(scope),
            ..Default::default()
        };

        self.states.push(attr)
    }

    fn push_access_attribute(&mut self, access: LuaAccessMode) {
        let attr = LuaBackendStates {
            scope: self.top_attribute().scope.clone(),
            access_mode: access,
            ..Default::default()
        };

        self.states.push(attr)
    }

    fn push_default_attribute(&mut self) {
        let attr = LuaBackendStates {
            scope: self.top_attribute().scope.clone(),
            ..Default::default()
        };

        self.states.push(attr);
    }

    fn pop_attribute(&mut self) -> LuaBackendStates {
        self.states.pop().unwrap()
    }

    fn top_attribute(&mut self) -> &mut LuaBackendStates {
        self.states.last_mut().unwrap()
    }

    /// Return current scope, will be panic if scope not set
    fn current_scope(&mut self) -> Scope {
        self.top_attribute().clone().scope.unwrap()
    }

    fn add_string_constant<S: AsRef<str>>(&mut self, s: S) -> usize {
        let constant = LuaConstants::String(s.as_ref().to_owned());
        let (idx, _inserted) = self.constants.insert_full(constant);
        idx
    }

    fn add_integer_constant(&mut self, i: i64) -> usize {
        let constant = LuaConstants::Integer(i);
        let (idx, _inserted) = self.constants.insert_full(constant);
        idx
    }

    fn add_float_constant(&mut self, f: f64) -> usize {
        let constant = LuaConstants::Float(f);
        let (idx, _inserted) = self.constants.insert_full(constant);
        idx
    }
}

impl CodeGenBackend for LuaBackend {
    type Label = usize;

    fn new(mgr: UnitsManager, app: ModuleContext) -> Self {
        Self {
            mgr,
            app,
            byte_codes: vec![],
            states: smallvec![],
            local_function: None,
            local_proto: None,
            constants: IndexSet::new(),
            reg_mgr: RegisterManager::new(),
            module_upvalues: smallvec![],
        }
    }

    fn get_module_bytes(&mut self, w: &mut dyn Write) -> io::Result<()> {
        lua_dump_module(self, w)
    }

    fn gen_function(&mut self, func: usize) -> Result<Box<dyn CompiledCode>, CodeGenError> {
        let app = self.app.read();
        let f = app
            .get_function(func)
            .ok_or(CodeGenError::FunctionNotDefined(func))?
            .clone();
        let p = app.get_declaration_by_id(func).cloned();

        self.local_function = Some(f.clone());
        self.local_proto = p;
        drop(app);

        let app_id = self.app.read().id();
        let fun_scope = Scope::new(Some(self.mgr.clone()), Some(app_id), Some(func));

        // generate VarArgPrep
        if let Some(p) = &self.local_proto {
            if is_vararg(p) {
                self.push_code(LuaByteCode::VarArgPrep(0));
            }
        }

        let mut fun = f.write();
        self.push_attribute_with_scope(fun_scope);
        self.visit_statement_mut(fun.parse_tree_mut());
        self.pop_attribute();

        // generate return
        self.push_code(LuaByteCode::Return(false, 0, 1, 1));

        let byte_codes = mem::take(&mut self.byte_codes);
        let constants = mem::replace(&mut self.constants, IndexSet::new());

        Ok(Box::new(LuaCompiledCode {
            byte_codes,
            constants,
            // TODO:: upvalues
            upvalues: smallvec![LuaUpValue {
                name: None,
                stack: 1,
                index: 0,
                kind: 0
            }],
        }))
    }

    fn define_label<S: AsRef<str>>(&mut self, label: Option<S>) -> Self::Label {
        0
    }

    fn gen_variable_load(&mut self, variable: &mut Variable) {
        todo!()
    }

    fn gen_operator(&mut self, operator: &mut OperatorExpression) {
        let operands = operator.operands_mut();

        self.visit_expression_mut(&mut operands[0]);
    }
}

impl AstVisitorMut for LuaBackend {
    fn visit_literal_mut(&mut self, literal: &mut LiteralExpression) {
        trace!("LuaGen: literal expression: {:?}", literal);

        // Literals can't WRITE
        assert!(!self
            .top_attribute()
            .access_mode
            .contains(LuaAccessMode::WRITE));

        // if literal can use LoadI instructions
        if let Some(v) = try_fit_sbx(literal.literal()) {
            let r = self
                .top_attribute()
                .register
                .unwrap_or_else(|| self.reg_mgr.alloc());
            self.push_code(LuaByteCode::LoadI(r as u8, v));
            self.top_attribute().register = Some(r);
            return;
        }

        match literal.literal() {
            LiteralValue::String(s) => {
                let constant_index = self.add_string_constant(s);
                self.top_attribute().constant_index = Some(constant_index);
            }
            LiteralValue::DInt(i) => {
                let constant_index = self.add_integer_constant(*i as i64);
                self.top_attribute().constant_index = Some(constant_index);
            }
            LiteralValue::Real(s) | LiteralValue::LReal(s) => {
                let f: f64 = s.parse().unwrap();
                let constant_index = self.add_float_constant(f);
                self.top_attribute().constant_index = Some(constant_index);
            }
            _ => {}
        }
    }

    fn visit_variable_expression_mut(&mut self, var_expr: &mut VariableExpression) {
        let scope = self.current_scope();
        let var = scope.find_variable(var_expr.name());

        trace!(
            "LuaGen: variable expression: {}: {:?}",
            var_expr,
            var.and_then(|x| x.ty())
        );

        let access_mode = self.top_attribute().access_mode;
        match access_mode & (LuaAccessMode::READ | LuaAccessMode::WRITE | LuaAccessMode::CALL) {
            // Callee process
            LuaAccessMode::CALL => {
                self.top_attribute().constant_index =
                    Some(self.add_string_constant(var_expr.org_name()))
            }
            // Write register into stack
            LuaAccessMode::WRITE => {}
            // Load into register
            LuaAccessMode::READ => {
                let scope = self.top_attribute().scope.as_ref().unwrap();
                if let Some(variable) = scope.find_variable(var_expr.name()) {
                    let reg = self.reg_mgr.alloc();
                    self.top_attribute().register = Some(reg);

                    // TODO: initialize
                    self.push_code(LuaByteCode::LoadI(reg as u8, 0));
                } else {
                    // TODO: variable not found error
                }
            }
            _ => unreachable!("{:?}", access_mode),
        }
    }

    fn visit_call_expression_mut(&mut self, call: &mut CallExpression) {
        trace!("LuaGen: call expression: {}", call);

        self.push_access_attribute(LuaAccessMode::CALL);
        self.visit_expression_mut(call.callee_mut());
        let callee_index = self.top_attribute().constant_index;
        self.pop_attribute();
        self.push_code(LuaByteCode::GetTabUp(0, 0, 0));

        // visit all arguments
        for arg in call.arguments_mut() {
            self.push_access_attribute(LuaAccessMode::PARAMETER);
            self.visit_expression_mut(arg);
            let arg_value_index = self.top_attribute().constant_index;
            self.pop_attribute();

            // Load argument
            if let Some(idx) = arg_value_index {
                self.push_code(LuaByteCode::LoadK(0, idx as u32));
            }
        }

        self.push_code(LuaByteCode::Call(
            callee_index.unwrap() as u8,
            call.arguments().len() as u8,
            0,
        ))
    }

    fn visit_if_statement_mut(&mut self, ifst: &mut IfStatement) {
        trace!("LuaGen: if statement: {}", ifst.condition());

        let cond_true = self.define_label(Some("if-true"));
        let cond_false = self.define_label(Some("if-false"));

        self.visit_expression_mut(ifst.condition_mut());

        if let Some(then_ctrl) = ifst.then_controlled_mut() {
            self.visit_statement_mut(then_ctrl);
        }
    }

    fn visit_operator_expression_mut(&mut self, operator: &mut OperatorExpression) {
        trace!("LuaGen: operator expression: {}", operator);

        let op = *operator.op();
        let operands = operator.operands_mut();

        match op {
            // binary operators
            Operator::Less
            | Operator::Plus
            | Operator::LessEqual
            | Operator::Equal
            | Operator::NotEqual
            | Operator::Greater
            | Operator::GreaterEqual => {
                let dest_reg = self
                    .top_attribute()
                    .register
                    .unwrap_or_else(|| self.reg_mgr.alloc());

                self.push_access_attribute(LuaAccessMode::READ);
                self.visit_expression_mut(&mut operands[0]);
                let op0_reg = self.pop_attribute().register.unwrap();

                self.push_access_attribute(LuaAccessMode::READ);
                self.visit_expression_mut(&mut operands[1]);
                let op1_reg = self.pop_attribute().register.unwrap();

                // generate operators
                match op {
                    // a + b
                    Operator::Plus => self.push_code(LuaByteCode::Add(
                        dest_reg as u8,
                        op0_reg as u8,
                        op1_reg as u8,
                    )),
                    // a = b
                    Operator::Equal => {
                        // (op0 == op1) != 1
                        self.push_code(LuaByteCode::Eq(op0_reg as u8, op1_reg as u8, 1))
                    }
                    _ => unreachable!(),
                }

                self.reg_mgr.free(&op0_reg);
                self.reg_mgr.free(&op1_reg);
                self.top_attribute().register = Some(dest_reg);
            }

            _ => unreachable!(),
        }
    }

    fn visit_assign_expression_mut(&mut self, assign: &mut AssignExpression) {
        trace!("LuaGen: assignment expression: {}", assign);

        self.push_access_attribute(LuaAccessMode::READ);
        assign.left_mut().accept_mut(self);
        let lhs_reg = self.pop_attribute().register;

        self.push_access_attribute(LuaAccessMode::READ);
        self.top_attribute().register = lhs_reg;
        assign.right_mut().accept_mut(self);
        let rhs = self.pop_attribute();

        self.push_access_attribute(LuaAccessMode::WRITE);
        self.top_attribute().register = lhs_reg;
        assign.left_mut().accept_mut(self);
        let lhs = self.pop_attribute();

        // free temporary registers
        if let Some(r) = rhs.register {
            // if rhs register is reused lhs_reg, ignore move
            if Some(r) != lhs_reg {
                self.push_code(LuaByteCode::Move(lhs.register.unwrap() as u8, r as u8));

                self.reg_mgr.free(&r)
            }
        }
    }
}
