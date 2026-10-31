//! Review probe for #1825 (not a test of record): face attributes through imprint,
//! split_faces, stitch and untrim, and the SSI methods on two crossing planes.
#![allow(clippy::unwrap_used, missing_docs)]
use solidcore::geom::attribute::{Attribute, Behavior};
use solidcore::EntityRef;
use solidcore_smlib_api::{Brep, Smlib};

fn tagged(api: &mut Smlib, origin: [f64; 3]) -> Brep {
    let mut a = api.make_box(origin, [20.0; 3]).unwrap();
    for (i, f) in a.face_list().to_vec().into_iter().enumerate() {
        a.add_attribute(
            EntityRef::Face(f),
            Attribute::long(7001, i as i64, Behavior::Copy),
        )
        .unwrap();
    }
    a
}

fn kinds(b: &Brep) -> String {
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
fn probe_surface_tools() {
    let mut api = Smlib::default();
    let a = tagged(&mut api, [0.0; 3]);
    println!("PROBE source {}", kinds(&a));
    let tool = api.make_box([10.0; 3], [20.0; 3]).unwrap();
    match api.imprint(&a, &tool) {
        Ok(r) => println!("PROBE imprint {}", kinds(&r)),
        Err(s) => println!("PROBE imprint err {s} {:?}", api.last_error()),
    }
    let plane = api
        .surface_plane(
            [10.0, -5.0, -5.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 30.0],
            [0.0, 30.0],
        )
        .unwrap();
    match api.split_faces(&a, &[0, 1], &[&plane]) {
        Ok((r, made)) => println!("PROBE split_faces {} made {made:?}", kinds(&r)),
        Err(s) => println!("PROBE split_faces err {s} {:?}", api.last_error()),
    }
    match api.stitch(&a, 1e-3) {
        Ok((r, closed)) => println!("PROBE stitch {} closed {closed}", kinds(&r)),
        Err(s) => println!("PROBE stitch err {s} {:?}", api.last_error()),
    }
    match api.untrim_face(&a, [10.0, 10.0, 20.0], 0.1, None, true, false) {
        Ok(r) => println!("PROBE untrim {}", kinds(&r)),
        Err(s) => println!("PROBE untrim err {s} {:?}", api.last_error()),
    }
    let p1 = api
        .surface_plane(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [-10.0, 10.0],
            [-10.0, 10.0],
        )
        .unwrap();
    let p2 = api
        .surface_plane(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [-10.0, 10.0],
            [-10.0, 10.0],
        )
        .unwrap();
    for method in [0, 1, 2] {
        match api.ssi(&p1, &p2, method, 1e-6) {
            Ok(r) => println!("PROBE ssi method {method}: {} curves", r.curve_count()),
            Err(s) => println!("PROBE ssi method {method}: err {s} {:?}", api.last_error()),
        }
    }
}
