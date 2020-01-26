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
    show_vars: fn(&T) -> String,
    lts: &Lts<T>,
    subformula_list: BiMap<usize, Formula>,
) {
    println!("digraph {{");

    // emit states
    for (id, state_ex) in lts.iter() {
        // for (id, (state, transs, marked)) in lts.iter() {
        print!("{} [label=\"{}\\n", id, id);
        for loc in state_ex.state.locations.iter() {
            print!("{} ", loc);
        }
        print!("\\n{}", show_vars(&state_ex.state.vars));
        for (i, formula) in subformula_list.iter() {
            if state_ex.is_marked(*i) {
                print!("\\n{}", formula)
            }
        }
        print!("\",");
        if state_ex.is_marked(subformula_list.len() - 1) {
            print!("style=filled,fillcolor=palegreen");
        }
        println!("];");
    }

    // emit transitions
    for (src_id, state_ex) in lts.iter() {
        for (label, dst_id) in state_ex.transs.iter() {
            println!("{} -> {} [label=\"{}\"];", src_id, dst_id, label)
        }
    }

    println!("}}");
}
