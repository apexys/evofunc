use evofunc::{Instruction, Program};


pub fn main(){
    let iset =  vec![
        Instruction::Const(0),
        Instruction::Const(1),
        Instruction::Add
    ];
    
    let consts = vec![1.0, 2.0];
    
    let program = Program::create(&[
    Instruction::Const(0),
    Instruction::Const(1),
    Instruction::Add
    ]);
    
    let result = program.evaluate_to_result(&consts, &[]);
    
    eprintln!("Result: {result:?}");
}
