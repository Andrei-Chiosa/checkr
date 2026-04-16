use ce_core::rand;
use ce_core::rand::Rng;
use ce_minimizer::*;

fn main() {
    let mut rng = rand::rng();
    let state_count = if rng.random_bool(0.6) {rng.random_range(2..=4)} else { rng.random_range(5..=8)};
    //let allow_nondeterminism = rng.random_bool(0.1);
    let allow_nondeterminism = false;

    let mut entries: Vec<(String, String, usize, usize)> = Vec::new();

    for i in 0..100 {
        let dfa_input = generate_random_dfa(&mut rng, state_count, allow_nondeterminism);
        let raw_dfa = parse_dfa(&dfa_input).expect("correct dfa expected");
        let mut named_dfa = NamedDFA::build(raw_dfa).expect("correct dfa expected");

        let dot_before = named_dfa.to_dot();
        let states_before = named_dfa.dfa.state_count;
        
        let minimized = named_dfa.minimize().expect("no minimization error expected");
        let dot_after = minimized.to_dot();
        let states_after = minimized.dfa.state_count;

        if i == 0 {
            println!("{}", dot_before);
            println!("{}", dot_after);
        }

        entries.push((dot_before, dot_after, states_before, states_after));
    }

    let html = generate_report(&entries);
    std::fs::write("report.html", html).expect("could not write report");
}

fn generate_report(entries: &[(String, String, usize, usize)]) -> String {
    let cards: String = entries.iter().enumerate().map(|(i, (_, _, s_before, s_after))| {
        format!(r#"
        <div class="card">
            <h2>Generation {i}</h2>
            <p class="stats">{s_before} states → {s_after} states</p>
            <div class="graphs">
                <div>
                    <h3>Before</h3>
                    <div class="graph" data-index="{i}" data-side="before"></div>
                </div>
                <div>
                    <h3>After</h3>
                    <div class="graph" data-index="{i}" data-side="after"></div>
                </div>
            </div>
        </div>
        "#)
    }).collect();

    let dots: String = entries.iter().enumerate().map(|(i, (before, after, _, _))| {
        format!("dots[{i}] = {{ before: `{}`, after: `{}` }};\n",
            before.replace('\\', "\\\\").replace('`', "\\`"),
            after.replace('\\', "\\\\").replace('`', "\\`"))
    }).collect();

    format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>DFA Minimization Report</title>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/viz.js/2.1.2/viz.js"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/viz.js/2.1.2/full.render.js"></script>
    <style>
        body {{ font-family: sans-serif; padding: 2rem; background: #f5f5f5; }}
        h1 {{ margin-bottom: 2rem; }}
        .card {{ background: white; border-radius: 8px; padding: 1.5rem; margin-bottom: 2rem; box-shadow: 0 2px 6px rgba(0,0,0,0.1); }}
        .stats {{ color: #666; font-size: 0.95rem; margin-bottom: 1rem; }}
        .graphs {{ display: flex; gap: 2rem; }}
        .graphs > div {{ flex: 1; }}
        .graph svg {{ width: 100%; height: auto; }}
        h2 {{ margin-top: 0; }}
        h3 {{ color: #444; }}
    </style>
</head>
<body>
    <h1>DFA Minimization Report</h1>
    {cards}
    <script>
        const dots = [];
        {dots}
        const viz = new Viz();
        document.querySelectorAll('.graph').forEach(el => {{
            const i = el.dataset.index;
            const side = el.dataset.side;
            const dot = dots[i][side];
            viz.renderSVGElement(dot)
               .then(svg => el.appendChild(svg))
               .catch(err => {{ el.innerText = 'Render error: ' + err; console.error(dot); }});
        }});
    </script>
</body>
</html>"#)
}
