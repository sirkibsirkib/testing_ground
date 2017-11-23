// use std::collections::HashMap;
use super::{LocationID,CPoint2};
//
// //primitives need to only define their ENTRY POINT
// //the server will resolve this to an exit point
// use std::hash::{Hash,Hasher};
// // impl Hash for f32 {
// //     fn hash<H>(&self, state: &mut H) where H: Hasher {
// //         state.write_u8(4);
// //     }
// // }
#[derive(Debug,Copy,Clone)]
pub struct UniquePoint {
    lid: LocationID,
    c_pt: CPoint2
}
//
// struct PortalResolver {
//     mapping: HashMap<UniquePoint,UniquePoint>,
// }
//
// /*
// client                                          server
//   |                                                |
// write(x)      ==TraversePortal(UniquePoint)==>  let x = read().unique_point
//   |                                             let out = portal_resolver.resolve(x)
// read()      <==LoadLID(out.lid)====             write(1)
// read()    <==ApplyDiff(AddEntity(you,out.cout)) write(2)
//
//
//
// */
//
//
// //make saveable-loadable
// //used only serverside. output to client is ultimately {Load(LID), addEntity(LID,EID)}
// impl PortalResolver {
//     pub fn new() -> PortalResolver {
//         PortalResolver{mapping: HashMap::new()}
//     }
//
//     pub fn resolve(&self, entry_point: UniquePoint) -> Option<UniquePoint> {
//         self.mapping.get(entry_point)
//     }
//
//     pub fn define_portal(&mut self, a: UniquePoint, b:UniquePoint) {
//         self.mapping.insert(a,b);
//     }
// }
