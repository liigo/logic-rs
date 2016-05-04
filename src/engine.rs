use variable::{VarBindingList};
use function::FnDef;
use instruction::InstDef;
use std::collections::HashMap;

// Logic Engine
// 管理指令函数和全局变量，执行函数
pub struct Engine {
    pub insts: HashMap<String, InstDef>, // 指令表
    pub fns: HashMap<String, FnDef>,     // 函数表
    pub globals: VarBindingList,         // 全局变量表
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            insts: HashMap::new(),
            fns: HashMap::new(),
            globals: VarBindingList::new(),
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

    pub fn exec_fn(&mut self, name: &str, args: &VarBindingList) -> Result<String,String> {
        if let Some(fndef) = self.find_fn(name) {
            fndef.exec(self, args)
        } else {
            Err(format!("No such fn: {}", name))
        }
    }
}
