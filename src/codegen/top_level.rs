use std::fs;

use chumsky::Parser;
use inkwell::module::Linkage;

use crate::Result;
use crate::codegen::{Codegen, ImportMarker, STDLIB_SRC};
use crate::frontend::ast::{Binding, ExprAST, FunctionAST, FunctionName, PrototypeAST, TopLevel};
use crate::frontend::parser::create_program_parser;

impl<'ctx> Codegen<'ctx> {
    fn compile_top_def(&mut self, f: &FunctionAST) -> Result<Option<String>> {
        let f = self.compile_function(f)?;
        match f.get_name().to_str() {
            Ok(fn_name) => Ok(Some(fn_name.to_string())),
            Err(_) => Ok(None),
        }
    }

    fn compile_top_extern(&mut self, p: &PrototypeAST) -> Result<Option<String>> {
        let f = self.compile_extern(p)?;
        match f.get_name().to_str() {
            Ok(fn_name) => Ok(Some(fn_name.to_string())),
            Err(_) => Ok(None),
        }
    }

    fn compile_top_let(&mut self, bindings: &[Binding]) -> Result<Option<String>> {
        for binding in bindings {
            let global = self.module.add_global(self.context.f64_type(), None, &binding.name);
            global.set_initializer(&self.context.f64_type().const_float(0.0));
            global.set_linkage(Linkage::External);
            self.globals.insert(binding.name.clone());
        }

        let func_name = format!("__anon_expr_{}", self.anon_counter);
        self.anon_counter += 1;

        let init_fn = self
            .module
            .add_function(&func_name, self.context.f64_type().fn_type(&[], false), None);
        let entry = self.context.append_basic_block(init_fn, "entry");
        self.builder.position_at_end(entry);

        let mut last_val = self.context.f64_type().const_float(f64::NAN);
        for binding in bindings {
            let init_val = match &binding.init {
                Some(e) => self.compile_expr(e)?.into_float_value(),
                None => self.context.f64_type().const_float(f64::NAN),
            };

            let global = self.module.get_global(&binding.name).unwrap();
            self.builder.build_store(global.as_pointer_value(), init_val)?;
            last_val = init_val;
        }

        self.builder.build_return(Some(&last_val))?;

        if !init_fn.verify(true) {
            unsafe {
                init_fn.delete();
            }
            return Err(format!("invalid let in {}", func_name).into());
        }

        Ok(Some(func_name))
    }

    fn compile_top_expr(&mut self, e: &ExprAST) -> Result<Option<String>> {
        let name = format!("__anon_expr_{}", self.anon_counter);
        let func = FunctionAST {
            proto: PrototypeAST {
                name: FunctionName::Ident(name.clone()),
                args: vec![],
            },
            body: e.clone(),
        };
        self.anon_counter += 1;
        self.compile_function(&func)?;
        Ok(Some(name))
    }

    fn compile_top_once(&mut self) -> Result<Option<String>> {
        Err("internal error: once should have been consumed by import".into())
    }

    fn compile_top_import(&mut self, path: &str) -> Result<Option<String>> {
        let (src, fallback) = if path == "std" {
            (STDLIB_SRC.to_string(), ImportMarker::Named("std".into()))
        } else {
            let canonical = fs::canonicalize(path).map_err(|e| format!("cannot resolve {}: {}", path, e))?;
            let content = fs::read_to_string(&canonical).map_err(|e| format!("cannot read {}: {}", path, e))?;
            (content, ImportMarker::Path(canonical))
        };

        let program = create_program_parser()
            .parse(src.as_str())
            .into_result()
            .map_err(|errs| errs.iter().map(|e| e.to_string()).collect::<Vec<String>>().join("\n"))?;

        let mut once: Option<Option<String>> = None;
        let mut real_items = Vec::with_capacity(program.items.len());

        for item in program.items {
            match item {
                TopLevel::Once(name) => {
                    if once.is_some() {
                        return Err(format!("duplicate once in {}", path).into());
                    }
                    once = Some(name);
                }
                other => real_items.push(other),
            }
        }

        let marker = match once {
            None => None,
            Some(None) => Some(fallback),
            Some(Some(name)) => Some(ImportMarker::Named(name)),
        };

        if let Some(m) = &marker {
            if !self.imported.insert(m.clone()) {
                return Err(format!(
                    "module {} duplicate imported",
                    (match m {
                        ImportMarker::Named(name) => name.as_str(),
                        ImportMarker::Path(path) => path.to_str().unwrap_or("<???>"),
                    })
                )
                .into());
            }
        }

        for item in &real_items {
            self.compile_top_level(item)?;
        }

        Ok(None)
    }

    pub(crate) fn compile_top_level(&mut self, item: &TopLevel) -> Result<Option<String>> {
        match item {
            TopLevel::Def(f) => self.compile_top_def(f),
            TopLevel::Extern(p) => self.compile_top_extern(p),
            TopLevel::Expr(ExprAST::Let(bindings)) => self.compile_top_let(bindings),
            TopLevel::Expr(e) => self.compile_top_expr(e),
            TopLevel::Once(_) => self.compile_top_once(),
            TopLevel::Import(path) => self.compile_top_import(path),
        }
    }
}
