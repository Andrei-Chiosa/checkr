
#[derive(Debug)]
pub struct RawDfa {
    states: Vec<String>, 
    initial: Option<String>, 
    accepting: Vec<String>, 
    alphabet: Vec<String>, 
    transitions: Vec<(String, String, String)> //from, symbol, to
}

pub fn parse_dfa(input: String) -> RawDfa {    
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

    for line in input.split_ascii_whitespace() {
        match line {
            "#states" => {
                current_section = Section::States;
            }
            "#initial" => {
                current_section = Section::Initial;
            }
            "#accepting" => {
                current_section = Section::Accepting;
            }
            "#alphabet" => {
                current_section = Section::Alphabet;
            }
            "#transitions" => {
                current_section = Section::Transitions;
            }
            _ => {
                let line = line.to_string();
                match current_section {
                    Section::States => dfa.states.push(line),
                    Section::Initial => {
                        dfa.initial = Some(line);
                    }
                    Section::Accepting => dfa.accepting.push(line),
                    Section::Alphabet => dfa.alphabet.push(line),
                    Section::Transitions => {
                        let (lhs, to) = line.split_once('>').expect("Valid transition expected");
                        let (from, symbol) = lhs.split_once(':').expect("Valid transition expected");
                        
                        dfa
                            .transitions
                            .push((from.to_string(), symbol.to_string(), to.to_string()));
                    }
                    _ => panic!("Parsing error")
                    
                } 
            }
        }  
    }

    dfa
}

// fn validate_dfa(input: RawDfa) -> Result<Dfa, InvalidDfa> {
//     todo!()
// }