//! WR-253 addendum (the `SmLocalSolveNd` part of the SolidCraft fork's tangency-labs/smlib@d2757e1):
//! SolidCraft's rig case fillets/1207, step 4. The B-rep in
//! `tests/data/fork_d2757e1_rig_fillets_1207_step4.smb` is the one SolidCraft's fillet
//! session receives there (a 100 x 30 x 10 bar joined with a d40 boss, the boss's base arc
//! already rounded r 2); the step rounds the bar's top front edge r 2 together with the
//! section curve it runs on into. With `fillet_trace_reaches_rails` the round's end against
//! the boss is found by a surface/surface intersection whose parallel-point solve
//! (`SmLocalSolveNd`, normalize) takes a clamped step at the round's v = 1 boundary.
//! Upstream then keeps the unscaled point's Jacobian for the scaled point it takes, and the
//! solve converges 1.6e-8 off the boundary; the round's rail is trimmed 3.1e-8 short of a
//! knot instead of on it, and its UV trim gets coincident control points (`SmBSplineCurve`
//! tests #15, #3, #11). The fork, which takes the Jacobian with the step, lands on the
//! boundary and the body checks clean.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // test helpers
use solidcore::blend::fillet_executive::session::FilletSession;
use solidcore::blend::fillet_solver::p1::FilletSurfaceGeneratorType;
use solidcore::geom::attribute::{AttributeStore, ObjectKey};
use solidcore::mass::integrator::properties::{brep_volume, PropertyAttributes};
use solidcore::topology::api_files::read_brep_from_file;
use solidcore::topology::brep::p10::TopologyEnv;
use solidcore::topology::brep::p3::topology_ref;
use solidcore::topology::brep::p5::{stitch_and_orient, StitchAndOrientOptions};
use solidcore::topology::brep::validation::{check_brep, CheckLevel};
use solidcore::topology::topology::NotifyRecords;
use solidcore::topology::{edge, vertex, Brep, EdgeId};
use solidcore::{Kernel, Point3};

const SMB: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/fork_d2757e1_rig_fillets_1207_step4.smb"
);

fn read(k: &mut Kernel) -> Brep {
    let bytes = std::fs::read(SMB).unwrap();
    let mut attributes = AttributeStore::new();
    let mut records = NotifyRecords::new();
    let read_key = |e| topology_ref(1, e);
    let mut env = TopologyEnv {
        all_attributes: &[],
        attributes: &mut attributes,
        key_of: &read_key,
        records: &mut records,
        next_key: 1 << 50,
    };
    let mut b = read_brep_from_file(k, Some(bytes), false, false, &mut env).unwrap();
    b.refresh_cached_topology();
    b
}

/// The edge whose points at 0, 1/4, .., 1 of its interval are `pts` (either way round).
fn edge_by(k: &Kernel, b: &Brep, pts: &[[f64; 3]; 5]) -> EdgeId {
    let mut best = (f64::INFINITY, None);
    for e in b.edges().ids() {
        let c = edge::curve(b, e).unwrap().unwrap();
        let ivl = edge::interval(b, e).unwrap();
        let s: Vec<Point3> = [0.0, 0.25, 0.5, 0.75, 1.0]
            .iter()
            .map(|&u| c.evaluate_point(k, ivl.evaluate(u)).unwrap())
            .collect();
        let at = |i: usize| Point3::new(pts[i][0], pts[i][1], pts[i][2]);
        let fwd = (0..5).map(|i| s[i].distance(at(i))).fold(0.0, f64::max);
        let bwd = (0..5).map(|i| s[4 - i].distance(at(i))).fold(0.0, f64::max);
        let d = fwd.min(bwd);
        if d < best.0 {
            best = (d, Some(e));
        }
    }
    assert!(best.0 < 1e-9, "no edge matches: {}", best.0);
    best.1.unwrap()
}

struct Rounded {
    reports: Vec<String>,
    counts: [usize; 3],
    volume: f64,
    vertices: Vec<Point3>,
}

/// Step 4 of fillets/1207 with `fillet_trace_reaches_rails` (and the fork's rail-end
/// continuation the case needs to build at all), WR-253 addendum's fix on or off.
fn round(fix: bool) -> Rounded {
    let mut k = Kernel::new(&solidcore::kernel::KernelOptions::default());
    k.fixes.fillet_trace_reaches_rails = true;
    k.fixes.fillet_through_extended_rail_end = true;
    k.fixes.nd_solve_step_from_its_evaluation = fix;
    let mut b = read(&mut k);
    let top = edge_by(
        &k,
        &b,
        &[
            [16.09347693943108, -15.0, 10.0],
            [37.07010770457331, -15.0, 10.0],
            [58.04673846971554, -15.0, 10.0],
            [79.02336923485778, -15.0, 10.0],
            [100.0, -15.0, 10.0],
        ],
    );
    let section = edge_by(
        &k,
        &b,
        &[
            [13.228756555322962, -15.0, 11.999999999999996],
            [13.54215880044533, -14.999999999999892, 11.1105555177324],
            [14.259567152304003, -14.999999999999995, 10.483331405453916],
            [15.144481419353674, -14.999999999999982, 10.120740307933572],
            [16.093476939431085, -15.0, 10.0],
        ],
    );
    {
        let mut s = FilletSession::new(&mut k, &mut b).unwrap();
        s.set_blend_curve_at_thirds(false);
        s.prepare_per_edge_constant_radius(
            &mut k,
            &[top, section],
            &[2.0, 2.0],
            &[FilletSurfaceGeneratorType::Circular],
            1,
            1.0,
        )
        .unwrap();
        s.create_corners(&k, None).unwrap();
        s.set_mark_corner_patches(false);
        s.do_filleting(&mut k, None).unwrap();
    }
    stitch_and_orient(&mut k, &mut b, &StitchAndOrientOptions::default()).unwrap();
    b.refresh_cached_topology();
    let reports = check_brep(&mut k, &mut b, CheckLevel::Level2)
        .unwrap()
        .lines();
    let store = AttributeStore::default();
    let attrs = PropertyAttributes {
        store: &store,
        key_of: &|_| ObjectKey(0),
    };
    let volume = brep_volume(&k, &mut b, 1e-9, &attrs).unwrap();
    let counts = [
        b.face_list().len(),
        b.edge_list().len(),
        b.vertex_list().len(),
    ];
    let vertices = b
        .vertex_list()
        .iter()
        .map(|&v| vertex::point(&b, v).unwrap())
        .collect();
    Rounded {
        reports,
        counts,
        volume,
        vertices,
    }
}

/// The vertex where the round ends on the first round's top boundary against the boss, as
/// the fork (68fb9162) computes it.
const FORK_END_VERTEX: [f64; 3] = [13.833221775543047, -14.444444444444441, 11.999999999999996];

fn nearest(vertices: &[Point3], p: [f64; 3]) -> f64 {
    let p = Point3::new(p[0], p[1], p[2]);
    vertices
        .iter()
        .map(|v| v.distance(p))
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn upstream_steps_with_the_unscaled_jacobian_and_leaves_a_sliver_trim() {
    let r = round(false);
    assert_eq!(r.counts, [11, 25, 16]);
    assert!(
        r.reports
            .iter()
            .any(|l| l.contains("Test #15 Bad Repeated CPts")),
        "{:#?}",
        r.reports
    );
    // the round's end lands off the boundary (z 12.0000000157)
    assert!(
        nearest(&r.vertices, FORK_END_VERTEX) > 1e-8,
        "{:?}",
        r.vertices
    );
}

#[test]
fn the_fix_takes_the_step_from_its_evaluation_and_checks_clean() {
    let r = round(true);
    assert_eq!(r.counts, [11, 25, 16]);
    assert!(r.reports.is_empty(), "{:#?}", r.reports);
    // the fork's corner vertex (3.6e-11 away under this test's two flags; bit-identical
    // with SolidCraft's full flag set)
    assert!(
        nearest(&r.vertices, FORK_END_VERTEX) < 1e-9,
        "{:?}",
        r.vertices
    );
    // the fork's volume (SmApiBrepComputeVolume at 1e-9 on the fork's result:
    // 49712.428085478452)
    assert!(
        (r.volume - 49712.428085478452).abs() <= 1e-9 * 49712.428085478452,
        "{}",
        r.volume
    );
}
