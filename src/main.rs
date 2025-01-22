use instructions::{Instruction, Program};
use nodes::MCTS;

mod arena;
mod instructions;
mod nodes;

fn main() {
    let iset = vec![
        Instruction::Const(0),
        Instruction::Const(1),
        Instruction::Const(2),
        Instruction::Add,
        Instruction::Sub,
        Instruction::Mul,
        Instruction::Div,
        Instruction::Exp,
        Instruction::Log,
    ];

    const CONSTS: [f32; 3] = [0.0, 1.0, 2.0];

    let distance_to_pi = move |prog: &Program| {
        prog.run(&CONSTS, &[])
            .map(|r| -(r - std::f32::consts::PI).abs())
            .unwrap_or(-9999999.0)
    };

    let mut mcts = MCTS::with_max_program_length(&iset, 32, distance_to_pi);
    mcts.exploration_chance = 1.0 / (2f32.sqrt());
    let mut current_high_score = mcts.high_score();
    eprintln!("start score {}", current_high_score);

    for i in 0..20_000_000 {
        let more_exploration = mcts.search_one();
        let high_score = mcts.high_score();
        if high_score > current_high_score {
            current_high_score = high_score;
            let prog = mcts.make_best_program();
            let result = prog.run(&CONSTS, &[]);
            let unformatted = mcts.make_best_program().render();
            let prog = mcts.make_best_program().render_pretty(&CONSTS)/*.map(|p| {
                let pycmd = format!("from sympy.parsing.sympy_parser import parse_expr; print(parse_expr(\"{p}\"))");
                let out = std::process::Command::new("python").arg("-c").arg(pycmd).output();
                match out {
                    Err(e) => {
                        format!("Unoptimized: {}", p)
                    }
                    Ok(out)=> {
                        String::from_utf8_lossy(&out.stdout).trim().to_string()
                    },
                }                
            })*/;
            eprintln!(
                "\nNew highscore {high_score} with program {} [{:?}] => {:?}",
                prog.unwrap_or_default(),
                unformatted,
                result
            );
        }
        if !more_exploration {
            break;
        }
    }
    eprintln!(
        "\nBest program has highscore {current_high_score} with program {:?}",
        mcts.make_best_program().render()
    );

    //std::fs::write("mcts.dot", mcts.write_dot());
}
