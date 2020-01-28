extern crate mcctl_rs;

use mcctl_rs::{
    formula::{Formula, Prop},
    lts::Lts,
    mark,
    process::{ExecUnit, Label, Location, Process, Trans},
    viz,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Vars {
    x: i32,
}

impl Vars {
    fn init() -> Self {
        Vars { x: 1 }
    }
}

use std::fmt;
impl fmt::Display for Vars {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "x={}", self.x)
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
    let mut lts = Lts::concurrent_composition(Vars::init(), processes).unwrap();

    fn prop_valuate(prop: &Prop, vars: &Vars) -> bool {
        match prop.as_str() {
            "x<=7" => vars.x <= 7,
            "x>=4" => vars.x <= 4,
            _ => panic!(),
        }
    }

    let spec1 = Formula::EG(Box::new(Formula::Prop("x<=7".to_string())));
    let fs1 = mark::mark(&mut lts, prop_valuate, spec1);
    viz::lts("test4-1.dot", &lts, fs1);

    let spec2 = Formula::EG(Box::new(Formula::Prop("x>=4".to_string())));
    let fs2 = mark::mark(&mut lts, prop_valuate, spec2);
    viz::lts("test4-2.dot", &lts, fs2);
}
