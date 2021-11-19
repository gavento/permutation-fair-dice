use fairdice::fdts::FDTS;
use itertools::Itertools;
use log::info;

fn main() {
    env_logger::init();
    let d6 = FDTS::new_single(6);
    let d12 = FDTS::new_single(12);
    let d6_6 = FDTS::new_combined(d6.mapped_as(&[0, -1]), d6.mapped_as(&[-1, 0]), &[]);
    let d6_6_6 = FDTS::new_combined(
        d6_6.mapped_as(&[0, 1, -1]),
        d6_6.mapped_as(&[0, -1, 1]),
        &[d6_6.mapped_as(&[-1, 0, 1])],
    );
    let d6_12 = FDTS::new_combined(d6.mapped_as(&[0, -1]), d12.mapped_as(&[-1, 0]), &[]);
    let d12_12 = FDTS::new_combined(d12.mapped_as(&[0, -1]), d12.mapped_as(&[-1, 0]), &[]);

    let d6_6_12 = FDTS::new_combined(
        d6_6.mapped_as(&[0, 1, -1]),
        d6_12.mapped_as(&[0, -1, 1]),
        &[d6_12.mapped_as(&[-1, 0, 1])],
    );
    let d6_12_12 = FDTS::new_combined(
        d6_12.mapped_as(&[0, 1, -1]),
        d6_12.mapped_as(&[0, -1, 1]),
        &[d12_12.mapped_as(&[-1, 0, 1])],
    );
    let d6_6_12_12 = FDTS::new_combined(
        d6_6_12.mapped_as(&[0, 1, 2, -1]),
        d6_6_12.mapped_as(&[0, 1, -1, 2]),
        &[d6_12_12.mapped_as(&[-1, 0, 1, 2]), d6_12_12.mapped_as(&[0, -1, 1, 2])],
    );
    info!("Found 6,6,12,12: {:#?}", d6_6_12_12.dice.iter().map(|x| x.as_string()).collect_vec());

    let d6_12_12_12 = FDTS::new_combined(
        d6_12_12.mapped_as(&[0, 1, 2, -1]),
        d6_12_12.mapped_as(&[0, 1, -1, 2]),
        &[d6_12_12.mapped_as(&[0, -1, 1, 2])],
    );
    return;
    
    let _d12_12_12 = FDTS::new_combined(
        d12_12.mapped_as(&[0, 1, -1]),
        d12_12.mapped_as(&[0, -1, 1]),
        &[d12_12.mapped_as(&[-1, 0, 1])],
    );
}
