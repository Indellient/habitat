pub mod departure;
pub mod election;
pub mod service;
pub mod service_config;
pub mod service_file;

use crate::btest;

#[test]
fn three_members_meshed_rumors_time_out() {
    let mut net = btest::SwimNet::new(3);
    net.mesh();
    // need a way to parameterize the timeout of rumors when creating them so I can make them
    // artificially short
    net.add_service(0, "core/lol/1.2.3/20190422121212");
    net.wait_for_gossip_rounds(2);
    assert!(net[1].service_store
                  .contains_rumor("witcher.prod", net[0].member_id()));
}
