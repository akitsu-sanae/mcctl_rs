use crate::{
    formula::{Formula, Prop},
    lts::Lts,
};
use std::collections::VecDeque;

use bimap::BiMap;
use std::hash::Hash;
pub fn mark<T: Clone + Hash + Eq>(
    lts: &mut Lts<T>,
    prop_valuate: fn(&Prop, &T) -> bool,
    spec: Formula,
) -> BiMap<usize, Formula> {
    let subformulas = spec.unfold();
    // println!("subformulas: {:?}", subformulas);

    for i in 0..subformulas.len() {
        /*
        println!("-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-");
        println!(
            "subformula at {} ({:?})",
            i,
            subformulas.get_by_left(&i).unwrap()
        ); */
        mark_impl(lts, prop_valuate, i, &subformulas);
        // println!("lts: {:?}", lts);
    }
    subformulas
}

fn mark_impl<T: Clone + Hash + Eq>(
    lts: &mut Lts<T>,
    prop_valuate: fn(&Prop, &T) -> bool,
    i: usize,
    subformulas: &BiMap<usize, Formula>,
) {
    use Formula::*;
    match subformulas.get_by_left(&i).unwrap() {
        Prop(ref p) => lts.update_mark(|_, state_ex| prop_valuate(p, &state_ex.state.vars), i),
        Not(ref f) => {
            let f_index = subformulas.get_by_right(f).unwrap();
            lts.update_mark(|_, state_ex| !state_ex.is_marked(*f_index), i);
        }
        And(box ref f1, box ref f2) => {
            let f1_index = subformulas.get_by_right(f1).unwrap();
            let f2_index = subformulas.get_by_right(f2).unwrap();
            lts.update_mark(
                |_, state_ex| state_ex.is_marked(*f1_index) && state_ex.is_marked(*f2_index),
                i,
            );
        }
        Or(box ref f1, box ref f2) => {
            let f1_index = subformulas.get_by_right(f1).unwrap();
            let f2_index = subformulas.get_by_right(f2).unwrap();
            lts.update_mark(
                |_, state_ex| state_ex.is_marked(*f1_index) || state_ex.is_marked(*f2_index),
                i,
            );
        }
        EX(box ref f) => {
            let f_index = subformulas.get_by_right(f).unwrap();
            let need_update_ids = lts.find_states(|_, state_ex| {
                state_ex
                    .transs
                    .iter()
                    .any(|(_, succ_id)| lts.0.get(&succ_id).unwrap().is_marked(*f_index))
            });
            for state_id in need_update_ids {
                lts.0.get_mut(&state_id).unwrap().mark(i)
            }
        }
        EU(box ref f1, box ref f2) => {
            let f1_index = subformulas.get_by_right(f1).unwrap();
            let f2_index = subformulas.get_by_right(f2).unwrap();

            let mut need_update_ids = lts.find_states(|_, state_ex| state_ex.is_marked(*f2_index));
            let mut queue = VecDeque::from(need_update_ids.clone());
            loop {
                if let Some(eu_id) = queue.pop_front() {
                    let mut founds = lts.find_states(|state_id, state_ex| {
                        state_ex.transs.iter().any(|(_, x)| *x == eu_id)
                            && state_ex.is_marked(*f1_index)
                            && !need_update_ids.iter().any(|x| *x == state_id) // not already exists
                    });
                    queue.append(&mut VecDeque::from(founds.clone()));
                    need_update_ids.append(&mut founds);
                } else {
                    break;
                }
            }
            for state_id in need_update_ids {
                lts.0.get_mut(&state_id).unwrap().mark(i)
            }
        }
        EG(box f) => {
            let f_index = subformulas.get_by_right(f).unwrap();
            let mut need_update_ids =
                Vec::from(lts.find_states(|_, state_ex| state_ex.is_marked(*f_index)));

            // calc gfp
            loop {
                let unmark_ids: Vec<_> = need_update_ids
                    .clone()
                    .into_iter()
                    .filter(|state_id| {
                        let state_ex = lts.0.get(&state_id).unwrap();
                        !state_ex
                            .transs
                            .iter()
                            .any(|(_, next_id)| need_update_ids.contains(next_id))
                    })
                    .collect();
                if unmark_ids.is_empty() {
                    // already at fixed point
                    break;
                }
                for unmark_id in unmark_ids {
                    need_update_ids.retain(|x| *x != unmark_id);
                }
            }

            for state_id in need_update_ids {
                lts.0.get_mut(&state_id).unwrap().mark(i)
            }
        }
        _ => unimplemented!(),
    }
}
