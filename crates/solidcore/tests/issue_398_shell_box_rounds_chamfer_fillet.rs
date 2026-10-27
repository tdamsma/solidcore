//! tangency-labs/solidcore#398: SolidCraft rig case shell/1525 (`tangency-labs/solidcraft`,
//! branch `kernel-smlib`, `tests/kernel-cases/shell/1525-shell-box-with-rounds-chamfer-and-fillet`):
//! a 50 x 50 x 30 box, its four vertical edges rounded r10, a 5 mm chamfer round the base,
//! a 4 mm fillet where the chamfer meets the wall, shelled 2 mm inward open at the top. The
//! SMLib fork failed it with status 1001; rebuilt through the fork's C API it now succeeds
//! with volume 13701.73. The Rust port reproduces a defect: the shell (open or closed)
//! returns success with a non-manifold body of zero volume (72 faces), so the open-top test
//! is `--ignored` until it is fixed. Without the 4 mm fillet both shells are manifold.
//!
//! The recipe follows the rig's commands as the fork's C API runs them: rounds as
//! `SmApiFilletEdgesPerEdge` (circular, constant radius), the chamfer as a linear
//! constant-radius section at offset 5 (equal setbacks across a right angle) with every
//! corner a bevel, then stitching; edges and faces are picked by the rig's points.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // test helpers

#[path = "support/offset_fixture.rs"]
mod offset_fixture;

use offset_fixture::{planar_faces_at, report, shell, volume};
use solidcore::blend::fillet_executive::session::FilletSession;
use solidcore::blend::fillet_solver::p1::FilletSurfaceGeneratorType;
use solidcore::math::Point3;
use solidcore::primitive::api_primitives::create_box;
use solidcore::topology::brep::p5::{stitch_and_orient, StitchAndOrientOptions};
use solidcore::{Brep, EdgeId, Kernel};

/// The fork's C API result for the shelled part (issue comment).
const FORK_VOLUME: f64 = 13_701.73;

/// The edge of `b` nearest each pick (sampled along each edge's curve).
fn edges_at(k: &Kernel, b: &Brep, picks: &[[f64; 3]]) -> Vec<EdgeId> {
    picks
        .iter()
        .map(|&[x, y, z]| {
            let p = Point3::new(x, y, z);
            let (d, id) = b
                .edges()
                .iter()
                .filter_map(|(id, e)| {
                    let c = e.curve.as_ref()?;
                    let (t0, t1) = (e.interval.min(), e.interval.max());
                    let d = (0..=256)
                        .map(|i| {
                            let q = c
                                .evaluate_point(k, t0 + (t1 - t0) * f64::from(i) / 256.0)
                                .unwrap();
                            (q - p).length()
                        })
                        .fold(f64::INFINITY, f64::min);
                    Some((d, id))
                })
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .unwrap();
            assert!(d < 1e-2, "no edge at {p:?} (nearest {d})");
            id
        })
        .collect()
}

fn round(k: &mut Kernel, b: &mut Brep, picks: &[[f64; 3]], radius: f64) {
    let edges = edges_at(k, b, picks);
    {
        let mut session = FilletSession::new(k, b).unwrap();
        session
            .prepare_per_edge_constant_radius(
                k,
                &edges,
                &vec![radius; edges.len()],
                &vec![FilletSurfaceGeneratorType::Circular; edges.len()],
                1,
                1.0,
            )
            .unwrap();
        session.do_fillet(k).unwrap();
    }
    stitch_and_orient(k, b, &StitchAndOrientOptions::default()).unwrap();
}

fn chamfer(k: &mut Kernel, b: &mut Brep, picks: &[[f64; 3]], distance: f64) {
    let edges = edges_at(k, b, picks);
    {
        let mut session = FilletSession::new(k, b).unwrap();
        session
            .prepare_per_edge_constant_radius(
                k,
                &edges,
                &vec![distance; edges.len()],
                &vec![FilletSurfaceGeneratorType::Linear; edges.len()],
                1,
                1.0,
            )
            .unwrap();
        session.create_corners(k, None).unwrap();
        for corner in session.fillet_corners().unwrap() {
            session.set_corner_bevel(corner, true).unwrap();
        }
        session.do_filleting(k, None).unwrap();
    }
    stitch_and_orient(k, b, &StitchAndOrientOptions::default()).unwrap();
}

fn part(k: &mut Kernel) -> Brep {
    let mut b = create_box(k, Point3::new(0.0, 0.0, 0.0), 50.0, 50.0, 30.0).unwrap();
    round(
        k,
        &mut b,
        &[
            [0.0, 0.0, 15.0],
            [50.0, 0.0, 15.0],
            [0.0, 50.0, 15.0],
            [50.0, 50.0, 15.0],
        ],
        10.0,
    );
    let base = |z: f64| {
        [
            [25.0, 0.0, z],
            [50.0, 25.0, z],
            [25.0, 50.0, z],
            [0.0, 25.0, z],
            [2.929, 2.929, z],
            [47.071, 2.929, z],
            [47.071, 47.071, z],
            [2.929, 47.071, z],
        ]
    };
    chamfer(k, &mut b, &base(0.0), 5.0);
    round(k, &mut b, &base(5.0), 4.0);
    b.refresh_cached_topology();
    b
}

#[test]
fn issue_398_part_input() {
    let mut k = Kernel::default();
    let mut b = part(&mut k);
    let got = volume(&k, &mut b).unwrap();
    eprintln!(
        "PROBE issue_398 input: faces={} volume={got}",
        b.face_list().len()
    );
    assert!(report(&k, "issue_398 input", Ok(b), None, 0.0));
}

#[test]
#[ignore = "reproduces #398: the port returns a non-manifold body of zero volume"]
fn issue_398_open_top_shell() {
    let mut k = Kernel::default();
    let b = part(&mut k);
    let top = planar_faces_at(&k, &b, 2, 30.0);
    assert_eq!(top.len(), 1, "the top face");
    let result = shell(&mut k, b, -2.0, &top);
    assert!(report(
        &k,
        "issue_398 open top",
        result,
        Some(FORK_VOLUME),
        1e-6
    ));
}

/// Narrows the open-top failure: the shell after each step of the recipe, open at the
/// top and closed (reported only).
#[test]
#[ignore = "diagnostic probe, run on demand"]
fn issue_398_by_step() {
    for steps in 1..=3 {
        for open in [true, false] {
            let mut k = Kernel::default();
            let mut b = create_box(&mut k, Point3::new(0.0, 0.0, 0.0), 50.0, 50.0, 30.0).unwrap();
            round(
                &mut k,
                &mut b,
                &[
                    [0.0, 0.0, 15.0],
                    [50.0, 0.0, 15.0],
                    [0.0, 50.0, 15.0],
                    [50.0, 50.0, 15.0],
                ],
                10.0,
            );
            let base = |z: f64| {
                [
                    [25.0, 0.0, z],
                    [50.0, 25.0, z],
                    [25.0, 50.0, z],
                    [0.0, 25.0, z],
                    [2.929, 2.929, z],
                    [47.071, 2.929, z],
                    [47.071, 47.071, z],
                    [2.929, 47.071, z],
                ]
            };
            if steps >= 2 {
                chamfer(&mut k, &mut b, &base(0.0), 5.0);
            }
            if steps >= 3 {
                round(&mut k, &mut b, &base(5.0), 4.0);
            }
            b.refresh_cached_topology();
            let top = if open {
                planar_faces_at(&k, &b, 2, 30.0)
            } else {
                Vec::new()
            };
            let result = shell(&mut k, b, -2.0, &top);
            let _ = report(
                &k,
                &format!("issue_398 steps={steps} open={open}"),
                result,
                None,
                0.0,
            );
        }
    }
}
