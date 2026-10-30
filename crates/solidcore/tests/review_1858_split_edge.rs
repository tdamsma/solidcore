//! Issue #1819 (WR-606; the fork's tangency-labs/smlib@14f18ed):
//! `SmFilletEdge::FindFilletBrepEdge`, looking for both edges of a fillet edge split in
//! two (`bFindMultipleEdges`), takes an edge whose midpoint drops onto the fillet edge's
//! curve within scaled zero as a match and keeps one within the edge tolerance only as
//! a "best" for when nothing matched. When one half matched to scaled zero and the other
//! only to tolerance, the second half was dropped, so `SmFilletCorner::
//! InsertIntersectionTopology` never merged it into its face (SolidCraft: a box corner's
//! eased r5 setback rounds came out with two free edges).
//!
//! The model is the difftest `blend.fillet_geom_p1` fixture `fb_find_edge_split_*`: a
//! fillet edge on the line (0,0,0)-(10,0,0) and fillet B-rep wire edges (tolerance
//! 1e-5): 0 the first half, its midpoint on the line; 1 the second half, ends on the
//! line, midpoint 1e-6 off it; 2 a crossing line 5e-7 off the midpoint (the fork's
//! B332 guard: its ends are far off the curve); 3 a near half whose far end is 1.5e-5
//! off. With `UpstreamFixes::fillet_split_edge_keeps_near_half` the closest near
//! edge is kept as the second when its two ends lie on the curve; off, as upstream.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // test helpers

use solidcore::blend::fillet_geom::p1::{FilletBrep, FilletVertex};
use solidcore::curve::curve::Curve;
use solidcore::kernel::Kernel;
use solidcore::math::Vec3;
use solidcore::nurbs::b_spline_curve::BSplineCurve;
use solidcore::nurbs::crv_basic::Curve as NurbCurve;
use solidcore::nurbs::math::{CPoint, NOW};
use solidcore::topology::{Brep, EdgeId};

fn line(a: [f64; 3], b: [f64; 3]) -> Curve {
    let pw = vec![
        CPoint::new(a[0], a[1], a[2], NOW),
        CPoint::new(b[0], b[1], b[2], NOW),
    ];
    let nurb = NurbCurve::new(pw, 1, vec![0.0, 0.0, 1.0, 1.0]).unwrap();
    Curve::from(BSplineCurve::from_nurb(nurb, 3, false))
}

/// A wire edge of the fillet B-rep on `curve` over its natural interval, tolerance 1e-5.
fn wire_edge(k: &Kernel, b: &mut Brep, curve: Curve) -> EdgeId {
    let ivl = curve.natural_interval();
    let ps = curve.evaluate_point(k, ivl.min()).unwrap();
    let pe = curve.evaluate_point(k, ivl.max()).unwrap();
    let vs = b.add_vertex(ps).unwrap();
    let ve = b.add_vertex(pe).unwrap();
    let e = b.add_edge(vs, ve).unwrap();
    let edge = b.edge_mut(e).unwrap();
    edge.curve = Some(curve);
    edge.interval = ivl;
    edge.tolerance = 1e-5;
    e
}

/// Runs `FindFilletBrepEdge(cands, TRUE)` and returns the indices (into the four
/// candidates) of the edges recorded as the first and second fillet B-rep edge.
fn find(fix: bool, cands: &[usize]) -> (Option<usize>, Option<usize>) {
    let mut k = Kernel::default();
    k.fixes.fillet_split_edge_keeps_near_half = fix;

    let mut fillet = Brep::new();
    let edges = [
        wire_edge(&k, &mut fillet, line([0.0, 0.0, 0.0], [5.0, 0.0, 0.0])),
        wire_edge(&k, &mut fillet, line([5.0, 0.0, 0.0], [10.0, 2e-6, 0.0])),
        wire_edge(&k, &mut fillet, line([7.5, -1.0, 5e-7], [7.5, 1.0, 5e-7])),
        wire_edge(&k, &mut fillet, line([5.0, 0.0, 0.0], [10.0, 0.0, 1.5e-5])),
    ];

    let mut fb = FilletBrep::new();
    let v0 = fb.brep_mut().add_vertex(Vec3::new(0.0, 0.0, 0.0)).unwrap();
    let v1 = fb.brep_mut().add_vertex(Vec3::new(10.0, 0.0, 0.0)).unwrap();
    fb.insert_fillet_vertex(v0, FilletVertex::new(&k, None));
    fb.insert_fillet_vertex(v1, FilletVertex::new(&k, None));
    let e = fb.new_fillet_edge(v0, v1, None).unwrap();
    fb.brep_mut().edge_mut(e).unwrap().curve = Some(line([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]));

    let cs: Vec<EdgeId> = cands.iter().map(|&i| edges[i]).collect();
    fb.find_fillet_brep_edge(&k, e, &fillet, &cs, true).unwrap();
    let rec = fb.fillet_edge(e).unwrap();
    let index = |x: Option<EdgeId>| x.map(|x| edges.iter().position(|&y| y == x).unwrap());
    (index(rec.fillet_brep_edge1), index(rec.fillet_brep_edge2))
}

#[test]
fn upstream_drops_the_near_second_half() {
    assert_eq!(find(false, &[0, 1]), (Some(0), None));
    assert_eq!(find(false, &[1, 0]), (Some(0), None));
}

#[test]
fn fix_keeps_the_near_second_half() {
    assert_eq!(find(true, &[0, 1]), (Some(0), Some(1)));
    assert_eq!(find(true, &[1, 0]), (Some(0), Some(1)));
    assert_eq!(find(true, &[3, 1, 0]), (Some(0), Some(1)));
}

#[test]
fn fix_refuses_a_near_edge_whose_ends_are_off_the_curve() {
    // a different curve crossing near the midpoint (B332)
    assert_eq!(find(true, &[0, 2]), (Some(0), None));
    // one end beyond the tolerance
    assert_eq!(find(true, &[0, 3]), (Some(0), None));
    // only the closest near edge is tried: the crossing one is closer than the half
    assert_eq!(find(true, &[0, 2, 1]), (Some(0), None));
}

#[test]
fn fix_leaves_the_no_match_fallback_alone() {
    // nothing matched to scaled zero: upstream's closest-approach fallback, both ways
    for fix in [false, true] {
        assert_eq!(find(fix, &[1]), (Some(1), None));
    }
}
