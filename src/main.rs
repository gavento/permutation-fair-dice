use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
};

use fairdice::{is_sorted, FDTS};
use itertools::Itertools;
use log::info;
use log::LevelFilter;
use structopt::StructOpt;

fn load_or_compute(sizes: &[usize], up_to: usize, dir: &PathBuf) -> Result<FDTS, Box<dyn Error>> {
    assert!(!sizes.is_empty());
    assert!(up_to <= sizes.len());
    if sizes.len() == 1 {
        assert_eq!(up_to, 1);
        return Ok(FDTS::new_single(sizes[0]));
    }
    let pf: PathBuf = format!("fdts_{}_fair{}.json", sizes.iter().format("_"), up_to).into();
    let ps: PathBuf = [dir, &pf].iter().collect();
    if Path::new(&ps).exists() {
        let r = File::open(&ps)?;
        let f = FDTS::from_json(&r)?;
        assert_eq!(up_to, f.fair_up_to);
        assert_eq!(sizes, f.sizes);
        info!(
            "# Read FDTS {} (fair up to {}, {} dice tuples) from {:?}",
            f.sizes_string(),
            f.fair_up_to,
            f.dice.len(),
            &ps
        );
        return Ok(f);
    }

    let n = sizes.len();
    let up_to_2 = std::cmp::min(up_to, n - 1);

    fn sizes_and_mapped_positions(sizes: &[usize], position: usize) -> (Vec<usize>, Vec<isize>) {
        let mut a_sizes: Vec<usize> = sizes.into();
        a_sizes.remove(position);
        let mut a_positions: Vec<isize> = (0..(sizes.len() - 1) as isize).collect();
        a_positions.insert(position, -1);
        (a_sizes, a_positions)
    }

    info!(
        "# Gathering data for FDTS [{}] (fair up to {}) ...",
        sizes.iter().format(","),
        up_to
    );
    let (a_s, a_p) = sizes_and_mapped_positions(sizes, n - 2);
    let da = load_or_compute(&a_s, up_to_2, dir)?;
    let (b_s, b_p) = sizes_and_mapped_positions(sizes, n - 1);
    let db = load_or_compute(&b_s, up_to_2, dir)?;

    let mut checking = vec![];
    for i in 0..(n - 2) {
        let (c_s, c_p) = sizes_and_mapped_positions(sizes, i);
        let dc = load_or_compute(&c_s, up_to_2, dir)?;
        checking.push((dc, c_p));
    }

    let f = FDTS::new_combined(
        da.mapped_as(&a_p),
        db.mapped_as(&b_p),
        checking.iter().map(|(c, p)| c.mapped_as(p)).collect_vec().as_slice(),
        up_to,
    );
    let mut w = File::create(&ps)?;
    f.write_json(&mut w)?;
    info!(
        "# Saved FDTS [{}] (fair up to {}, {} dice tuples) to {:?}",
        f.sizes_string(),
        f.fair_up_to,
        f.dice.len(),
        ps
    );
    Ok(f)
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Limit permutation fairness to all k-tuples (default: all)
    #[structopt(short, long, default_value = "-1")]
    fair_up_to: isize,

    /// Verbose mode
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Output file
    #[structopt(short, long, parse(from_os_str), default_value = "fdts_data")]
    output_dir: PathBuf,

    /// Sizes to process
    #[structopt(name = "SIZE")]
    sizes: Vec<usize>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut opt = Opt::from_args();
    if opt.fair_up_to < 0 {
        opt.fair_up_to = opt.sizes.len() as isize;
    }
    simple_logging::log_to_stderr(if opt.verbose == 0 { LevelFilter::Info } else { LevelFilter::Debug });
    if !opt.output_dir.exists() {
        info!("Creating new dir {:?}", &opt.output_dir);
        std::fs::create_dir_all(&opt.output_dir)?;
    }
    assert!(!opt.sizes.is_empty(), "Needs at least one SIZE");
    assert!(is_sorted(&opt.sizes), "Sizes need to be non-descending in size");
    load_or_compute(&opt.sizes, opt.fair_up_to as usize, &opt.output_dir)?;
    Ok(())
}
