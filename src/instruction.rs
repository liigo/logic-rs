use variable::{VarDef, VarDefList, VarBinding, VarBindingList};
use std::slice;

// 指令的定义和实现
pub struct InstDef {
    pub name: String,
    pub canid: u32,
    pub args: VarDefList,
    pub note: String,
}

impl InstDef {
    pub fn new(name: &str, canid: u32) -> InstDef {
        InstDef {
            name: name.to_string(),
            canid: canid,
            args: VarDefList::new(),
            note: "".to_string(),
        }
    }

    pub fn exec(&self, args: &VarBindingList, data: &mut Vec<u8>) {
        for vardef in &self.args.defs {
            let value: &str = {
                // TODO: use eval_expr()
                match args.raw_value_of(vardef.name.as_str()) {
                    Some(value) => value,
                    None => {
                        if vardef.default == "" {
                            panic!("Require arg: {}", vardef.name);
                        }
                        vardef.default.as_str()
                    }
                }
            };
            match vardef.typ.as_str() {
                "byte" | "i8" | "u8" => {
                    data.push(value.parse().expect("invalid byte/i8/u8"));
                }
                "i16" | "u16" => {
                    let v: u16 = value.parse().expect("invalid i16/u16");
                    let v = v.to_be(); // to big endian
                    assert!(data.len() <= 6);
                    data.extend_from_slice(unsafe { slice::from_raw_parts(&v as *const u16 as *const u8, 2) });
                }
                "i32" | "u32" => {
                    let v: u32 = value.parse().expect("invalid i32/u32");
                    let v = v.to_be(); // to big endian
                    assert!(data.len() <= 4);
                    data.extend_from_slice(unsafe { slice::from_raw_parts(&v as *const u32 as *const u8, 4) });
                }
                _ => {
                    panic!("Unsupport arg type: {}", vardef.typ);
                }
            }
        }
        assert!(data.len() == 8); // we need 8 bytes data here
        println!("instruction data: {:?}", data);
    }
}

#[cfg(test)]
mod tests {
    use super::InstDef;
    use variable::{VarDef, VarDefList, VarBinding, VarBindingList};

    #[test]
    fn test_instdef() {
        let mut movr = InstDef::new("move radar", 1000);
        assert!(movr.name == "move radar");
        assert!(movr.canid == 1000);
        movr.args.add(VarDef::new("a", "i32"));
        movr.args.add(VarDef::new("b", "u8"));
        movr.args.add(VarDef::new("c", "u16"));
        movr.args.add(VarDef::new("d", "u8"));

        let mut args = VarBindingList::new();
        args.set_binding("a", "1357900");
        args.set_binding("b", "255");
        args.set_binding("c", "25135");
        args.set_binding("d", "0");

        let mut data = vec![];
        movr.exec(&args, &mut data);
        assert_eq!(data, vec![0x00,0x14,0xb8,0x4c, 0xff, 0x62,0x2f, 0x0]);
    }
}
