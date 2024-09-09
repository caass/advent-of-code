pub mod types;

macro_rules! mod_years {
    ($($year:literal),+) => {
        ::paste::paste!(
            $(
                #[path = "" $year "/mod.rs"]
                mod [<_ $year>];
                use [<_ $year>]::PROBLEMS as [<PROBLEMS_ $year>];
            )+

            pub const AOC: crate::types::AdventOfCode = crate::types::AdventOfCode([
                $([<PROBLEMS_ $year>],)+
            ]);
        );
    };
}

mod_years!(2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022, 2023);
