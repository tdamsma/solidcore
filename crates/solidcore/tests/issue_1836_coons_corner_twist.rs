//! Regression test for #1836 (the fork's tangency-labs/smlib@7fd9fb0): a fillet corner's
//! bicubic Coons patch takes each corner's twist as the mean of its two sides' twists.
//! Where one side lies on a face of the body and the other on a blend, the twists differ
//! along the normal (each is its own surface's curvature on the two boundary tangents),
//! so the mean tilts the patch's cross derivative off the face along that side. With
//! `coons_corner_twist_from_blend_side`, such a corner takes the blend side's twist and
//! the patch meets the face as the face's own twist has it.
//!
//! The issue's trigger (SolidCraft's test-peg: a stadium bar running tangent into a
//! cylinder, its front top edge and the concave arc at the wall rounded together) needs
//! the fork's 3x2 tangent-side-edge fixes (WR-597, WR-600), not ported yet: the round
//! fails 1001 in the port with and without this flag. The same 3x2 mixed-convexity
//! corner without the tangency is used instead: an L-shaped box (20 x 20 x 10 less
//! x 10..20, y 10..20) with the notch's concave vertical edge and one convex top edge
//! at the corner (10, 10, 10) rounded r 2. The patch's two sides on the planar faces
//! meet them at 26.2 and 38.6 degrees by default, and tangentially with the flag.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // test helpers

use solidcore::blend::fillet_executive::api::fillet_edges;
use solidcore::boolean::api_brep::boolean_difference;
use solidcore::math::Point3;
use solidcore::nurbs::knots::Side;
use solidcore::primitive::api_primitives::create_box;
use solidcore::query::api_queries::surface_closest_point;
use solidcore::surface::surface::{Surface, SurfaceKind};
use solidcore::topology::face::p2 as face;
use solidcore::topology::{edge, vertex, EdgeId};
use solidcore::Kernel;

/// The worst angle, in degrees, between the corner patch (the one B-spline face) and the
/// planar faces it meets, sampled at 39 interior points of each seam, after rounding
/// the notch's concave vertical edge and its convex top edge along x = 10 at `r`.
fn worst_planar_seam(twist_fix: bool, r: f64) -> f64 {
    let mut k = Kernel::default();
    k.fixes.coons_corner_twist_from_blend_side = twist_fix;
    let block = create_box(&mut k, Point3::new(0.0, 0.0, 0.0), 20.0, 20.0, 10.0).unwrap();
    let notch = create_box(&mut k, Point3::new(10.0, 10.0, -1.0), 20.0, 20.0, 12.0).unwrap();
    let mut b = boolean_difference(&mut k, block, notch).unwrap();
    b.refresh_cached_topology();

    let corner = Point3::new(10.0, 10.0, 10.0);
    let concave_end = Point3::new(10.0, 10.0, 0.0);
    let convex_end = Point3::new(10.0, 20.0, 10.0);
    let mut edges: Vec<EdgeId> = Vec::new();
    for e in b.edges().ids() {
        let (Some(s), Some(t)) = edge::start_end_vertices(&b, e).unwrap() else {
            continue;
        };
        let (ps, pt) = (vertex::point(&b, s).unwrap(), vertex::point(&b, t).unwrap());
        let other = if ps.distance(corner) < 1e-9 {
            pt
        } else if pt.distance(corner) < 1e-9 {
            ps
        } else {
            continue;
        };
        if other.distance(concave_end) < 1e-9 || other.distance(convex_end) < 1e-9 {
            edges.push(e);
        }
    }
    assert_eq!(
        edges.len(),
        2,
        "the concave and the convex edge at the corner"
    );
    fillet_edges(&mut k, &mut b, &edges, r, 1, 1, 1.0).unwrap();

    let normal = |s: &Surface, p: Point3| {
        let uv = surface_closest_point(&k, s, p).unwrap().uvs[0];
        s.evaluate_normal(&k, uv, Side::Left, Side::Left).unwrap()
    };
    let is = |s: &Surface, plane: bool| {
        if plane {
            matches!(s.kind(), SurfaceKind::Plane(_))
        } else {
            matches!(s.kind(), SurfaceKind::BSpline(_))
        }
    };
    let mut worst: f64 = 0.0;
    let mut seams = 0;
    for e in b.edges().ids() {
        let faces = edge::faces(&b, e, None).unwrap();
        let [f1, f2] = faces[..] else { continue };
        let (Some(s1), Some(s2)) = (
            face::surface(&b, f1).unwrap(),
            face::surface(&b, f2).unwrap(),
        ) else {
            continue;
        };
        let (patch, plane) = if is(s1, false) && is(s2, true) {
            (s1, s2)
        } else if is(s2, false) && is(s1, true) {
            (s2, s1)
        } else {
            continue;
        };
        seams += 1;
        let c = edge::curve(&b, e).unwrap().unwrap();
        let iv = edge::interval(&b, e).unwrap();
        for i in 1..40 {
            let p = c
                .evaluate_point(&k, iv.min() + iv.length() * f64::from(i) / 40.0)
                .unwrap();
            let cos = normal(patch, p).dot(normal(plane, p)).abs().min(1.0);
            worst = worst.max(cos.acos().to_degrees());
        }
    }
    assert_eq!(seams, 2, "the corner patch meets two planar faces");
    worst
}

#[test]
fn issue_1836_mean_twist_tilts_patch_off_its_faces_by_default() {
    let worst = worst_planar_seam(false, 2.0);
    assert!(worst > 20.0, "upstream's mean twist: {worst} degrees");
}

#[test]
fn issue_1836_corner_patch_meets_its_faces_with_the_fix() {
    for r in [1.0, 2.0, 3.0] {
        let worst = worst_planar_seam(true, r);
        assert!(worst < 1e-3, "r {r}: {worst} degrees off a planar face");
    }
}
