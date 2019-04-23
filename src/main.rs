#[allow(dead_code)]
#[derive(Debug, Clone)]
enum Value {
    Integer(i32),
    Closure { code: Code, env: Env },
    Block { tag: u8, vec: Vec<Value> },
    Epsilon,
}

use Value::*;

#[derive(Clone)]
struct Code(fn(&mut Vm) -> Code);

impl std::fmt::Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

type Env = Vec<Value>;

type Stack = Vec<Value>;

#[derive(Debug)]
struct Vm {
    arg_stack: Stack,
    ret_stack: Stack,
    local_env: Env,
}

#[allow(dead_code)]
impl Vm {
    fn new() -> Vm {
        Vm {
            arg_stack: vec![],
            ret_stack: vec![],
            local_env: vec![],
        }
    }

    fn call_prim<F>(&mut self, f: F)
    where
        F: Fn(&mut Self),
    {
        f(self)
    }

    fn ldi(&mut self, i: i32) {
        self.arg_stack.push(Integer(i));
    }

    fn access(&mut self, i: usize) {
        self.arg_stack
            .push(self.local_env[self.local_env.len() - i - 1].clone());
    }

    fn closure(&mut self, code: Code) {
        self.arg_stack.push(Closure {
            code,
            env: self.local_env.clone(),
        });
    }

    fn let_(&mut self) {
        if let Some(v) = self.arg_stack.pop() {
            self.local_env.push(v);
        }
    }

    fn endlet(&mut self) {
        self.local_env.pop();
    }

    fn test(&mut self, c1: Code, c2: Code) -> Code {
        if let Some(Integer(1)) = self.arg_stack.pop() {
            c1
        } else {
            c2
        }
    }

    fn add(&mut self) {
        if let (Some(Integer(x)), Some(Integer(y))) = (self.arg_stack.pop(), self.arg_stack.pop()) {
            self.arg_stack.push(Integer(x + y))
        }
    }

    fn eq(&mut self) {
        if let (Some(Integer(x)), Some(Integer(y))) = (self.arg_stack.pop(), self.arg_stack.pop()) {
            self.arg_stack.push(Integer(if x == y { 1 } else { 0 }))
        }
    }

    fn make_block(&mut self, tag: u8, len: usize) {
        let start = self.arg_stack.len() - len;
        let end = self.arg_stack.len();
        let vec = self.arg_stack.drain(start..end).collect();

        self.arg_stack.push(Value::Block { tag, vec });
    }

    fn field(&mut self, i: usize) {
        if let Some(Value::Block { tag: _, vec }) = self.arg_stack.pop() {
            self.arg_stack.push(vec[i].clone())
        }
    }

    fn invoke(&mut self, tag: u8, c1: Code, c2: Code) -> Code {
        if let Some(Value::Block { tag: tag1, vec: _ }) = self.arg_stack.last() {
            if tag == *tag1 {
                c1
            } else {
                c2
            }
        } else {
            panic!("error(invoke)\n");
        }
    }

    fn apply(&mut self, cont: Code) -> Code {
        use Value::*;
        if let Some(Closure { code, env }) = self.arg_stack.pop() {
            if let Some(val) = self.arg_stack.pop() {
                self.ret_stack.push(Closure {
                    code: cont,
                    env: self.local_env.clone(),
                });
                self.local_env = env.clone();
                self.local_env.push(Closure {
                    code: code.clone(),
                    env,
                });
                self.local_env.push(val);
                code
            } else {
                panic!("error(apply 0)\n");
            }
        } else {
            panic!("error(apply 1)\n");
        }
    }

    fn tail_apply(&mut self) -> Code {
        use Value::*;
        if let Some(Closure { code, env }) = self.arg_stack.pop() {
            if let Some(val) = self.arg_stack.pop() {
                self.local_env = env.clone();
                self.local_env.push(Closure {
                    code: code.clone(),
                    env,
                });
                self.local_env.push(val);
                code
            } else {
                panic!("error(tail_apply 0)\n");
            }
        } else {
            panic!("error(tail_apply 1)\n");
        }
    }

    fn push_mark(&mut self) {
        self.arg_stack.push(Value::Epsilon);
    }

    fn grab(&mut self, cont: Code) -> Code {
        use Value::*;
        match self.arg_stack.pop() {
            Some(Epsilon) => {
                if let Some(Closure { code, env }) = self.ret_stack.pop() {
                    self.arg_stack.push(Closure {
                        code: cont,
                        env: self.local_env.clone(),
                    });
                    self.local_env = env;
                    code
                } else {
                    panic!("error(grab 0)\n");
                }
            }
            Some(v) => {
                let clos = Closure {
                    code: cont.clone(),
                    env: self.local_env.clone(),
                };
                self.local_env.push(clos);
                self.local_env.push(v);
                cont
            }
            None => panic!("error(grab 1)\n"),
        }
    }

    fn return_clos(&mut self) -> Code {
        use Value::*;
        let x = self.arg_stack.pop();
        let y = self.arg_stack.pop();
        match (x, y) {
            (Some(v), Some(Epsilon)) => {
                if let Some(Closure { code, env }) = self.ret_stack.pop() {
                    self.local_env = env;
                    self.arg_stack.push(v);
                    code
                } else {
                    panic!("error(return_clos 0)\n");
                }
            }
            (Some(Closure { code, env }), Some(v)) => {
                self.local_env = env.clone();
                self.local_env.push(Closure {
                    code: code.clone(),
                    env,
                });
                self.local_env.push(v);
                code
            }
            (_, _) => panic!("error(return_clos 1)\n"),
        }
    }
}

fn entry(vm: &mut Vm) -> Code {
    vm.closure(Code(|ref mut vm| {
        vm.grab(Code(|ref mut vm| {
            vm.ldi(0);
            vm.access(2);
            vm.eq();
            vm.test(
                Code(|ref mut vm| {
                    vm.access(0);
                    vm.return_clos()
                }),
                Code(|ref mut vm| {
                    vm.access(0);
                    vm.access(2);
                    vm.add();
                    vm.ldi(-1);
                    vm.access(2);
                    vm.add();
                    vm.access(3);
                    vm.tail_apply()
                }),
            )
        }))
    }));
    vm.let_();
    vm.push_mark();
    vm.ldi(0);
    vm.ldi(1000);
    vm.access(0);
    vm.apply(Code(|ref mut vm| {
        vm.endlet();
        println!("{:?}", vm.arg_stack);
        std::process::exit(0)
    }))
}

fn cons_entry(vm: &mut Vm) -> Code {
    vm.ldi(810);
    vm.ldi(42);
    vm.make_block(0, 0);
    vm.make_block(1, 2);
    vm.invoke(
        0,
        Code(|ref mut vm| {
            let v = vm.arg_stack.pop();
            println!("{:?}", v);
            std::process::exit(0)
        }),
        Code(|ref mut vm| {
            vm.invoke(
                1,
                Code(|ref mut vm| {
                    let v = vm.arg_stack.pop();
                    println!("{:?}", v);
                    std::process::exit(0);
                }),
                Code(|ref mut vm| {
                    println!("{:?}", vm.arg_stack);
                    std::process::exit(1);
                }),
            )
        }),
    )
}

fn main() {
    let mut vm = Vm::new();
    let mut cont = Code(cons_entry);

    loop {
        cont = (cont.0)(&mut vm)
    }
}
