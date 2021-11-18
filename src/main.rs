use fairdice::fdts::FDTS;

fn main() {
    env_logger::init();
    let d6 = FDTS::new_single(12);
    let d6_6 = FDTS::new_combined(d6.mapped_as(&[0, -1]), d6.mapped_as(&[-1, 0]), &[]);
    let d6_6_6 = FDTS::new_combined(
        d6_6.mapped_as(&[0, 1, -1]),
        d6_6.mapped_as(&[0, -1, 1]),
        &[d6_6.mapped_as(&[-1, 0, 1])],
    );
}
