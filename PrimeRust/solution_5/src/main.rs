//! # Rust solution 5 by Kulasko
//!
//! As it's written in the readme, this solution focuses in different multithreading algorithms.
//! There are currently three algorithms, each run with a bit vector and a bool vector.
//!
//! The algorithms are implemented as a trait specialisation of the corresponding data struct. They
//! are a bit unwieldy because of the verbose instantiation, this could be improved by taking the
//! constructor out of the trait.

#[warn(missing_docs)]
mod integer;
mod sieve;

pub use integer::Integer;

use sieve::flag_data::FlagData;
use sieve::{algorithm, flag_data, Algorithm, Sieve, SieveExecute};

use std::time::{Duration, Instant};
use structopt::StructOpt;

/// Most things are hardcoded. Performs one bench for each combination of algorithm and data
/// structure.
pub fn main() {
    let arguments = Arguments::from_args();

    eprintln!("Starting benchmark");
    eprintln!("Working set size is {} kB", arguments.set_size);
    perform_bench::<Sieve<algorithm::Serial, FlagData<flag_data::Bool, u8>, u8>, algorithm::Serial>(
        algorithm::Serial,
        arguments.sieve_size,
        arguments.duration,
    );
    perform_bench::<Sieve<algorithm::Serial, FlagData<flag_data::Bit, u32>, u32>, algorithm::Serial>(
        algorithm::Serial,
        arguments.sieve_size,
        arguments.duration,
    );
    perform_bench::<Sieve<algorithm::Stream, FlagData<flag_data::Bool, u8>, u8>, algorithm::Stream>(
        algorithm::Stream,
        arguments.sieve_size,
        arguments.duration,
    );
    perform_bench::<Sieve<algorithm::Stream, FlagData<flag_data::Bit, u8>, u8>, algorithm::Stream>(
        algorithm::Stream,
        arguments.sieve_size,
        arguments.duration,
    );
    perform_bench::<Sieve<algorithm::Stream, FlagData<flag_data::Bit, u32>, u32>, algorithm::Stream>(
        algorithm::Stream,
        arguments.sieve_size,
        arguments.duration,
    );
    perform_bench::<Sieve<algorithm::Tile, FlagData<flag_data::Bool, u8>, u8>, algorithm::Tile>(
        algorithm::Tile(arguments.set_size * 1024),
        arguments.sieve_size,
        arguments.duration,
    );
    perform_bench::<Sieve<algorithm::Tile, FlagData<flag_data::Bit, u8>, u8>, algorithm::Tile>(
        algorithm::Tile(arguments.set_size * 1024),
        arguments.sieve_size,
        arguments.duration,
    );
    perform_bench::<Sieve<algorithm::Tile, FlagData<flag_data::Bit, u32>, u32>, algorithm::Tile>(
        algorithm::Tile(arguments.set_size * 1024),
        arguments.sieve_size,
        arguments.duration,
    );
}

/// Executes a specific bench and prints the result.
fn perform_bench<S: SieveExecute<A>, A: Algorithm>(
    algorithm: A,
    sieve_size: usize,
    duration: usize,
) {
    let mut passes = 0;
    let mut last_sieve = None;
    let mut elapsed = Duration::from_secs(0);
    let id_string = format!("{}-{}-u{}", A::ID_STR, S::ID_STR, S::BITS);

    eprintln!();
    eprintln!(
        "Running {} with {} primes for {} seconds",
        id_string, sieve_size, duration
    );

    let start = Instant::now();

    while elapsed < Duration::from_secs(duration as u64) {
        let mut sieve = S::new(sieve_size, algorithm);
        sieve.sieve();

        last_sieve.replace(sieve);
        passes += 1;
        elapsed = Instant::now() - start;
    }

    let sieve = last_sieve.expect("Used a duration of zero!");
    let result = sieve.count_primes();

    eprintln!(
        "Time: {}, Passes: {}, Per second: {}, Average time: {}, Threads: {}, Prime count: {}",
        elapsed.as_secs_f64(),
        passes,
        passes as f64 / elapsed.as_secs_f64(),
        elapsed.as_secs_f64() / passes as f64,
        sieve.thread_count(),
        result
    );
    if let Ok(index) = PRIMES_IN_SIEVE.binary_search_by_key(&sieve_size, |(key, _)| *key) {
        if PRIMES_IN_SIEVE[index].1 == result {
            eprintln!("This result is verified to be correct");
        } else {
            eprintln!("ERROR: Incorrect sieve result!");
        }
    }

    println!(
        "kulasko-rust-{};{};{};{};algorithm=base,faithful=yes,bits={}",
        id_string,
        passes,
        elapsed.as_secs_f64(),
        sieve.thread_count(),
        S::FLAG_SIZE
    );
}

/// Contains the arguments of the program.
#[derive(Debug, StructOpt)]
#[structopt(name = "kulasko-rust")]
struct Arguments {
    /// The amount of numbers in a sieve.
    #[structopt(short, long, default_value = "1000000")]
    sieve_size: usize,
    /// The test duration in seconds.
    #[structopt(short, long, default_value = "5")]
    duration: usize,
    /// The size of the working set in kibibytes. Is used by the tiling algorithm. Should not
    /// exceed your memory layer of choice.
    #[structopt(
        short,
        long,
        help = "The working set size in kibibytes",
        default_value = "16"
    )]
    set_size: usize,
}

/// Known prime counts for specific sieve sizes.
const PRIMES_IN_SIEVE: [(usize, usize); 11] = [
    (2, 1),
    (3, 2),
    (4, 2),
    (10, 4),
    (100, 25),
    (1000, 168),
    (10000, 1229),
    (100000, 9592),
    (1000000, 78498),
    (10000000, 664579),
    (100000000, 5761455),
];

#[cfg(test)]
mod test {
    use crate::sieve::flag_data::FlagData;
    use crate::sieve::{algorithm, flag_data, Algorithm, Sieve, SieveExecute};
    use crate::PRIMES_IN_SIEVE;

    /// Generic performing function to reduce code redundancy.
    fn run_test<S: SieveExecute<A>, A: Algorithm>(algorithm: A) {
        for (numbers, primes) in PRIMES_IN_SIEVE {
            let mut sieve = S::new(numbers, algorithm);
            sieve.sieve();
            assert_eq!(
                sieve.count_primes(),
                primes,
                "Numbers {}, expected {}",
                numbers,
                primes
            );
        }
    }

    #[test]
    fn serial_bool_u8() {
        run_test::<Sieve<algorithm::Serial, FlagData<flag_data::Bool, u8>, u8>, algorithm::Serial>(
            algorithm::Serial,
        );
    }

    #[test]
    fn serial_bool_u32() {
        run_test::<Sieve<algorithm::Serial, FlagData<flag_data::Bool, u32>, u32>, algorithm::Serial>(
            algorithm::Serial,
        );
    }

    #[test]
    fn serial_bit_u8() {
        run_test::<Sieve<algorithm::Serial, FlagData<flag_data::Bit, u8>, u8>, algorithm::Serial>(
            algorithm::Serial,
        );
    }

    #[test]
    fn serial_bit_u32() {
        run_test::<Sieve<algorithm::Serial, FlagData<flag_data::Bit, u32>, u32>, algorithm::Serial>(
            algorithm::Serial,
        );
    }

    #[test]
    fn stream_bool_u8() {
        run_test::<Sieve<algorithm::Stream, FlagData<flag_data::Bool, u8>, u8>, algorithm::Stream>(
            algorithm::Stream,
        );
    }

    #[test]
    fn stream_bool_u32() {
        run_test::<Sieve<algorithm::Stream, FlagData<flag_data::Bool, u32>, u32>, algorithm::Stream>(
            algorithm::Stream,
        );
    }

    #[test]
    fn stream_bit_u8() {
        run_test::<Sieve<algorithm::Stream, FlagData<flag_data::Bit, u8>, u8>, algorithm::Stream>(
            algorithm::Stream,
        );
    }

    #[test]
    fn stream_bit_u32() {
        run_test::<Sieve<algorithm::Stream, FlagData<flag_data::Bit, u32>, u32>, algorithm::Stream>(
            algorithm::Stream,
        );
    }

    #[test]
    fn tile_bool_u8() {
        run_test::<Sieve<algorithm::Tile, FlagData<flag_data::Bool, u8>, u8>, algorithm::Tile>(
            algorithm::Tile(1 << 14),
        );
    }

    #[test]
    fn tile_bool_u32() {
        run_test::<Sieve<algorithm::Tile, FlagData<flag_data::Bool, u32>, u32>, algorithm::Tile>(
            algorithm::Tile(1 << 14),
        );
    }

    #[test]
    fn tile_bit_u8() {
        run_test::<Sieve<algorithm::Tile, FlagData<flag_data::Bit, u8>, u8>, algorithm::Tile>(
            algorithm::Tile(1 << 14),
        );
    }

    #[test]
    fn tile_bit_u32() {
        run_test::<Sieve<algorithm::Tile, FlagData<flag_data::Bit, u32>, u32>, algorithm::Tile>(
            algorithm::Tile(1 << 14),
        );
    }
}
