extern crate mcctl_rs;

use mcctl_rs::ddsv::{self, ExecUnit, Label, Location, Process, Trans};
use mcctl_rs::mcctl::{self, Formula, Prop};
use mcctl_rs::viz;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Vars {
    x: i32,
    y: i32,
    z: i32,
}

impl Vars {
    fn init() -> Self {
        Vars { x: 0, y: 0, z: 0 }
    }
}

fn main() {
    // x = 1
    // y = 1
    // z = 1
    // y = 0
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
            transs: vec![Trans {
                label: Label::new("y=1"),
                dst: Location::new("P2"),
                guard: |_| true,
                action: |mut vars| {
                    vars.y = 1;
                    vars
                },
            }],
        },
        ExecUnit {
            src: Location::new("P2"),
            transs: vec![Trans {
                label: Label::new("z=1"),
                dst: Location::new("P3"),
                guard: |_| true,
                action: |mut vars| {
                    vars.z = 1;
                    vars
                },
            }],
        },
        ExecUnit {
            src: Location::new("P3"),
            transs: vec![Trans {
                label: Label::new("z=1"),
                dst: Location::new("P4"),
                guard: |_| true,
                action: |mut vars| {
                    vars.y = 0;
                    vars
                },
            }],
        },
        ExecUnit {
            src: Location::new("P4"),
            transs: vec![],
        },
    ];

    let processes = vec![process_p];
    let mut lts = ddsv::concurrent_composition(Vars::init(), processes).unwrap();

    fn prop_valuate(prop: &Prop, vars: &Vars) -> bool {
        match prop.as_str() {
            "x=1" => vars.x == 1,
            "y>0" => vars.y > 0,
            "z=0" => vars.z == 0,
            _ => panic!(),
        }
    }

    let spec = Formula::Or(
        Box::new(Formula::And(
            Box::new(Formula::Prop("x=1".to_string())),
            Box::new(Formula::Prop("y>0".to_string())),
        )),
        Box::new(Formula::Not(Box::new(Formula::Prop("z=0".to_string())))),
    );

    let fs = mcctl::mark(&mut lts, prop_valuate, spec);

    let show_vars = |vars: &Vars| format!("x={} y={} z={}", vars.x, vars.y, vars.z);

    viz::lts("test1.dot", show_vars, &lts, fs);
}
