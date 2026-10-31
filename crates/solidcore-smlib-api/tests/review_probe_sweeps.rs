//! Review probe for #1829 (not a test of record): sweep results' surface kinds, volumes and
//! SMLib-check reports for the sweep cases solidcraft#8526 tracked.
#![allow(clippy::unwrap_used, missing_docs)]
use std::f64::consts::PI;

use solidcore::surface::surface::SurfaceKind;
use solidcore_smlib_api::{
    Brep, Smlib, CURVE_ARC, CURVE_LINE, DRAFT_CORNER_CURVATURE, SWEEP_FOLLOW,
};

fn kinds_of(b: &Brep) -> String {
    let mut v: Vec<&str> = b
        .face_list()
        .iter()
        .map(
            |&f| match b.face(f).unwrap().surface.as_ref().map(|s| s.kind()) {
                Some(SurfaceKind::Plane(_)) => "plane",
                Some(SurfaceKind::Cone(_)) => "cone/cylinder",
                Some(SurfaceKind::Sphere(_)) => "sphere",
                Some(SurfaceKind::Torus(_)) => "torus",
                Some(SurfaceKind::SurfOfRevolution(_)) => "revolution",
                Some(SurfaceKind::SurfOfExtrusion(_)) => "extrusion",
                Some(SurfaceKind::BSpline(_)) => "bspline",
                Some(_) => "other",
                None => "none",
            },
        )
        .collect();
    v.sort_unstable();
    v.join(",")
}

fn report(api: &mut Smlib, what: &str, r: solidcore_smlib_api::Result<Brep>, want: f64) {
    match r {
        Ok(b) => {
            let v = api.brep_volume(&b, 1e-9).unwrap_or(f64::NAN);
            let check = api
                .brep_check(&b, 2)
                .map(|(text, n)| (n, text.lines().next().unwrap_or("").to_string()));
            println!(
                "PROBE {what}: vol {v} (want {want}, rel {:.1e}) faces [{}] check {:?}",
                (v - want) / want,
                kinds_of(&b),
                check
            );
        }
        Err(s) => println!("PROBE {what}: err {s} {:?}", api.last_error()),
    }
}

#[test]
fn probe_sweeps() {
    let mut api = Smlib::default();
    // a full circle r 5 in z = 0, extruded 10 up: a cylinder (#1477, linear_sweep_arc_is_cylinder)
    let circle = [
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
        1.0,
        0.0,
        0.0,
        5.0,
        0.0,
        2.0 * PI,
    ];
    let r = api.extrude(
        &[CURVE_ARC],
        &circle,
        [0.0, 0.0, 1.0],
        10.0,
        0.0,
        DRAFT_CORNER_CURVATURE,
        1,
    );
    report(&mut api, "extruded circle", r, PI * 25.0 * 10.0);
    // a half circle and its diameter, extruded: one half-cylinder wall
    let half = [
        0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 5.0, 0.0, PI, -5.0, 0.0, 0.0, 5.0, 0.0, 0.0,
    ];
    let r = api.extrude(
        &[CURVE_ARC, CURVE_LINE],
        &half,
        [0.0, 0.0, 1.0],
        10.0,
        0.0,
        DRAFT_CORNER_CURVATURE,
        1,
    );
    report(&mut api, "extruded half disc", r, PI * 12.5 * 10.0);
    // one circle off the axis, a full turn: a torus (#1491 one-circle full revolve)
    let ring = [
        20.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        1.0,
        0.0,
        0.0,
        5.0,
        0.0,
        2.0 * PI,
    ];
    let r = api.revolve(&[CURVE_ARC], &ring, [0.0; 3], [0.0, 0.0, 1.0], 2.0 * PI, 1);
    report(
        &mut api,
        "revolved circle (torus)",
        r,
        2.0 * PI * PI * 20.0 * 25.0,
    );
    // a half disc about -z: a sphere (#1672 horn-torus case), and about +z
    let disc = [
        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 10.0, 0.0, PI, 0.0, 0.0, -10.0, 0.0, 0.0, 10.0,
    ];
    for (name, axis) in [
        ("half disc about -z", [0.0, 0.0, -1.0]),
        ("half disc about +z", [0.0, 0.0, 1.0]),
    ] {
        let r = api.revolve(&[CURVE_ARC, CURVE_LINE], &disc, [0.0; 3], axis, 2.0 * PI, 1);
        report(&mut api, name, r, 4.0 / 3.0 * PI * 1000.0);
    }
    // a rectangle in x = 10..12 revolved a full turn: a tube (inside-out cylinder frame, #1463)
    let rect = [
        10.0, 0.0, 0.0, 12.0, 0.0, 0.0, 12.0, 0.0, 0.0, 12.0, 0.0, 5.0, 12.0, 0.0, 5.0, 10.0, 0.0,
        5.0, 10.0, 0.0, 5.0, 10.0, 0.0, 0.0,
    ];
    let r = api.revolve(
        &[CURVE_LINE; 4],
        &rect,
        [0.0; 3],
        [0.0, 0.0, 1.0],
        2.0 * PI,
        1,
    );
    report(
        &mut api,
        "revolved rectangle (tube)",
        r,
        PI * (144.0 - 100.0) * 5.0,
    );
    // a unit square swept along a line then a quarter arc (piecewise exact sweep)
    let sq = [
        -0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.5, 0.5, 0.0, 0.5, 0.5, 0.0, -0.5, 0.5,
        0.0, -0.5, 0.5, 0.0, -0.5, -0.5, 0.0,
    ];
    let path = [
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        10.0,
        10.0,
        0.0,
        10.0,
        0.0,
        1.0,
        0.0,
        -1.0,
        0.0,
        0.0,
        10.0,
        0.0,
        0.5 * PI,
    ];
    let r = api.sweep(
        &[CURVE_LINE; 4],
        &sq,
        &[CURVE_LINE, CURVE_ARC],
        &path,
        SWEEP_FOLLOW,
        1,
    );
    report(&mut api, "square along line+arc", r, 10.0 + 0.5 * PI * 10.0);
}
