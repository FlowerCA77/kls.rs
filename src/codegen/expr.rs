use std::collections::HashMap;

use inkwell::FloatPredicate;
use inkwell::module::Linkage;
use inkwell::values::{BasicValue, BasicValueEnum, FastMathFlags, FloatValue, ValueKind};

use crate::Result;
use crate::codegen::{BindingValue, Codegen};
use crate::frontend::ast::{Binding, ExprAST, FunctionName};

impl<'ctx> Codegen<'ctx> {
    fn compile_number(&mut self, n: f64) -> Result<BasicValueEnum<'ctx>> {
        Ok(self.context.f64_type().const_float(n).into())
    }

    fn compile_variable(&mut self, name: &str) -> Result<BasicValueEnum<'ctx>> {
        let binding = self.lookup_variable(name).ok_or(format!("unknown variable {}", name))?;

        match binding {
            BindingValue::Value(v) => Ok(v),
            BindingValue::Alloca(ptr) => Ok(self.builder.build_load(self.context.f64_type(), ptr, name)?),
            BindingValue::Global => {
                let g = if let Some(g) = self.module.get_global(name) {
                    g
                } else {
                    let g = self.module.add_global(self.context.f64_type(), None, name);
                    g.set_linkage(Linkage::External);
                    g
                };
                let v = self
                    .builder
                    .build_load(self.context.f64_type(), g.as_pointer_value(), name)?;
                Ok(v.into())
            }
        }
    }

    fn compile_unary(&mut self, op: &str, operand: &ExprAST) -> Result<BasicValueEnum<'ctx>> {
        let v = self.compile_expr(operand)?.into_float_value();

        let value = match op {
            "+" => v,

            "-" => {
                let zero = self.context.f64_type().const_float(0.0);
                self.builder.build_float_sub(zero, v, "negtmp")?
            }

            _ => {
                let llvm_name = FunctionName::Unary(op.to_string()).llvm_name();

                let function = if let Some(f) = self.module.get_function(&llvm_name) {
                    f
                } else if let Some(proto) = self.function_protos.get(&llvm_name).cloned() {
                    self.compile_prototype(&proto)?
                } else {
                    return Err(format!("undefined unary operator: {}", op).into());
                };

                let call = self.builder.build_call(function, &[v.into()], "optmp")?;

                match call.try_as_basic_value() {
                    ValueKind::Basic(bv) => bv.into_float_value(),
                    ValueKind::Instruction(_) => {
                        return Err(format!("operator {} returns void", op).into());
                    }
                }
            }
        };
        Ok(value.into())
    }

    fn compile_comparison(
        &self,
        predicate: FloatPredicate,
        x: FloatValue<'ctx>,
        y: FloatValue<'ctx>,
        name: &str,
    ) -> Result<FloatValue<'ctx>> {
        let compare_float = self.builder.build_unsigned_int_to_float(
            self.builder.build_float_compare(predicate, x, y, name)?,
            self.context.f64_type(),
            "booltmp",
        )?;

        if self.options.fast_math {
            return Ok(compare_float);
        }

        Ok(self
            .builder
            .build_select(
                self.builder.build_float_compare(FloatPredicate::UNO, x, y, "isnan")?,
                self.context.f64_type().const_float(f64::NAN),
                compare_float,
                "cmp_result",
            )?
            .into_float_value())
    }

    fn compile_binary(&mut self, op: &str, lhs: &ExprAST, rhs: &ExprAST) -> Result<BasicValueEnum<'ctx>> {
        let x = self.compile_expr(lhs)?.into_float_value();
        let y = self.compile_expr(rhs)?.into_float_value();

        let value: FloatValue = match op {
            "+" => self.builder.build_float_add(x, y, "addtmp")?.into(),
            "-" => self.builder.build_float_sub(x, y, "subtmp")?.into(),
            "*" => self.builder.build_float_mul(x, y, "multmp")?.into(),
            "/" => self.builder.build_float_div(x, y, "divtmp")?.into(),

            "<" => self.compile_comparison(FloatPredicate::OLT, x, y, "lttmp")?.into(),
            "<=" => self.compile_comparison(FloatPredicate::OLE, x, y, "leptmp")?.into(),
            ">" => self.compile_comparison(FloatPredicate::OGT, x, y, "gttmp")?.into(),
            ">=" => self.compile_comparison(FloatPredicate::OGE, x, y, "getmp")?.into(),
            "==" => self.compile_comparison(FloatPredicate::OEQ, x, y, "eqtmp")?.into(),
            "!=" => self.compile_comparison(FloatPredicate::UNE, x, y, "netmp")?.into(),

            "<=>" => self
                .builder
                .build_select(
                    self.builder.build_float_compare(FloatPredicate::UNO, x, y, "arenan")?,
                    self.context.f64_type().const_float(f64::NAN),
                    self.builder
                        .build_select(
                            self.builder.build_float_compare(FloatPredicate::OGT, x, y, "isgt")?,
                            self.context.f64_type().const_float(1.0),
                            self.builder
                                .build_select(
                                    self.builder.build_float_compare(FloatPredicate::OLT, x, y, "islt")?,
                                    self.context.f64_type().const_float(-1.0),
                                    self.context.f64_type().const_float(0.0),
                                    "ltoeq",
                                )?
                                .into_float_value(),
                            "isordered",
                        )?
                        .into_float_value(),
                    "tricmp",
                )?
                .into_float_value(),

            _ => {
                let llvm_name = FunctionName::Binary(op.to_string()).llvm_name();

                let function = if let Some(f) = self.module.get_function(&llvm_name) {
                    f
                } else if let Some(proto) = self.function_protos.get(&llvm_name).cloned() {
                    self.compile_prototype(&proto)?
                } else {
                    return Err(format!("undefined binary operator: {}", op).into());
                };

                let call = self.builder.build_call(function, &[x.into(), y.into()], "optmp")?;

                match call.try_as_basic_value() {
                    ValueKind::Basic(v) => v.into_float_value(),
                    ValueKind::Instruction(_) => {
                        return Err(format!("operator {} returns void", op).into());
                    }
                }
            }
        };

        if self.options.fast_math {
            if let Some(inst) = value.as_instruction_value() {
                inst.set_fast_math_flags(FastMathFlags::all())?;
            }
        }

        Ok(BasicValueEnum::FloatValue(value))
    }

    fn compile_call(&mut self, callee: &str, args: &[ExprAST]) -> Result<BasicValueEnum<'ctx>> {
        let function = if let Some(f) = self.module.get_function(callee) {
            f
        } else if let Some(proto) = self.function_protos.get(callee).cloned() {
            self.compile_prototype(&proto)?
        } else {
            return Err(format!("unknown function: {}", callee).into());
        };

        if function.count_params() as usize != args.len() {
            return Err(format!(
                "{} expects {} arguments, got {}",
                callee,
                function.count_params(),
                args.len()
            )
            .into());
        }

        let compiled_args: Result<Vec<_>> = args.iter().map(|arg| self.compile_expr(arg).map(Into::into)).collect();
        let compiled_args = compiled_args?;

        let call_site = self.builder.build_call(function, &compiled_args, "calltmp")?;

        match call_site.try_as_basic_value() {
            ValueKind::Basic(basic_value) => Ok(basic_value),
            ValueKind::Instruction(_) => Err(format!("{} returns void", callee).into()),
        }
    }

    fn compile_if(&mut self, cond: &ExprAST, e_true: &ExprAST, e_false: &ExprAST) -> Result<BasicValueEnum<'ctx>> {
        let cond_val = self.compile_expr(cond)?.into_float_value();
        let zero = self.context.f64_type().const_float(0.0);
        let cmp = self
            .builder
            .build_float_compare(FloatPredicate::ONE, cond_val, zero, "ifcond")?;

        let function = self
            .builder
            .get_insert_block()
            .ok_or("llvm error")?
            .get_parent()
            .ok_or("llvm error")?;

        let then_bb = self.context.append_basic_block(function, "then");
        let else_bb = self.context.append_basic_block(function, "else");
        let ifcont_bb = self.context.append_basic_block(function, "ifcont");

        self.builder.build_conditional_branch(cmp, then_bb, else_bb)?;

        self.builder.position_at_end(then_bb);
        let then_val = self.compile_expr(e_true)?.into_float_value();
        self.builder.build_unconditional_branch(ifcont_bb)?;
        let then_bb = self.builder.get_insert_block().ok_or("llvm error")?;

        self.builder.position_at_end(else_bb);
        let else_val = self.compile_expr(e_false)?.into_float_value();
        self.builder.build_unconditional_branch(ifcont_bb)?;
        let else_bb = self.builder.get_insert_block().ok_or("llvm error")?;

        self.builder.position_at_end(ifcont_bb);
        let phi = self.builder.build_phi(self.context.f64_type(), "iftmp")?;
        phi.add_incoming(&[(&then_val, then_bb), (&else_val, else_bb)]);

        Ok(phi.as_basic_value())
    }

    fn compile_for(
        &mut self,
        var: &str,
        e_init: &ExprAST,
        e_cond: &ExprAST,
        e_step: &ExprAST,
        e_body: &ExprAST,
    ) -> Result<BasicValueEnum<'ctx>> {
        let init_val = self.compile_expr(e_init)?.into_float_value();

        let function = self
            .builder
            .get_insert_block()
            .ok_or("llvm error")?
            .get_parent()
            .ok_or("llvm error")?;
        let preheader_bb = self.builder.get_insert_block().ok_or("llvm error")?;

        let loop_bb = self.context.append_basic_block(function, "loop");
        self.builder.build_unconditional_branch(loop_bb)?;

        self.builder.position_at_end(loop_bb);
        let variable = self.builder.build_phi(self.context.f64_type(), var)?;
        variable.add_incoming(&[(&init_val, preheader_bb)]);
        let forbody = self.builder.build_phi(self.context.f64_type(), "forbody")?;
        forbody.add_incoming(&[(&self.context.f64_type().const_float(f64::NAN), preheader_bb)]);

        self.scopes.push(HashMap::new());
        self.scopes
            .last_mut()
            .ok_or("scope error")?
            .insert(var.to_string(), BindingValue::Value(variable.as_basic_value()));

        let cond_val = self.compile_expr(e_cond)?.into_float_value();
        let zero = self.context.f64_type().const_float(0.0);
        let cmp = self
            .builder
            .build_float_compare(FloatPredicate::ONE, cond_val, zero, "loopcond")?;

        let body_bb = self.context.append_basic_block(function, "loopbody");
        let after_bb = self.context.append_basic_block(function, "afterloop");
        self.builder.build_conditional_branch(cmp, body_bb, after_bb)?;

        self.builder.position_at_end(body_bb);

        let body_val = self.compile_expr(e_body)?.into_float_value();
        let step_val = self.compile_expr(e_step)?.into_float_value();

        let body_end_bb = self.builder.get_insert_block().ok_or("llvm error")?;
        self.builder.build_unconditional_branch(loop_bb)?;
        variable.add_incoming(&[(&step_val, body_end_bb)]);
        forbody.add_incoming(&[(&body_val, body_end_bb)]);

        self.scopes.pop();

        self.builder.position_at_end(after_bb);

        Ok(forbody.as_basic_value())
    }

    fn compile_block(&mut self, exprs: &[ExprAST]) -> Result<BasicValueEnum<'ctx>> {
        self.scopes.push(HashMap::new());

        let mut last = None;

        for expr in exprs {
            last = Some(self.compile_expr(expr)?);
        }

        self.scopes.pop();

        Ok(last.unwrap_or_else(|| self.context.f64_type().const_float(0.0).into()))
    }

    pub fn compile_let(&mut self, bindings: &[Binding]) -> Result<BasicValueEnum<'ctx>> {
        for binding in bindings {
            let init_val = match &binding.init {
                Some(e) => self.compile_expr(e)?.into_float_value(),
                None => self.context.f64_type().const_float(f64::NAN),
            };

            let alloca = self.create_entry_block_alloca(&binding.name)?;
            self.builder.build_store(alloca, init_val)?;
            self.scopes
                .last_mut()
                .ok_or("try to define a local let-binding in global scope")?
                .insert(binding.name.clone(), BindingValue::Alloca(alloca));
        }

        Ok(self.context.f64_type().const_float(0.0).into())
    }

    pub fn compile_letin(&mut self, bindings: &[Binding], body: &ExprAST) -> Result<BasicValueEnum<'ctx>> {
        self.scopes.push(HashMap::new());

        for binding in bindings {
            let init_val = match &binding.init {
                Some(e) => self.compile_expr(e)?.into_float_value(),
                None => self.context.f64_type().const_float(f64::NAN),
            };

            let alloca = self.create_entry_block_alloca(&binding.name)?;
            self.builder.build_store(alloca, init_val)?;
            self.scopes
                .last_mut()
                .ok_or("compile error")?
                .insert(binding.name.clone(), BindingValue::Alloca(alloca));
        }

        let result = self.compile_expr(body)?;
        self.scopes.pop();
        Ok(result)
    }

    fn compile_assign(&mut self, name: &str, value: &ExprAST) -> Result<BasicValueEnum<'ctx>> {
        let binding = self
            .lookup_variable(name)
            .ok_or(format!("unknown variable: {}", name))?;

        match binding {
            BindingValue::Alloca(ptr) => {
                let value = self.compile_expr(value)?.into_float_value();
                self.builder.build_store(ptr, value)?;
                Ok(value.into())
            }
            BindingValue::Value(_) => Err(format!("cannot assign to temporary binding {}", name).into()),
            BindingValue::Global => {
                let g = if let Some(g) = self.module.get_global(name) {
                    g
                } else {
                    let g = self.module.add_global(self.context.f64_type(), None, name);
                    g.set_linkage(Linkage::External);
                    g
                };
                let value = self.compile_expr(value)?.into_float_value();
                self.builder.build_store(g.as_pointer_value(), value)?;
                Ok(value.into())
            }
        }
    }

    pub fn compile_expr(&mut self, expr: &ExprAST) -> Result<BasicValueEnum<'ctx>> {
        match expr {
            ExprAST::Number(n) => self.compile_number(*n),
            ExprAST::Variable(name) => self.compile_variable(name),
            ExprAST::Unary { op, operand } => self.compile_unary(op.as_str(), operand),
            ExprAST::Binary { op, lhs, rhs } => self.compile_binary(op.as_str(), lhs, rhs),
            ExprAST::Call { callee, args } => self.compile_call(callee, args),
            ExprAST::If { cond, e_true, e_false } => self.compile_if(cond, e_true, e_false),
            ExprAST::For {
                var,
                e_init,
                e_cond,
                e_step,
                e_body,
            } => self.compile_for(var, e_init, e_cond, e_step, e_body),
            ExprAST::Block(exprs) => self.compile_block(exprs),
            ExprAST::Let(bindings) => self.compile_let(bindings),
            ExprAST::Letin { bindings, body } => self.compile_letin(bindings, body),
            ExprAST::Assign { name, value } => self.compile_assign(name.as_str(), value),
        }
    }
}
