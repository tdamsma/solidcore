#![allow(clippy::unwrap_used, missing_docs)]
use solidcore::geom::attribute::{Attribute, Behavior};
use solidcore::math::Point3;
use solidcore::EntityRef;
use solidcore_smlib_api::Smlib;

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
fn probe_offset_attrs() {
    let mut api = Smlib::default();
    let mut a = api.make_box([0.0; 3], [20.0; 3]).unwrap();
    for (i, f) in a.face_list().to_vec().into_iter().enumerate() {
        a.add_attribute(
            EntityRef::Face(f),
            Attribute::long(7001, i as i64, Behavior::Copy),
        )
        .unwrap();
    }
    println!("PROBE source {}", kinds(&a));
    match api.shell(&a, &[[10.0, 10.0, 20.0]], 0.1, -2.0) {
        Ok(r) => println!("PROBE shell {}", kinds(&r)),
        Err(s) => println!("PROBE shell err {s} {:?}", api.last_error()),
    }
    match api.draft(
        &a,
        &[[20.0, 10.0, 10.0]],
        0.1,
        [0.0; 3],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        0.1,
    ) {
        Ok(r) => println!("PROBE draft {}", kinds(&r)),
        Err(s) => println!("PROBE draft err {s} {:?}", api.last_error()),
    }
    let mut sheet = solidcore::primitive::api_primitives::create_plane(
        api.kernel_mut(),
        Point3::new(0.0, 0.0, 0.0),
        10.0,
        10.0,
    )
    .unwrap();
    for f in sheet.face_list().to_vec() {
        sheet
            .add_attribute(EntityRef::Face(f), Attribute::long(7002, 1, Behavior::Copy))
            .unwrap();
    }
    println!("PROBE sheet {}", kinds(&sheet));
    match api.thicken(&sheet, 2.0, false) {
        Ok(r) => println!("PROBE thicken {}", kinds(&r)),
        Err(s) => println!("PROBE thicken err {s} {:?}", api.last_error()),
    }
    match api.offset_sheet(&sheet, 2.0) {
        Ok(r) => println!("PROBE offset_sheet {}", kinds(&r)),
        Err(s) => println!("PROBE offset_sheet err {s} {:?}", api.last_error()),
    }
}

#[test]
fn probe_stepped_shell() {
    let mut api = Smlib::default();
    let b = api.make_box([0.0, 0.0, 0.0], [30.0, 35.0, 7.5]).unwrap();
    let notch = api.make_box([0.0, 30.0, 2.5], [22.5, 20.0, 22.5]).unwrap();
    let stepped = api
        .boolean(&b, &notch, solidcore_smlib_api::BOOLEAN_SUBTRACT)
        .unwrap();
    let r = api.shell(&stepped, &[[18.75, 35.0, 1.25]], 0.1, -1.5);
    println!(
        "PROBE stepped strip (C: -1105): {:?} {:?}",
        r.as_ref().err(),
        api.last_error()
    );
    let r = api.shell(&stepped, &[[18.75, 32.5, 2.5]], 0.1, -1.5);
    println!(
        "PROBE stepped floor (C: OK): {:?} {:?}",
        r.as_ref().map(|b| b.face_list().len()).err(),
        api.last_error()
    );
}
