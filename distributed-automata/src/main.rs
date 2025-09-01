use std::collections::{HashMap, HashSet, BTreeMap};
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};

#[derive(Debug)]
struct Event {
    node_id: u32,
    transition: String,
    state: String,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct LabeledTransition {
    from: String,
    input: String,
    to: String,
    is_transformed: bool,
}

fn main() -> io::Result<()> {
    let mut transform_map: HashMap<String, String> = HashMap::new();
    {
        let transform_file = File::open("data/transform.txt")?;
        let transform_reader = io::BufReader::new(transform_file);

        for line in transform_reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.len() != 2 {
                eprintln!("Línea inválida en tranform.txt: {}", line);
                continue;
            }
            transform_map.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    // Leer path al archivo principal
    let mut input = String::new();
    println!("Ingrese el path al log:");
    io::stdin().read_line(&mut input)?;
    let path = input.trim();

    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    let mut events: Vec<Event> = vec![];

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.trim().split_whitespace().collect();

        if parts.len() != 3 {
            eprintln!("Línea inválida: {}", line);
            continue;
        }

        let node_id: u32 = match parts[0].parse() {
            Ok(n) => n,
            Err(_) => {
                eprintln!("ID inválido: {}", parts[0]);
                continue;
            }
        };

        let mut transition = parts[1].to_string();
        let state = parts[2];

        if let Some(new_name) = transform_map.get(&transition) {
            transition = new_name.clone();
        }

        events.push(Event {
            node_id,
            transition,
            state: state.to_string(),
        });
    }

    let mut states_per_node: HashMap<u32, HashSet<String>> = HashMap::new();
    let mut transitions_per_node: HashMap<u32, HashSet<LabeledTransition>> = HashMap::new();

    for pair in events.windows(2) {
        let curr = &pair[0];
        let next = &pair[1];

        if curr.node_id == next.node_id {
            let (transformed, is_transformed) = match transform_map.get(&curr.transition) {
                Some(new) => (new.clone(), true),
                None => (curr.transition.clone(), false),
            };

            states_per_node.entry(curr.node_id).or_default().insert(curr.state.clone());
            states_per_node.entry(curr.node_id).or_default().insert(next.state.clone());

            transitions_per_node
                .entry(curr.node_id)
                .or_default()
                .insert(LabeledTransition {
                    from: curr.state.clone(),
                    input: transformed,
                    to: next.state.clone(),
                    is_transformed,
                });
        }
    }

    for node_id in transitions_per_node.keys().collect::<Vec<_>>() {
        println!("\nNodo {}:", node_id);

        if let Some(states) = states_per_node.get(node_id) {
            println!("  Estados:");
            for s in states {
                println!("   - {}", s);
            }
        }

        if let Some(transitions) = transitions_per_node.get(node_id) {
            println!("  Transiciones:");
            for t in transitions {
                println!("   - {} --{}--> {}", t.from, t.input, t.to);
            }

            // Agrupar transiciones por (from, to)
            let mut grouped: BTreeMap<(String, String), Vec<(String, bool)>> = BTreeMap::new();
            for t in transitions {
                grouped
                    .entry((t.from.clone(), t.to.clone()))
                    .or_default()
                    .push((t.input.clone(), t.is_transformed));
            }

            // Generar archivo .dot
            let filename = format!("nodo_{}.dot", node_id);
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&filename)?;

            writeln!(file, "digraph G {{")?;
            writeln!(file, "    rankdir=LR;")?;
            writeln!(file, "    node [shape=circle, fontsize=10, width=0.5];")?;
            writeln!(file, "    edge [fontsize=9];")?;

            for ((from, to), labels) in grouped {
                let label_str = labels
                    .iter()
                    .map(|(label, is_transformed)| {
                        if *is_transformed {
                            format!(r#"<FONT COLOR="blue">{}</FONT>"#, label)
                        } else {
                            label.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("<BR/>");

                writeln!(
                    file,
                    r#"    "{}" -> "{}" [label=<{}>];"#,
                    from, to, label_str
                )?;
            }

            writeln!(file, "}}")?;

            println!("  → Archivo .dot generado: {}", filename);
        }
    }

    Ok(())
}
