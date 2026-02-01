mod dfa;

use std::fmt::format;

use ce_core::{Env, Generate, ValidationResult, define_env, rand};
use serde::{Deserialize, Serialize};

use dfa::{
    RawDfa
};

define_env!(MinimizerEnv);

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    raw_input: String 
}

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output {
    test_output: String
}

impl Env for MinimizerEnv {
    type Input = Input;

    type Output = Output;

    type Meta = ();

    fn run(input: &Self::Input) -> ce_core::Result<Self::Output> {
        let dfa = dfa::parse_dfa(input.raw_input.as_str());
        
        Ok( Output { test_output: format!("{:?}", dfa)})
    }

    fn validate(_input: &Self::Input, _output: &Self::Output) -> ce_core::Result<ValidationResult> {
        Ok(ValidationResult::Correct)
    }
}

impl Generate for Input {
    type Context = ();

    fn gn<R: rand::Rng>(_cx: &mut Self::Context, _rng: &mut R) -> Self {
        Self::default()
    }
}
