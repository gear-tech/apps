use gstd::BTreeSet;
use gtest::{Program, System};
use supply_chain_io::InitSupplyChain;

#[test]
fn it_works() {
    let system = System::new();
    system.init_logger();
    let supply_chain = Program::current(&system);
    let result = supply_chain.send(
        0,
        InitSupplyChain {
            producers: BTreeSet::new(),
            distributors: BTreeSet::new(),
            retailers: BTreeSet::new(),
        },
    );

    assert!(!result.main_failed());
}
