
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
    
    let result = program.run(&consts, &[]);
    
    eprintln!("Result: {result}");
}
