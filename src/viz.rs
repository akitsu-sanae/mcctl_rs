use crate::{lts::Lts, mark::Marks};

use std::fmt::Display;
use std::hash::Hash;
pub fn lts<T: Eq + Hash + Display>(filename: &str, lts: &Lts<T>, marks: Marks) {
    use std::fs;
    use std::io::{BufWriter, Write};
    let mut f = BufWriter::new(fs::File::create(filename).expect("cannot create output file."));

    f.write(b"digraph {{").unwrap();

    // emit states
    for (state_id, trans) in lts.0.iter().enumerate() {
        f.write_fmt(format_args!("{} [label=\"{}\\n", state_id, state_id))
            .unwrap();
        for loc in trans.state.locations.iter() {
            f.write_fmt(format_args!("{}", loc)).unwrap();
        }
        f.write_fmt(format_args!("\\n{}", &trans.state.vars))
            .unwrap();
        for (i, formula) in marks.subformulas.iter() {
            if marks.marks.get(state_id).unwrap().is_marked(*i) {
                f.write_fmt(format_args!("\\n{}", formula)).unwrap();
            }
        }
        f.write(b"\",").unwrap();
        if marks
            .marks
            .get(state_id)
            .unwrap()
            .is_marked(marks.subformulas.len() - 1)
        {
            f.write(b"style=filled,fillcolor=palegreen").unwrap();
        }
        f.write(b"];\n").unwrap();
    }

    // emit transitions
    for (src_id, trans) in lts.0.iter().enumerate() {
        for (label, dst_id) in trans.dst.iter() {
            f.write_fmt(format_args!(
                "{} -> {} [label=\"{}\"];\n",
                src_id, dst_id, label
            ))
            .unwrap()
        }
    }
    f.write(b"}}").unwrap();
}
