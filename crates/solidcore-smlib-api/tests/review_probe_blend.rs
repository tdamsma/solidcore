#![allow(clippy::unwrap_used, missing_docs)]
use solidcore::geom::attribute::{Attribute, Behavior};
use solidcore::EntityRef;
use solidcore_smlib_api::{Smlib, Status};
use std::f64::consts::PI;

fn tagged(api: &mut Smlib) -> solidcore::topology::Brep {
    let mut a = api.make_box([0.0; 3], [20.0; 3]).unwrap();
    for (i, f) in a.face_list().to_vec().into_iter().enumerate() {
        a.add_attribute(
            EntityRef::Face(f),
            Attribute::long(7001, i as i64, Behavior::Copy),
        )
        .unwrap();
    }
    a
}

fn kinds(b: &solidcore::topology::Brep) -> String {
    let e = b.attributed_entities();
    let f = e.iter().filter(|x| matches!(x, EntityRef::Face(_))).count();
    format!(
        "{} attributed ({} faces, {} other) of {} faces",
        e.len(),
        f,
        e.len() - f,
        b.face_list().len()
    )
}

#[test]
fn probe_blend_attrs() {
    let mut api = Smlib::default();
    let a = tagged(&mut api);
    println!("PROBE source {}", kinds(&a));
    let edge = [[10.0, 0.0, 20.0]];
    match api.fillet(&a, &edge, 0.1, 2.0) {
        Ok(r) => println!("PROBE fillet {}", kinds(&r)),
        Err(s) => println!("PROBE fillet err {s}"),
    }
    match api.chamfer(&a, &edge, 0.1, 2.0) {
        Ok(r) => println!("PROBE chamfer {}", kinds(&r)),
        Err(s) => println!("PROBE chamfer err {s}"),
    }
}

#[test]
fn probe_rim_fillet_and_60deg() {
    let mut api = Smlib::default();
    let can = api.make_cylinder([0.0; 3], 10.0, 20.0).unwrap();
    let at = 100.0 * PI / 180.0;
    let top_rim = [[10.0 * at.cos(), 10.0 * at.sin(), 20.0]];
    let down = [0.0, 0.0, -1.0];
    for d in [9.99f64, 10.0] {
        let r = api.fillet(&can, &top_rim, 0.1, d);
        let want = if d == 10.0 {
            PI * 100.0 * 10.0 + 2.0 / 3.0 * PI * 1000.0
        } else {
            PI * 100.0 * 20.0
                - 2.0
                    * PI
                    * (10.0 - d * (10.0 - 3.0 * PI) / (12.0 - 3.0 * PI))
                    * (1.0 - PI / 4.0)
                    * d
                    * d
        };
        match r {
            Ok(b) => println!(
                "PROBE rim fillet {d}: vol {} want {want} faces {}",
                api.brep_volume(&b, 1e-9).unwrap(),
                b.face_list().len()
            ),
            Err(s) => println!("PROBE rim fillet {d}: err {s} last {:?}", api.last_error()),
        }
        let r = api.chamfer_sides(&can, &top_rim, 0.1, d / (PI / 3.0).tan(), PI / 3.0, 1, down);
        match r {
            Ok(b) => println!(
                "PROBE 60deg {d}: vol {}",
                api.brep_volume(&b, 1e-9).unwrap()
            ),
            Err(s) => println!("PROBE 60deg {d}: err {s}"),
        }
    }
}

#[test]
fn probe_linear_law() {
    let mut api = Smlib::default();
    let a = api.make_box([0.0; 3], [20.0; 3]).unwrap();
    let p = [10.0, 0.0, 20.0];
    let s = [0.0, 0.0, 20.0];
    // straight law with an interior knot: no slope jump
    let r = api.fillet_variable(
        &a,
        p,
        0.1,
        s,
        1,
        &[0.0, 0.0, 0.5, 1.0, 1.0],
        &[1.0, 1.5, 2.0],
    );
    println!(
        "PROBE collinear degree-1 law: {:?}",
        r.as_ref().map(|b| b.face_list().len()).err()
    );
    // corner law, point that picks nothing
    let r = api.fillet_variable(
        &a,
        [100.0, 100.0, 100.0],
        0.1,
        s,
        1,
        &[0.0, 0.0, 0.5, 1.0, 1.0],
        &[1.0, 2.0, 1.0],
    );
    println!(
        "PROBE corner law, far point: {:?} (C: PICK_NONE {})",
        r.err(),
        Status::ERR_PICK_NONE
    );
}
