// Standalone native oracle; no production implementation or reference rebuild.
#include <cstdio>
#include <SmBrep.h>
#include <SmPlane.h>
#include <SmLine.h>
#include <SmFace.h>
#include <SmTrimmingTools.h>
struct Trace : SmNotifyCallback {
  SmNotifyCallback* prior; SmObject* surface; SmObject* lines[4]; int surface_destroyed=0,curve_destroyed=0,events=0;
  Trace(SmNotifyCallback* p,SmSurface* s,const SmTArray<SmCurve*>& c):prior(p),surface(s) {for(int i=0;i<4;++i)lines[i]=c[i];}
  SmStatus Execute(SmNotifyOperation op,SmObject* caller,SmObject* a,SmObject* b,SmObject* c) override {
    ++events;
    if(op==SM_NO_DESTRUCTION) {
      if(caller==surface) {++surface_destroyed;surface=NULL;std::puts("{\"event\":\"destroy_original_surface\"}");}
      for(int i=0;i<4;++i)if(caller==lines[i]) {++curve_destroyed;lines[i]=NULL;std::printf("{\"event\":\"destroy_original_curve\",\"index\":%d}\n",i);}
    }
    return prior?prior->Execute(op,caller,a,b,c):SM_SUCCESS;
  }
};
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
  Trace trace(context.GetUserNotifyCallback(),original,curves);
  context.SetUserNotifyCallback(&trace);
  SmStatus status=SmTrimmingTools::TrimSurfaceWithModelSpaceCurves(context,(SmRegion*)NULL,1e-5,
    counts,curves,orientations,loop_points,original,domain,SM_OT_SAME,TRUE,FALSE,FALSE,&made);
  context.SetUserNotifyCallback(trace.prior);
  // Original pointer values are never dereferenced or compared after the operation.
  original=NULL;
  SmTArray<SmFace*> faces;body->GetFaces(faces);
  std::printf("{\"status\":%d,\"faces\":%lu,\"made_faces\":%lu,\"original_surface_destroyed\":%d,\"original_curves_destroyed\":%d,\"observed_events\":%d}\n",
    int(status),(unsigned long)faces.GetSize(),(unsigned long)made.GetSize(),trace.surface_destroyed,trace.curve_destroyed,trace.events);
  delete body;
  return status==SM_ERR_NULL_POINTER && trace.surface_destroyed==1 && trace.curve_destroyed==4 && faces.GetSize()==0 && made.GetSize()==0 ? 0:1;
}
