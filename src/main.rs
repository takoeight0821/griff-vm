mod vm;
use vm::*;

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
        Code(cons_entry)
    }))
}

fn cons_entry(vm: &mut Vm) -> Code {
    vm.make_block(0, 0);
    vm.make_block(1, 2);
    vm.invoke(
        0,
        Code(|ref mut vm| {
            println!("{:?}", vm.arg_stack);
            std::process::exit(0)
        }),
        Code(|ref mut vm| {
            vm.invoke(
                1,
                Code(|ref mut vm| {
                    println!("{:?}", vm.arg_stack);
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
    let mut cont = Code(entry);

    loop {
        cont = (cont.0)(&mut vm)
    }
}
