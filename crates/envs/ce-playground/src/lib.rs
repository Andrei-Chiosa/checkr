use ce_core::{Env, Generate, ValidationResult, define_env, rand};
use serde::{Deserialize, Serialize};

define_env!(PlaygroundEnv);

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    simple_string: String
}

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output {
    capitalised_string: String
}

impl Env for PlaygroundEnv {
    type Input = Input;

    type Output = Output;

    type Meta = ();

    fn run(_input: &Self::Input) -> ce_core::Result<Self::Output> {    
        let capitalised = _input
        .simple_string
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");


        Ok(Output { 
            capitalised_string: capitalised
        })
    }

    fn validate(_input: &Self::Input, _output: &Self::Output) -> ce_core::Result<ValidationResult> {
        let reference = Self::run(_input)?;
        
        Ok(
            match (
                &reference.capitalised_string, 
                &_output.capitalised_string
            ) {
                // No error implemented here
                // Both string are equal 
                (r, o) if r == o => ValidationResult::Correct, 
                // Otherwise it is not correct 
                _ => {
                    let info = format!(
                        "Output: result={:?}; Reference: result={:?};",
                        _output.capitalised_string, reference.capitalised_string
                    );
                    ValidationResult::Mismatch { 
                        reason: format!("Did not produce same as reference. {info}"),
                    }
                }
            }
        )
    }
}

enum TestTypes {
    SingleWordLowercase,
    MultiWordLowercase,   
    NonLetterCharacters
}

impl Generate for Input {
    type Context = ();

    fn gn<R: rand::Rng>(_cx: &mut Self::Context, rng: &mut R) -> Self {
        let mut s = String::new();

        match rng.random_range(0..4) {
            // single word lowecase
            0 => {
                s = (0..12).map(|_| (rng.random_range(b'a'..=b'z')) as char).collect();
            }
            // multiple words lower case
            1 => {
                let word_no = rng.random_range(0..=4);
                
                for _ in 0..=word_no {
                    let word_len = rng.random_range(7..=12);
                    let owned_string: String = (0..word_len).map(|_| (rng.random_range(b'a'..=b'z')) as char).collect();
                    s.push_str(&owned_string);
            
                    s.push(' ');
                     
                }

            }
            _ => {
                let len = rng.random_range(1..50);
                s = (0..len).map(|_| (rng.random_range(b'a'..=b'z')) as char).collect();
            }
        }

        Input {
            simple_string: s
        }

    }
}
