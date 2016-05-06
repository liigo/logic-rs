use variable::{VarBindingList};
use function::FnDef;
use instruction::InsDef;
use std::collections::HashMap;

// Logic Engine
// 管理指令函数和全局变量，执行函数
pub struct Engine {
    pub inss: HashMap<String, InsDef>, // 指令表
    pub fns: HashMap<String, FnDef>,     // 函数表
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            inss: HashMap::new(),
            fns: HashMap::new(),
        }
    }

    pub fn find_ins(&self, name: &str) -> Option<&InsDef> {
        self.inss.get(name)
    }

    pub fn find_fn(&self, name: &str) -> Option<&FnDef> {
        self.fns.get(name)
    }

    pub fn add_ins(&mut self, def: InsDef) {
        self.inss.insert(def.name.clone(), def);
    }

    pub fn add_fn(&mut self, def: FnDef) {
        self.fns.insert(def.name.clone(), def);
    }

    pub fn exec_fn(&self, name: &str, args: &VarBindingList, context: &mut Context) -> Result<(),String> {
        if let Some(fndef) = self.find_fn(name) {
            fndef.exec(args, context /* &mut Context */, self /* &Engine */)
        } else {
            let err = format!("No such fn: {}", name);
            context.log_error(&err);
            Err(err)
        }
    }

    pub fn exec_ins(&self, name: &str, args: &VarBindingList, context: &mut Context) -> Result<(),String> {
        if let Some(insdef) = self.find_ins(name) {
            let mut data = Vec::new();
            insdef.exec(args, &mut data, context);
            Ok(())
        } else {
            let err = format!("No such fn: {}", name);
            context.log_error(&err);
            Err(err)
        }
    }
}

// 引擎执行的上下文对象
// 被 Engine::exec_fn() 和 FnDef::exec() 使用
pub struct Context {
    pub globals: VarBindingList, // 全局变量表
    // TODO: logs, errors
}

impl Context {
    pub fn new() -> Context {
        Context {
            globals: VarBindingList::new(),
        }
    }

    pub fn log_info(&self, text: &str) {
        println!("[info] {}", text);
    }

    pub fn log_warning(&self, text: &str) {
        println!("[Warning] {}", text);
    }

    pub fn log_error(&self, text: &str) {
        println!("[ERROR] {}", text);
    }

    pub fn log_fatal(&self, text: &str) {
        println!("[!!FATAL!!] {}", text);
    }
}

#[cfg(test)]
mod tests {
    use engine::{Engine, Context};
    use function::{FnDef};
    use statement::{Stmt};
    use variable::{VarBindingList};

    #[test]
    fn test_exec_fn() {
        let engine = {
            let mut fn1 = FnDef::new("fn1");
            fn1.add_stmt(Stmt::new_call_fn("fn2", VarBindingList::new()));
            fn1.add_stmt(Stmt::new_set_global("a=1"));
            fn1.add_stmt(Stmt::new_return("123")); // returns here
            fn1.add_stmt(Stmt::new_set_global("a=2")); // never execute this statement

            let mut fn2 = FnDef::new("fn2");
            fn2.add_stmt(Stmt::new_set_global("a=3"));

            let mut engine = Engine::new();
            engine.add_fn(fn1);
            engine.add_fn(fn2);
            engine
        };

        let mut context = Context::new();
        let args = VarBindingList::new();
        let result = engine.exec_fn("fn2", &args, &mut context);
        assert!(result.is_ok());
        assert_eq!(context.globals.raw_value_of("a"), Some("3")); // ensures fn2 was executed correctly
        assert_eq!(context.globals.raw_value_of("$return"), None); // fn2 returns nothing

        let result = engine.exec_fn("fn1", &args, &mut context);
        assert!(result.is_ok());
        // ensures fn1 was executed, and it returned before statement "a=2"
        assert_eq!(context.globals.raw_value_of("a"), Some("1"));
        assert_eq!(context.globals.raw_value_of("$return"), Some("123")); // fn1 returns "123"
    }
}
