Native control for the retained cliff trimming continuation

This standalone control calls the original `SmTrimmingTools::TrimSurfaceWithModelSpaceCurves` with a real infinite region, an original Plane and four Line segments. It uses the fillet call’s flags: first loop outer, loops not pre-oriented, no seam crossing.

Run from the repository root against the existing C++ reference (no rebuild):

```sh
nice -n 10 g++ -std=c++17 -O2 -ffp-contract=off -fno-fast-math \
  -DNDEBUG -DNOMINMAX -DSM_GRAPHICS_CALLBACKS=1 -D_GLIBCXX_USE_CXX11_ABI=1 \
  -Iupstream/source/SMLib/inc harness/cpp/probes/original_g1_trim.cpp \
  -Lbin/cpp/lib -lSMLib -lpthread -o /tmp/solidcore-original-g1-trim
nice -n 10 /tmp/solidcore-original-g1-trim
```

Measured with upstream `c9c5979`: status 1000 (SM_SUCCESS), 4 vertices, 4 edges, 1 face, body tolerance 1e-5. The installed surface is the original object, and its owner is the actual new face. The program reports pointer equality as booleans, never addresses.

This is a success-path native oracle for the trimming operation itself. It implements no Rust successor and establishes no early-error destruction guarantee. Existing face-constructor tests exercise a different operation.

The companion `original_g1_trim_null.cpp` changes only the region argument to null, exercising the defined early error. Compile with the same command, replacing the source and output names. Measured status: 1006, zero faces, and five destruction callbacks: all four original curves followed by the original surface. The observer forwards every event and prior return status, leaves the system callback intact and restores the previous user callback before cleanup. No destroyed pointer is dereferenced. This establishes consumed inputs for this early error; it does not demonstrate a valid fillet reaching a non-G1 failure.
