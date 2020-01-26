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
        Vars { x: 0 }
    }
}

fn main() {
    let process_p: Process<Vars> = vec![
        ExecUnit {
            src: Location::new("P0"),
            transs: vec![Trans {
                label: Label::new("x=1"),
                dst: Location::new("P1"),
                guard: |_| true,
                action: |mut vars| {
                    vars.x = 1;
                    vars
                },
            }],
        },
        ExecUnit {
            src: Location::new("P1"),
            transs: vec![
                Trans {
                    label: Label::new("x=2"),
                    dst: Location::new("P2"),
                    guard: |_| true,
                    action: |mut vars| {
                        vars.x = 2;
                        vars
                    },
                },
                Trans {
                    label: Label::new("x=3"),
                    dst: Location::new("P2"),
                    guard: |_| true,
                    action: |mut vars| {
                        vars.x = 3;
                        vars
                    },
                },
                Trans {
                    label: Label::new("x=4"),
                    dst: Location::new("P2"),
                    guard: |_| true,
                    action: |mut vars| {
                        vars.x = 4;
                        vars
                    },
                },
            ],
        },
        ExecUnit {
            src: Location::new("P2"),
            transs: vec![Trans {
                label: Label::new("x--"),
                dst: Location::new("P3"),
                guard: |_| true,
                action: |mut vars| {
                    vars.x -= 1;
                    vars
                },
            }],
        },
        ExecUnit {
            src: Location::new("P3"),
            transs: vec![],
        },
    ];

    let processes = vec![process_p];
    let mut lts = ddsv::concurrent_composition(Vars::init(), processes).unwrap();

    fn prop_valuate(prop: &Prop, vars: &Vars) -> bool {
        match prop.as_str() {
            "x=2" => vars.x == 2,
            _ => panic!(),
        }
    }

    let spec = Formula::EX(Box::new(Formula::EX(Box::new(Formula::EX(Box::new(
        Formula::Prop("x=2".to_string()),
    ))))));

    let fs = mcctl::mark(&mut lts, prop_valuate, spec);

    let show_vars = |vars: &Vars| format!("x={}", vars.x);

    viz::lts(show_vars, &lts, fs);
}
