extern crate mcctl_rs;

use mcctl_rs::ddsv::{self, ExecUnit, Label, Location, Process, Trans};
use mcctl_rs::mcctl::{self, Formula, Prop};
use mcctl_rs::viz;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Vars {
    x: i32,
}

impl Vars {
    fn init() -> Self {
        Vars { x: 1 }
    }
}

fn main() {
    let process_p: Process<Vars> = vec![ExecUnit {
        src: Location::new("S"),
        transs: vec![
            Trans {
                label: Label::new(""),
                dst: Location::new("S"),
                guard: |vars| vars.x < 9,
                action: |mut vars| {
                    vars.x += 1;
                    vars
                },
            },
            Trans {
                label: Label::new(""),
                dst: Location::new("S"),
                guard: |vars| vars.x == 6,
                action: |mut vars| {
                    vars.x = 3;
                    vars
                },
            },
            Trans {
                label: Label::new(""),
                dst: Location::new("S"),
                guard: |vars| vars.x == 9,
                action: |mut vars| {
                    vars.x = 5;
                    vars
                },
            },
        ],
    }];

    let processes = vec![process_p];
    let mut lts = ddsv::concurrent_composition(Vars::init(), processes).unwrap();

    fn prop_valuate(prop: &Prop, vars: &Vars) -> bool {
        match prop.as_str() {
            "x<=7" => vars.x <= 7,
            "x>=4" => vars.x <= 4,
            _ => panic!(),
        }
    }

    let show_vars = |vars: &Vars| format!("x={}", vars.x);

    let spec1 = Formula::EG(Box::new(Formula::Prop("x<=7".to_string())));
    let fs1 = mcctl::mark(&mut lts, prop_valuate, spec1);
    viz::lts("test4-1.dot", show_vars, &lts, fs1);

    let spec2 = Formula::EG(Box::new(Formula::Prop("x>=4".to_string())));
    let fs2 = mcctl::mark(&mut lts, prop_valuate, spec2);
    viz::lts("test4-2.dot", show_vars, &lts, fs2);
}
