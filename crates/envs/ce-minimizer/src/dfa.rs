
#[derive(Debug)]
pub struct RawDfa {
    states: Vec<String>, 
    initial: Option<String>, 
    accepting: Vec<String>, 
    alphabet: Vec<String>, 
    transitions: Vec<(String, String, String)> //from, symbol, to
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken { line: usize, token: String },
    InvalidTransition { line: usize, text: String },
    MultipleInitial { line: usize },
    TokenOutsideSection { line: usize, token: String },
}

pub fn parse_dfa(input: &str) -> Result<RawDfa,ParseError> {    
    #[derive(PartialEq)]
    enum Section {
        Null,
        States, 
        Initial, 
        Accepting, 
        Alphabet, 
        Transitions
    }
    
    let mut dfa= RawDfa {
        states: Vec::new(),
        initial: None, 
        accepting: Vec::new(), 
        alphabet: Vec::new(), 
        transitions: Vec::new()
    };
    let mut current_section = Section::Null;

    for (line_no, raw_line) in input.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        match line {
            "#states" => current_section = Section::States,
            "#initial" => current_section = Section::Initial,
            "#accepting" => current_section = Section::Accepting,
            "#alphabet" => current_section = Section::Alphabet,
            "#transitions" => current_section = Section::Transitions,
            _ => {
                let line = line.to_string();
                match current_section {
                    Section::States => dfa.states.push(line),
                    Section::Initial => {
                        if dfa.initial.is_some() {
                            return Err({ParseError::MultipleInitial { line: line_no + 1 }})
                        }
                        dfa.initial = Some(line.to_string())
                    }
                    Section::Accepting => dfa.accepting.push(line),
                    Section::Alphabet => dfa.alphabet.push(line),
                    Section::Transitions => {
                        let (lhs, to) = line
                            .split_once('>')
                            .ok_or(ParseError::InvalidTransition { 
                                line: line_no + 1, 
                                text: line.to_string(), 
                            })?;
                        let (from, symbol) = lhs
                            .split_once(':')
                            .ok_or(ParseError::InvalidTransition { 
                                line: line_no + 1, 
                                text: line.to_string(), 
                            })?;
                        dfa
                            .transitions
                            .push((from.to_string(), symbol.to_string(), to.to_string()));
                    }
                    Section::Null => {
                        return Err(ParseError::TokenOutsideSection { 
                            line: line_no + 1, 
                            token: line.to_string(), 
                        })
                    }
                    
                } 
            }
        }  
    }

    Ok(dfa)
}

// fn validate_dfa(input: RawDfa) -> Result<Dfa, String> {
//     todo!()
// }