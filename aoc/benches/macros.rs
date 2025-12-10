#![allow(unused_macros)]

macro_rules! bench {
    ($($year:literal: { $($day:literal: [$( $(#[$attrs:meta])* $part:literal),* $(,)?]),* $(,)? }),* $(,)?) => {
        $(bench_year!($year, [$(($day, [$( ($(#[$attrs])*, $part) ),*])),*]);)*
    };
}

macro_rules! bench_year {
    ($year:literal, [$(($day:literal, [$( ($(#[$attrs:meta])*, $part:literal) ),*])),*]) => {
        ::pastey::paste! {
            mod [<y $year>] {
                $(bench_day!($year, $day, [$( ($(#[$attrs])*, $part) ),*]);)*
            }
        }
    };
}

macro_rules! bench_day {
    ($year:literal, $day:literal, [$( ($(#[$attrs:meta])*, $part:literal) ),*]) => {
        ::pastey::paste! {
            mod [<d $day>] {
                static INPUT: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
                    let two_digit_day = if $day < 10 {
                        concat!("0", stringify!($day))
                    } else {
                        stringify!($day)
                    };

                    let mut path: std::path::PathBuf =
                        env!("CARGO_MANIFEST_DIR").parse().unwrap();
                    path.pop();
                    path.push("target");
                    path.push("inputs");
                    path.push(stringify!($year));
                    path.push(two_digit_day);

                    std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()))
                });

                $(bench_part!($year, $day, $part, [$(#[$attrs])*]);)*
            }
        }
    };
}

macro_rules! bench_part {
    // No attributes - use default #[divan::bench]
    ($year:literal, $day:literal, $part:literal, []) => {
        bench_part!($year, $day, $part, [#[divan::bench]]);
    };

    // With attributes
    ($year:literal, $day:literal, $part:literal, [$(#[$attrs:meta])+]) => {
        ::pastey::paste! {
            $(#[$attrs])+
            fn [<part $part>](bencher: divan::Bencher) {
                let input = INPUT.trim();

                let year = ::aoc_meta::Year::from_u16($year).unwrap();
                let day = ::aoc_meta::Day::from_u8($day).unwrap();
                let part = ::aoc_meta::Part::try_from($part as u8).unwrap();

                let f = &::aoc::AOC[year][day][part];

                bencher.bench(|| f.solve(divan::black_box(input)));
            }
        }
    };
}
