use variable::{VarBindingList};
use function::FnDef;
use instruction::InsDef;
use std::collections::HashMap;

/// Logic Engine
/// 管理指令函数和全局变量，执行函数
pub struct Engine {
    /// 指令表
    pub inss: HashMap<String, InsDef>,
    /// 函数表
    pub fns: HashMap<String, FnDef>,
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
    
    #[test]
    fn test_set_var() {
        let engine = {
            let mut fndef = FnDef::new("foo");
            fndef.add_stmt(Stmt::new_set_var("a", "=", "Hello")); // a = "Hello" (str)
            fndef.add_stmt(Stmt::new_set_var("b", "=", "var:a"));
            fndef.add_stmt(Stmt::new_set_var("a", "+=", "World"));
            fndef.add_stmt(Stmt::new_set_var("b", "+=", "var:a"));
            fndef.add_stmt(Stmt::new_set_var("g1", ":=", "100"));
            fndef.add_stmt(Stmt::new_set_var("g2", ":=", "var:b"));
            fndef.add_stmt(Stmt::new_set_var("gi1", ":=", "int:200")); // gi1 = 200 (int)
            fndef.add_stmt(Stmt::new_set_var("gi1", "+=", "int:2")); // gi1 += 2
            fndef.add_stmt(Stmt::new_set_var_ex("gi2", ":=", "var:gi1", "-", "int:2")); // gi2 := gi1 - 2
            let mut engine = Engine::new();
            engine.add_fn(fndef);
            engine
        };
        let mut context = Context::new();
        
        engine.exec_fn("foo", &VarBindingList::new(), &mut context).expect("ok");
        assert_eq!(context.globals.eval_var("g1", None, None), Some("100".to_string()));
        assert_eq!(context.globals.eval_var("g2", None, None), Some("str:HelloHelloWorld".to_string())); // string concat
        assert_eq!(context.globals.eval_var("gi1", None, None), Some("int:202".to_string())); // integer add
        assert_eq!(context.globals.eval_var("gi2", None, None), Some("int:200".to_string()));
    }
    
    #[test]
    fn test_loop() {
        let engine = {
            let mut foo = FnDef::new("foo");
            foo.add_stmt(Stmt::new_set_global("s=str:"));
            foo.add_stmt(Stmt::new_set_global("x=int:0"));
            foo.add_stmt(Stmt::new_set_global("y=int:100"));
            foo.add_stmt(Stmt::new_set_global("z=0"));
            foo.add_stmt(Stmt::new_loop(3));    // loop1  x3
              foo.add_stmt(Stmt::new_set_var("s", "+=", "Hi"));
              foo.add_stmt(Stmt::new_loop(6));     // loop2  x6
                foo.add_stmt(Stmt::new_set_var("x", "+=", "int:1"));
              foo.add_stmt(Stmt::new_end_loop());  // endloop2
              foo.add_stmt(Stmt::new_set_var("y", "-=", "int:5"));
            foo.add_stmt(Stmt::new_end_loop()); // endloop1
            foo.add_stmt(Stmt::new_set_global("z=1"));

            let mut engine = Engine::new();
            engine.add_fn(foo);
            engine
        };

        let mut context = Context::new();
        let args = VarBindingList::new();

        // execute fn foo more than one time (to test cleanup of rtargs of loop statements)
        for i in 0..2 {
            let result = engine.exec_fn("foo", &args, &mut context);
            assert!(result.is_ok());
            assert_eq!(context.globals.eval_var("z", None, None), Some("1".to_string())); // fn foo exits normally
            assert_eq!(context.globals.eval_var("s", None, None), Some("str:HiHiHi".to_string())); // Hi x3
            assert_eq!(context.globals.eval_var("y", None, None), Some("int:85".to_string()));     // 100 - 5x3
            assert_eq!(context.globals.eval_var("x", None, None), Some("int:18".to_string()));     // 0 + 1x6x3
        }
    }
}
