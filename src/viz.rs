use crate::ddsv::{Lts, Process};
use crate::mcctl::Formula;

pub fn process<T>(p: &Process<T>) {
    println!("digraph {{");

    for exec in p.iter() {
        println!("{};", exec.src);
    }
    for exec in p.iter() {
        for trans in exec.transs.iter() {
            println!("{} -> {} [label=\"{}\"];", exec.src, trans.dst, trans.label);
        }
    }

    println!("}}");
}

use bimap::BiMap;
use std::hash::Hash;
pub fn lts<T: Eq + Hash>(
    filename: &str,
    show_vars: fn(&T) -> String,
    lts: &Lts<T>,
    subformula_list: BiMap<usize, Formula>,
) {
    use std::fs;
    use std::io::{BufWriter, Write};
    let mut f = BufWriter::new(fs::File::create(filename).expect("cannot create output file."));

    f.write(b"digraph {{").unwrap();

    // emit states
    for (id, state_ex) in lts.0.iter() {
        f.write_fmt(format_args!("{} [label=\"{}\\n", id, id))
            .unwrap();
        for loc in state_ex.state.locations.iter() {
            f.write_fmt(format_args!("{}", loc)).unwrap();
        }
        f.write_fmt(format_args!("\\n{}", show_vars(&state_ex.state.vars)))
            .unwrap();
        for (i, formula) in subformula_list.iter() {
            if state_ex.is_marked(*i) {
                f.write_fmt(format_args!("\\n{}", formula)).unwrap();
            }
        }
        f.write(b"\",").unwrap();
        if state_ex.is_marked(subformula_list.len() - 1) {
            f.write(b"style=filled,fillcolor=palegreen").unwrap();
        }
        f.write(b"];\n").unwrap();
    }

    // emit transitions
    for (src_id, state_ex) in lts.0.iter() {
        for (label, dst_id) in state_ex.transs.iter() {
            f.write_fmt(format_args!(
                "{} -> {} [label=\"{}\"];\n",
                src_id, dst_id, label
            ))
            .unwrap()
        }
    }
    f.write(b"}}").unwrap();
}
