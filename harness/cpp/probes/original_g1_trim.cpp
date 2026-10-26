// Standalone native oracle; no production implementation or reference rebuild.
#include <cstdio>
#include <SmBrep.h>
#include <SmPlane.h>
#include <SmLine.h>
#include <SmFace.h>
#include <SmTrimmingTools.h>
int main() {
  SmContext context;
  SmBrep* body = new (context) SmBrep();
  SmSurface* original = new (context) SmPlane(SmPoint3d(0,0,0), SmVector3d(0,0,1), &context);
  const SmPoint2d uv[4] = {SmPoint2d(0,0),SmPoint2d(1,0),SmPoint2d(1,1),SmPoint2d(0,1)};
  SmPoint3d points[4];
  for(int i=0;i<4;++i) if(original->EvaluatePoint(uv[i],points[i])!=SM_SUCCESS) return 2;
  SmTArray<SmCurve*> curves; SmTArray<ULONG> counts; SmTArray<SmOrientType> orientations;
  SmTArray<SmPoint3d> loop_points; SmTArray<SmFace*> made;
  for(int i=0;i<4;++i) {
    SmLine* line=NULL; SmExtent1d interval(0,1);
    if(SmLine::CreateLineSegment(context,3,points[i],points[(i+1)%4],line,&interval)!=SM_SUCCESS) return 3;
    curves.Add(line); orientations.Add(SM_OT_SAME);
  }
  counts.Add(4);
  SmExtent2d domain(SmPoint2d(0,0),SmPoint2d(1,1));
  SmStatus status=SmTrimmingTools::TrimSurfaceWithModelSpaceCurves(context,body->GetInfiniteRegion(),1e-5,
    counts,curves,orientations,loop_points,original,domain,SM_OT_SAME,TRUE,FALSE,FALSE,&made);
  SmTArray<SmVertex*> vertices; SmTArray<SmEdge*> edges; SmTArray<SmFace*> faces;
  body->GetVertices(vertices);body->GetEdges(edges);body->GetFaces(faces);
  const bool installed=made.GetSize()==1 && made[0]->GetSurface()==original;
  const bool owner=installed && original->GetOwner()==made[0];
  std::printf("{\"status\":%d,\"vertices\":%lu,\"edges\":%lu,\"faces\":%lu,\"made_faces\":%lu,\"tolerance\":%.17g,\"original_surface_installed\":%s,\"owner_is_actual_new_face\":%s}\n",
    int(status),(unsigned long)vertices.GetSize(),(unsigned long)edges.GetSize(),(unsigned long)faces.GetSize(),
    (unsigned long)made.GetSize(),double(body->GetTolerance()),installed?"true":"false",owner?"true":"false");
  delete body;
  return status==SM_SUCCESS && installed && owner ? 0:1;
}
