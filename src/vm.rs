use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i32),
    Closure { code: Code, env: Env },
    Block { tag: u8, vec: Vec<Value> },
    Epsilon,
}

use Value::*;

#[derive(Clone)]
pub struct Code(pub fn(&mut Vm) -> Code);

impl std::fmt::Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

type Env = Rc<Vec<Value>>;

type Stack = Vec<Value>;

#[derive(Debug)]
pub struct Vm {
    pub arg_stack: Stack,
    pub ret_stack: Stack,
    pub local_env: Env,
}

#[allow(dead_code)]
impl Vm {
    pub fn new() -> Vm {
        Vm {
            arg_stack: vec![],
            ret_stack: vec![],
            local_env: Rc::new(vec![]),
        }
    }

    pub fn pop_arg(&mut self) -> Value {
        if let Some(v) = self.arg_stack.pop() {
            v
        } else {
            unreachable!()
        }
    }

    pub fn push_local(&mut self, v: Value) {
        Rc::make_mut(&mut self.local_env).push(v)
    }

    pub fn call_prim<F>(&mut self, f: F)
    where
        F: Fn(&mut Self),
    {
        f(self)
    }

    pub fn ldi(&mut self, i: i32) {
        self.arg_stack.push(Integer(i));
    }

    pub fn access(&mut self, i: usize) {
        self.arg_stack
            .push(self.local_env[self.local_env.len() - i - 1].clone());
    }

    pub fn closure(&mut self, code: Code) {
        self.arg_stack.push(Closure {
            code,
            env: self.local_env.clone(),
        });
    }

    pub fn let_(&mut self) {
        let v = self.pop_arg();
        self.push_local(v);
    }

    pub fn endlet(&mut self) {
        Rc::make_mut(&mut self.local_env).pop();
    }

    pub fn test(&mut self, c1: Code, c2: Code) -> Code {
        if let Integer(1) = self.pop_arg() {
            c1
        } else {
            c2
        }
    }

    pub fn add(&mut self) {
        if let (Integer(x), Integer(y)) = (self.pop_arg(), self.pop_arg()) {
            self.arg_stack.push(Integer(x + y))
        }
    }

    pub fn eq(&mut self) {
        if let (Integer(x), Integer(y)) = (self.pop_arg(), self.pop_arg()) {
            self.arg_stack.push(Integer(if x == y { 1 } else { 0 }))
        }
    }

    pub fn make_block(&mut self, tag: u8, len: usize) {
        let start = self.arg_stack.len() - len;
        let end = self.arg_stack.len();
        let vec = self.arg_stack.drain(start..end).collect();

        self.arg_stack.push(Value::Block { tag, vec });
    }

    pub fn field(&mut self, i: usize) {
        if let Value::Block { tag: _, vec } = self.pop_arg() {
            self.arg_stack.push(vec[i].clone())
        }
    }

    pub fn invoke(&mut self, tag: u8, c1: Code, c2: Code) -> Code {
        if let Some(Value::Block { tag: tag1, vec: _ }) = self.arg_stack.last() {
            if tag == *tag1 {
                c1
            } else {
                c2
            }
        } else {
            unreachable!("invoke")
        }
    }

    pub fn apply(&mut self, cont: Code) -> Code {
        use Value::*;
        let clos = self.pop_arg();
        if let Closure { code, env } = clos.clone() {
            let val = self.pop_arg();
            self.ret_stack.push(Closure {
                code: cont,
                env: self.local_env.clone(),
            });
            self.local_env = env;
            self.push_local(clos);
            self.push_local(val);
            code
        } else {
            unreachable!("apply")
        }
    }

    pub fn tail_apply(&mut self) -> Code {
        use Value::*;
        let clos = self.pop_arg();
        if let Closure { code, env } = clos.clone() {
            let val = self.pop_arg();
            self.local_env = env;
            self.push_local(clos);
            self.push_local(val);
            code
        } else {
            unreachable!("tail_apply");
        }
    }

    pub fn push_mark(&mut self) {
        self.arg_stack.push(Value::Epsilon);
    }

    pub fn grab(&mut self, cont: Code) -> Code {
        use Value::*;
        match self.pop_arg() {
            Epsilon => {
                if let Some(Closure { code, env }) = self.ret_stack.pop() {
                    self.arg_stack.push(Closure {
                        code: cont,
                        env: self.local_env.clone(),
                    });
                    self.local_env = env;
                    code
                } else {
                    unreachable!("grab");
                }
            }
            v => {
                let clos = Closure {
                    code: cont.clone(),
                    env: self.local_env.clone(),
                };
                self.push_local(clos);
                self.push_local(v);
                cont
            }
        }
    }

    pub fn return_clos(&mut self) -> Code {
        use Value::*;
        let x = self.pop_arg();
        let y = self.pop_arg();
        match (x, y) {
            (v, Epsilon) => {
                if let Some(Closure { code, env }) = self.ret_stack.pop() {
                    self.local_env = env;
                    self.arg_stack.push(v);
                    code
                } else {
                    unreachable!("return_clos")
                }
            }
            (Closure { code, env }, v) => {
                self.local_env = env.clone();
                self.push_local(Closure {
                    code: code.clone(),
                    env,
                });
                self.push_local(v);
                code
            }
            (_, _) => unreachable!("return_clos"),
        }
    }
}
