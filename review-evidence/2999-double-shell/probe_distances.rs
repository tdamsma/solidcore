use solidcore::{Kernel,kernel::UpstreamFixes};
use solidcore::geom::attribute::AttributeStore;
use solidcore::topology::{loopuse,topology::NotifyRecords,brep::p10::TopologyEnv};
use solidcore::offset::offset_executive::p2::{shell_brep_simple,OffsetOptions,ShellOptions};
fn main(){let path=std::env::args().nth(1).unwrap();let data=std::fs::read(path).unwrap();for d in [0.25,0.5,0.75]{
 let mut k=Kernel{fixes:UpstreamFixes::all(),..Kernel::default()};let mut attributes=AttributeStore::default();let mut records=NotifyRecords::new();let mut env=TopologyEnv{all_attributes:&[],attributes:&mut attributes,key_of:&loopuse::notify_ref,records:&mut records,next_key:1<<50};
 let b=solidcore::topology::api_files::read_brep_from_file(&mut k,Some(data.clone()),true,false,&mut env).unwrap();let open=b.face_list()[14];
 let result=shell_brep_simple(&mut k,&b,ShellOptions{offset:OffsetOptions{distance:-d,extend_convex_edges:true,merge_results:true},create_offset_solid:true,merge_sheet_side_walls:true},&[open]);
 println!("distance=-{d}: {:?}",result.map(|x|(x.face_list().len(),x.edge_list().len())));
}}
