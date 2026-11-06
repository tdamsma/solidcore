use solidcore::{Kernel,Brep};
use solidcore::kernel::UpstreamFixes;
use solidcore::geom::attribute::AttributeStore;
use solidcore::topology::{loopuse,integrity,topology::NotifyRecords,brep::p10::TopologyEnv};
use solidcore::query::{edge::{edge_faces,edge_vertices},api_queries::{vertex_get_point,face_bounding_box}};
fn main(){
 for path in std::env::args().skip(1){
 let mut k=Kernel{fixes:UpstreamFixes::all(),..Kernel::default()};let mut attributes=AttributeStore::default();let mut records=NotifyRecords::new();let mut env=TopologyEnv{all_attributes:&[],attributes:&mut attributes,key_of:&loopuse::notify_ref,records:&mut records,next_key:1<<50};
 let mut b:Brep=solidcore::topology::api_files::read_brep_from_file(&mut k,Some(std::fs::read(&path).unwrap()),true,false,&mut env).unwrap();
 println!("FILE {path} faces={} edges={} vertices={} regions={}",b.face_list().len(),b.edge_list().len(),b.vertex_list().len(),b.regions().len());println!("integrity before {:?}",integrity::check(&b));b.refresh_cached_topology();println!("integrity after {:?}",integrity::check(&b));
 for &e in b.edge_list(){if !solidcore::topology::edge::is_manifold(&b,e).unwrap(){let faces=edge_faces(&b,e).unwrap();let pts:Vec<_>=edge_vertices(&b,e).unwrap().into_iter().map(|v|vertex_get_point(&b,v).unwrap()).collect();println!("BAD_EDGE {e:?} faces={faces:?} ends={pts:?}");}}
 for &f in b.face_list(){println!("FACE {f:?} surface={} bounds={:?}",b.face(f).unwrap().surface.as_ref().unwrap().type_code(),face_bounding_box(&k,&b,f,true));}
 }
}
