use variable::{VarBindingList};
use function::FnDef;
use instruction::InstDef;
use std::collections::HashMap;

// Logic Engine
// 管理指令函数和全局变量，执行函数
pub struct Engine {
    pub insts: HashMap<String, InstDef>, // 指令表
    pub fns: HashMap<String, FnDef>,     // 函数表
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            insts: HashMap::new(),
            fns: HashMap::new(),
        }
    }

    pub fn find_inst(&self, name: &str) -> Option<&InstDef> {
        self.insts.get(name)
    }

    pub fn find_fn(&self, name: &str) -> Option<&FnDef> {
        self.fns.get(name)
    }

    pub fn add_inst(&mut self, def: InstDef) {
        self.insts.insert(def.name.clone(), def);
    }

    pub fn add_fn(&mut self, def: FnDef) {
        self.fns.insert(def.name.clone(), def);
    }

    pub fn exec_fn(&self, name: &str, args: &VarBindingList, context: &mut Context) -> Result<String,String> {
        if let Some(fndef) = self.find_fn(name) {
            fndef.exec(args, context /* &mut Context */, self /* &Engine */)
        } else {
            Err(format!("No such fn: {}", name))
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
            let mut func = FnDef::new("foo");
            func.add_stmt(Stmt::new_return("0"));
            let mut engine = Engine::new();
            engine.add_fn(func);
            engine
        };
        let mut context = Context::new();
        let args = VarBindingList::new();
        let result = engine.exec_fn("foo", &args, &mut context);
        assert!(result.is_ok());
    }
}
