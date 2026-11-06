use solidcore::{Kernel,kernel::UpstreamFixes};
use solidcore::geom::attribute::AttributeStore;
use solidcore::topology::{loopuse,topology::NotifyRecords,brep::p10::TopologyEnv};
use solidcore::offset::offset_executive::p2::{shell_brep_simple,OffsetOptions,ShellOptions};
fn main(){let path=std::env::args().nth(1).unwrap();let data=std::fs::read(path).unwrap();for face_index in [0,1,2,3,4,5,11,12,14,16,18]{ let d = 0.5;
 let mut k=Kernel{fixes:UpstreamFixes::all(),..Kernel::default()};let mut attributes=AttributeStore::default();let mut records=NotifyRecords::new();let mut env=TopologyEnv{all_attributes:&[],attributes:&mut attributes,key_of:&loopuse::notify_ref,records:&mut records,next_key:1<<50};
 let b=solidcore::topology::api_files::read_brep_from_file(&mut k,Some(data.clone()),true,false,&mut env).unwrap();let open=b.face_list()[face_index];
 let result=shell_brep_simple(&mut k,&b,ShellOptions{offset:OffsetOptions{distance:-d,extend_convex_edges:true,merge_results:true},create_offset_solid:true,merge_sheet_side_walls:true},&[open]);
 println!("face={face_index} distance=-{d}: {:?}",result.map(|mut x|(x.face_list().len(),x.edge_list().len(),solidcore::topology::brep::p2::is_manifold_solid(&x),solidcore::mass::integrator::properties::brep_volume_default(&k,&mut x,1e-8))));
}}
