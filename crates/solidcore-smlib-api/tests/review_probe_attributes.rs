#![allow(clippy::unwrap_used, missing_docs)]
use solidcore::geom::attribute::{Attribute, Behavior};
use solidcore::EntityRef;
use solidcore_smlib_api::{Smlib, BOOLEAN_UNION};

#[test]
fn probe() {
    let mut api = Smlib::default();
    let mut a = api.make_box([0.0; 3], [10.0; 3]).unwrap();
    for f in a.face_list().to_vec() {
        a.add_attribute(EntityRef::Face(f), Attribute::long(7001, 1, Behavior::Copy)).unwrap();
    }
    println!("PROBE a attributed: {}", a.attributed_entities().len());
    let c = api.brep_copy(&a).unwrap();
    println!("PROBE copy attributed: {}", c.attributed_entities().len());
    let b = api.make_box([5.0; 3], [10.0; 3]).unwrap();
    let r = api.boolean(&a, &b, BOOLEAN_UNION).unwrap();
    println!("PROBE union attributed: {}", r.attributed_entities().len());
    let bytes = api.brep_write_bytes(&a, true).unwrap();
    let back = api.brep_read_bytes(&bytes, true).unwrap();
    println!("PROBE write/read attributed: {}", back.attributed_entities().len());
    let text = String::from_utf8_lossy(&bytes);
    println!("PROBE ascii mentions 7001: {}", text.contains("7001"));
    // reflection in x
    let m = [[-1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    println!("PROBE mirror: {:?}", api.brep_transform(&a, &m).map(|x| x.face_list().len()));
    let shear = [[1.0, 0.0, 0.0, 0.0], [0.5, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]];
    println!("PROBE shear: {:?}", api.brep_transform(&a, &shear).map(|x| x.face_list().len()));
    let t = [[2.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [1.0, 2.0, 3.0, 1.0]];
    let s = api.brep_transform(&a, &t).unwrap();
    println!("PROBE scaled vol: {:?} attributed {}", api.brep_volume(&s, 1e-9), s.attributed_entities().len());
    println!("PROBE mass props sheetless: {:?}", api.brep_mass_properties(&a, 1e-9).map(|m| m[0]));
}

#[test]
fn probe_hung_sheet() {
    use solidcore::math::Point3;
    let mut api = Smlib::default();
    let cube = solidcore::primitive::api_primitives::create_box(api.kernel_mut(), Point3::new(0.0, 0.0, 0.0), 10.0, 10.0, 10.0).unwrap();
    let flap = solidcore::primitive::api_primitives::create_plane(api.kernel_mut(), Point3::new(10.0, 0.0, 10.0), 10.0, 10.0).unwrap();
    let both = solidcore::boolean::api_brep::boolean_merge(api.kernel_mut(), cube, flap);
    match both {
        Ok(b) => println!("PROBE hung sheet mass: {:?}", api.brep_mass_properties(&b, 1e-8).map(|m| (m[0], m[1]))),
        Err(e) => println!("PROBE hung sheet merge failed: {e}"),
    }
}

#[test]
fn probe_union_direct() {
    use solidcore::math::Point3;
    use solidcore::geom::a_object::BooleanOperation;
    let mut api = Smlib::default();
    let k = api.kernel_mut();
    let mut a = solidcore::primitive::api_primitives::create_box(k, Point3::new(0.0, 0.0, 0.0), 10.0, 10.0, 10.0).unwrap();
    for f in a.face_list().to_vec() {
        a.add_attribute(EntityRef::Face(f), Attribute::long(7001, 1, Behavior::Copy)).unwrap();
    }
    let b = solidcore::primitive::api_primitives::create_box(k, Point3::new(5.0, 5.0, 5.0), 10.0, 10.0, 10.0).unwrap();
    let r = solidcore::boolean::api_brep::boolean(k, a.clone(), b, BooleanOperation::Union).unwrap().result;
    println!("PROBE direct union faces {} attributed {}", r.face_list().len(), r.attributed_entities().len());
    let b2 = api.make_box([5.0; 3], [10.0; 3]).unwrap();
    let r2 = api.boolean(&a, &b2, BOOLEAN_UNION).unwrap();
    println!("PROBE api union faces {} attributed {}", r2.face_list().len(), r2.attributed_entities().len());
    let mut fork = solidcore::Kernel::default();
    let a3 = { let mut x = solidcore::primitive::api_primitives::create_box(&mut fork, Point3::new(0.0, 0.0, 0.0), 10.0, 10.0, 10.0).unwrap(); for f in x.face_list().to_vec() { x.add_attribute(EntityRef::Face(f), Attribute::long(7001, 1, Behavior::Copy)).unwrap(); } x };
    let b3 = solidcore::primitive::api_primitives::create_box(&mut fork, Point3::new(5.0, 5.0, 5.0), 10.0, 10.0, 10.0).unwrap();
    let r3 = solidcore::boolean::api_brep::boolean(&mut fork, a3, b3, BooleanOperation::Union).unwrap().result;
    println!("PROBE default-kernel direct union faces {} attributed {}", r3.face_list().len(), r3.attributed_entities().len());
}

#[test]
fn probe_union_which() {
    let mut api = Smlib::default();
    let mut a = api.make_box([0.0; 3], [10.0; 3]).unwrap();
    for (i, f) in a.face_list().to_vec().into_iter().enumerate() {
        a.add_attribute(EntityRef::Face(f), Attribute::long(7001, i as i64, Behavior::Copy)).unwrap();
    }
    let b2 = api.make_box([5.0; 3], [10.0; 3]).unwrap();
    let c = api.brep_copy(&a).unwrap();
    let r2 = api.boolean(&a, &b2, BOOLEAN_UNION).unwrap();
    let rc = api.boolean(&c, &b2, BOOLEAN_UNION).unwrap();
    let k = api.kernel().clone();
    for (name, r) in [("a", &a), ("copy", &c), ("union", &r2), ("union-of-copy", &rc)] {
        let mut out = Vec::new();
        for f in r.face_list().to_vec() {
            let bx = solidcore::query::api_queries::face_bounding_box(&k, r, f, false).unwrap();
            let vals: Vec<i64> = r.attributed_entities().iter().filter(|e| **e == EntityRef::Face(f)).map(|_| 1).collect();
            out.push(format!("[{:.0},{:.0},{:.0}-{:.0},{:.0},{:.0}]{}", bx.u_min(), bx.v_min(), bx.w_min(), bx.u_max(), bx.v_max(), bx.w_max(), if vals.is_empty() {""} else {"*"}));
        }
        println!("PROBE {name}: {}", out.join(" "));
    }
}

#[test]
fn probe_entities() {
    let mut api = Smlib::default();
    let mut a = api.make_box([0.0; 3], [10.0; 3]).unwrap();
    for (i, f) in a.face_list().to_vec().into_iter().enumerate() {
        a.add_attribute(EntityRef::Face(f), Attribute::long(7001, i as i64, Behavior::Copy)).unwrap();
    }
    let c = api.brep_copy(&a).unwrap();
    let b2 = api.make_box([5.0; 3], [10.0; 3]).unwrap();
    let r = api.boolean(&a, &b2, BOOLEAN_UNION).unwrap();
    println!("PROBE a faces {:?}", a.face_list());
    println!("PROBE a attributed {:?}", a.attributed_entities());
    println!("PROBE copy faces {:?}", c.face_list());
    println!("PROBE copy attributed {:?}", c.attributed_entities());
    println!("PROBE union faces {:?}", r.face_list());
    println!("PROBE union attributed {:?}", r.attributed_entities());
}

#[test]
fn probe_plain_copy() {
    use solidcore::topology::brep::p2::ArenaNotifyState;
    let mut api = Smlib::default();
    let mut a = api.make_box([0.0; 3], [10.0; 3]).unwrap();
    for (i, f) in a.face_list().to_vec().into_iter().enumerate() {
        a.add_attribute(EntityRef::Face(f), Attribute::long(7001, i as i64, Behavior::Copy)).unwrap();
    }
    let fresh = solidcore::topology::Brep::new();
    let mut state = ArenaNotifyState::new(&fresh);
    let mut env = state.env(None);
    let c = solidcore::topology::brep::p3::api_brep_copy(api.kernel_mut(), &mut env, Some(&a), &solidcore::topology::loopuse::notify_ref).unwrap();
    println!("PROBE notify_ref copy attributed {:?}", c.attributed_entities());
}
