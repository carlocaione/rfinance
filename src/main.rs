use anyhow::Result;
use clap_repl::{
    reedline::{DefaultPrompt, DefaultPromptSegment},
    ClapEditor,
};
use rfinance::{
    cmd::{Cmd, Command},
    data::Data,
    finance::FinanceProvider,
};

fn main() -> Result<()> {
    let mut data = Data::load()?;
    let mut finance = FinanceProvider::new(&data.api_key);
    let mut cmd = Cmd::new(&mut data, &mut finance);

    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("rfinance".to_owned()),
        right_prompt: DefaultPromptSegment::Empty,
    };

    let rl = ClapEditor::<Command>::builder()
        .with_prompt(Box::new(prompt))
        .build();

    rl.repl(|command| {
        if let Err(err) = cmd.parse(command) {
            eprint!("ERROR: {}", err);
        }
    });

    Ok(())
}
