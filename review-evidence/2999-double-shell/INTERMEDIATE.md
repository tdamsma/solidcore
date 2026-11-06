# Intermediate offset diagnosis

Main b7002862b, default ops fix set. The captured input exports to the ASCII SMB beside this file. The direct kernel regression in PR 2999 fails with shell_brep status 1001. The private cfg(test) probe calls the existing run_shell_offset with the same options, before shell validation/subtraction; its exact source and binary hash are retained. That probe was removed and the original production-source hash verified before publication.

The intermediate offset has 23 faces, 66 edges, 44 vertices and one region. It is not a manifold solid. Independent readback identifies 16 nonmanifold edges: two with three adjacent faces, fourteen with one. All are beside the two ends of the selected inner wall, x about ±29 and y -23..-23.5.

The moved floor is z=1.5 and the moved top/rim is z=29.5. The bridging faces still reach the original vertical-edge heights z=2..30: lower gaps span z=1.5..2; upper flaps span z=29.5..30 and meet the rim at three-face edges. Full coordinates and adjacent faces are in edge-inspection.log. These measurements localize the defect to the open-wall bridges' ends. The rail/centre-line endpoint construction is a source hypothesis, not yet a traced root cause or a proposed patch.

The native renderer cannot convert this invalid intermediate into a record: “a sheet touching itself along an edge”. The original input renders normally. The initial cached-topology integrity errors are not the geometric defect: after refresh_cached_topology, both input and intermediate pass integrity::check, while the intermediate remains nonmanifold.

Reproduction: run PR 2999's ignored issue_2954_second_shell test with ISSUE_2954_OUTPUT=/tmp/solidcore-2954-evidence to produce the original input. The archived private test can be inserted in p2.rs's existing tests module for a diagnostic build; it reuses NotificationFixture and invokes production run_shell_offset. It is not a production change. inspect_edges.rs then reads both saved SMBs through public Core APIs; compile it with the Core rlib from the same build. No committed case, expected value, known-failure marker, tolerance or production body changed.
