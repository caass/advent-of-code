use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use deranged::RangedUsize;
use eyre::{Report, Result, bail, eyre};
use tabled::settings::Style;
use winnow::ascii::alpha1;
use winnow::combinator::{alt, preceded, separated, terminated};
use winnow::error::ContextError;
use winnow::prelude::*;

use aoc_meta::Problem;

use self::element::Element;

pub const RADIOISOTOPE_THERMOELECTRIC_GENERATORS: Problem =
    Problem::partially_solved(&minimum_steps);

fn minimum_steps(input: &str) -> Result<usize> {
    let col: Column = input.parse()?;
    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    Microchip { element: Element },
    Generator { element: Element },
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Item::Microchip { element } => write!(f, "{element}M"),
            Item::Generator { element } => write!(f, "{element}G"),
        }
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Item::Microchip { element: this }, Item::Microchip { element: that })
            | (Item::Generator { element: this }, Item::Generator { element: that }) => {
                this.cmp(that)
            }
            (Item::Microchip { element: this }, Item::Generator { element: that }) => {
                this.cmp(that).then(Ordering::Less)
            }
            (Item::Generator { element: this }, Item::Microchip { element: that }) => {
                this.cmp(that).then(Ordering::Greater)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Column {
    floors: [Vec<Item>; 4],
    elevator: RangedUsize<0, 3>,
}

impl Column {
    const fn new() -> Column {
        Self {
            floors: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            elevator: RangedUsize::new_static::<0>(),
        }
    }

    fn len(&self) -> usize {
        self.floors.iter().map(Vec::len).sum()
    }

    fn items(&self) -> impl Iterator<Item = Item> {
        self.floors.iter().flatten().copied()
    }
}

impl Default for Column {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Column {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut items: Vec<_> = self.items().collect();
        items.sort();

        let mut floors = vec![vec![".".to_string(); self.len() + 2]; 4];

        for (i, floor) in floors.iter_mut().rev().enumerate() {
            floor[0] = format!("F{}", i + 1);
        }

        floors[3 - self.elevator.get()][1] = "E".to_string();

        for (i, floor) in self.floors.iter().rev().enumerate() {
            for item in floor.iter().copied() {
                floors[i][items.iter().position(|it| *it == item).unwrap() + 2] = item.to_string()
            }
        }

        let mut t = tabled::Table::from_iter(floors);

        Display::fmt(t.with(Style::empty()), f)
    }
}

impl FromStr for Column {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut col = Column::default();

        for (i, line) in s.lines().enumerate() {
            if i > 3 {
                bail!("too many floors ({}) in column!", i + 1);
            }

            let Some((prefix, contains)) = line.split_once(" contains ") else {
                bail!("expected to find string \"contains\" in {line}")
            };

            match (i, prefix) {
                (0, "The first floor")
                | (1, "The second floor")
                | (2, "The third floor")
                | (3, "The fourth floor") => Ok(()),
                _ => Err(eyre!("unexpected prefix \"{prefix}\" on floor {i}")),
            }?;

            col.floors[i] = parse_contains(contains)?;
        }

        Ok(col)
    }
}

fn parse_contains(input: &str) -> eyre::Result<Vec<Item>> {
    terminated(
        alt((
            "nothing relevant".map(|_| Vec::new()),
            preceded(
                "a ",
                separated(
                    1..,
                    alt((
                        terminated(
                            alpha1::<_, ContextError>.parse_to(),
                            "-compatible microchip",
                        )
                        .map(|elem| Item::Microchip { element: elem }),
                        terminated(alpha1.parse_to(), " generator")
                            .map(|elem| Item::Generator { element: elem }),
                    )),
                    alt((", and a ", ", a ", " and a ")),
                ),
            ),
        )),
        '.',
    )
    .parse(input)
    .map_err(|e| eyre!("{e:#?}"))
}

mod element {
    use std::fmt::{self, Display, Formatter};
    use std::str::FromStr;

    use eyre::{Report, Result, eyre};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub(super) enum Element {
        Hydrogen,
        Helium,
        Lithium,
        Beryllium,
        Boron,
        Carbon,
        Nitrogen,
        Oxygen,
        Fluorine,
        Neon,
        Sodium,
        Magnesium,
        Aluminium,
        Silicon,
        Phosphorus,
        Sulfur,
        Chlorine,
        Argon,
        Potassium,
        Calcium,
        Scandium,
        Titanium,
        Vanadium,
        Chromium,
        Manganese,
        Iron,
        Cobalt,
        Nickel,
        Copper,
        Zinc,
        Gallium,
        Germanium,
        Arsenic,
        Selenium,
        Bromine,
        Krypton,
        Rubidium,
        Strontium,
        Yttrium,
        Zirconium,
        Niobium,
        Molybdenum,
        Technetium,
        Ruthenium,
        Rhodium,
        Palladium,
        Silver,
        Cadmium,
        Indium,
        Tin,
        Antimony,
        Tellurium,
        Iodine,
        Xenon,
        Cesium,
        Barium,
        Lanthanum,
        Cerium,
        Praseodymium,
        Neodymium,
        Promethium,
        Samarium,
        Europium,
        Gadolinium,
        Terbium,
        Dysprosium,
        Holmium,
        Erbium,
        Thulium,
        Ytterbium,
        Lutetium,
        Hafnium,
        Tantalum,
        Tungsten,
        Rhenium,
        Osmium,
        Iridium,
        Platinum,
        Gold,
        Mercury,
        Thallium,
        Lead,
        Bismuth,
        Polonium,
        Astatine,
        Radon,
        Francium,
        Radium,
        Actinium,
        Thorium,
        Protactinium,
        Uranium,
        Neptunium,
        Plutonium,
        Americium,
        Curium,
        Berkelium,
        Californium,
        Einsteinium,
        Fermium,
        Mendelevium,
        Nobelium,
        Lawrencium,
        Rutherfordium,
        Dubnium,
        Seaborgium,
        Bohrium,
        Hassium,
        Meitnerium,
        Darmstadtium,
        Roentgenium,
        Copernicium,
        Nihonium,
        Flerovium,
        Moscovium,
        Livermorium,
        Tennessine,
        Oganesson,
    }

    impl FromStr for Element {
        type Err = Report;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "hydrogen" => Ok(Element::Hydrogen),
                "helium" => Ok(Element::Helium),
                "lithium" => Ok(Element::Lithium),
                "beryllium" => Ok(Element::Beryllium),
                "boron" => Ok(Element::Boron),
                "carbon" => Ok(Element::Carbon),
                "nitrogen" => Ok(Element::Nitrogen),
                "oxygen" => Ok(Element::Oxygen),
                "fluorine" => Ok(Element::Fluorine),
                "neon" => Ok(Element::Neon),
                "sodium" => Ok(Element::Sodium),
                "magnesium" => Ok(Element::Magnesium),
                "aluminium" => Ok(Element::Aluminium),
                "silicon" => Ok(Element::Silicon),
                "phosphorus" => Ok(Element::Phosphorus),
                "sulfur" => Ok(Element::Sulfur),
                "chlorine" => Ok(Element::Chlorine),
                "argon" => Ok(Element::Argon),
                "potassium" => Ok(Element::Potassium),
                "calcium" => Ok(Element::Calcium),
                "scandium" => Ok(Element::Scandium),
                "titanium" => Ok(Element::Titanium),
                "vanadium" => Ok(Element::Vanadium),
                "chromium" => Ok(Element::Chromium),
                "manganese" => Ok(Element::Manganese),
                "iron" => Ok(Element::Iron),
                "cobalt" => Ok(Element::Cobalt),
                "nickel" => Ok(Element::Nickel),
                "copper" => Ok(Element::Copper),
                "zinc" => Ok(Element::Zinc),
                "gallium" => Ok(Element::Gallium),
                "germanium" => Ok(Element::Germanium),
                "arsenic" => Ok(Element::Arsenic),
                "selenium" => Ok(Element::Selenium),
                "bromine" => Ok(Element::Bromine),
                "krypton" => Ok(Element::Krypton),
                "rubidium" => Ok(Element::Rubidium),
                "strontium" => Ok(Element::Strontium),
                "yttrium" => Ok(Element::Yttrium),
                "zirconium" => Ok(Element::Zirconium),
                "niobium" => Ok(Element::Niobium),
                "molybdenum" => Ok(Element::Molybdenum),
                "technetium" => Ok(Element::Technetium),
                "ruthenium" => Ok(Element::Ruthenium),
                "rhodium" => Ok(Element::Rhodium),
                "palladium" => Ok(Element::Palladium),
                "silver" => Ok(Element::Silver),
                "cadmium" => Ok(Element::Cadmium),
                "indium" => Ok(Element::Indium),
                "tin" => Ok(Element::Tin),
                "antimony" => Ok(Element::Antimony),
                "tellurium" => Ok(Element::Tellurium),
                "iodine" => Ok(Element::Iodine),
                "xenon" => Ok(Element::Xenon),
                "cesium" => Ok(Element::Cesium),
                "barium" => Ok(Element::Barium),
                "lanthanum" => Ok(Element::Lanthanum),
                "cerium" => Ok(Element::Cerium),
                "praseodymium" => Ok(Element::Praseodymium),
                "neodymium" => Ok(Element::Neodymium),
                "promethium" => Ok(Element::Promethium),
                "samarium" => Ok(Element::Samarium),
                "europium" => Ok(Element::Europium),
                "gadolinium" => Ok(Element::Gadolinium),
                "terbium" => Ok(Element::Terbium),
                "dysprosium" => Ok(Element::Dysprosium),
                "holmium" => Ok(Element::Holmium),
                "erbium" => Ok(Element::Erbium),
                "thulium" => Ok(Element::Thulium),
                "ytterbium" => Ok(Element::Ytterbium),
                "lutetium" => Ok(Element::Lutetium),
                "hafnium" => Ok(Element::Hafnium),
                "tantalum" => Ok(Element::Tantalum),
                "tungsten" => Ok(Element::Tungsten),
                "rhenium" => Ok(Element::Rhenium),
                "osmium" => Ok(Element::Osmium),
                "iridium" => Ok(Element::Iridium),
                "platinum" => Ok(Element::Platinum),
                "gold" => Ok(Element::Gold),
                "mercury" => Ok(Element::Mercury),
                "thallium" => Ok(Element::Thallium),
                "lead" => Ok(Element::Lead),
                "bismuth" => Ok(Element::Bismuth),
                "polonium" => Ok(Element::Polonium),
                "astatine" => Ok(Element::Astatine),
                "radon" => Ok(Element::Radon),
                "francium" => Ok(Element::Francium),
                "radium" => Ok(Element::Radium),
                "actinium" => Ok(Element::Actinium),
                "thorium" => Ok(Element::Thorium),
                "protactinium" => Ok(Element::Protactinium),
                "uranium" => Ok(Element::Uranium),
                "neptunium" => Ok(Element::Neptunium),
                "plutonium" => Ok(Element::Plutonium),
                "americium" => Ok(Element::Americium),
                "curium" => Ok(Element::Curium),
                "berkelium" => Ok(Element::Berkelium),
                "californium" => Ok(Element::Californium),
                "einsteinium" => Ok(Element::Einsteinium),
                "fermium" => Ok(Element::Fermium),
                "mendelevium" => Ok(Element::Mendelevium),
                "nobelium" => Ok(Element::Nobelium),
                "lawrencium" => Ok(Element::Lawrencium),
                "rutherfordium" => Ok(Element::Rutherfordium),
                "dubnium" => Ok(Element::Dubnium),
                "seaborgium" => Ok(Element::Seaborgium),
                "bohrium" => Ok(Element::Bohrium),
                "hassium" => Ok(Element::Hassium),
                "meitnerium" => Ok(Element::Meitnerium),
                "darmstadtium" => Ok(Element::Darmstadtium),
                "roentgenium" => Ok(Element::Roentgenium),
                "copernicium" => Ok(Element::Copernicium),
                "nihonium" => Ok(Element::Nihonium),
                "flerovium" => Ok(Element::Flerovium),
                "moscovium" => Ok(Element::Moscovium),
                "livermorium" => Ok(Element::Livermorium),
                "tennessine" => Ok(Element::Tennessine),
                "oganesson" => Ok(Element::Oganesson),
                _ => Err(eyre!("unknown element: {s}")),
            }
        }
    }

    impl Element {
        fn symbol(&self) -> &'static str {
            match self {
                Element::Hydrogen => "H",
                Element::Helium => "He",
                Element::Lithium => "Li",
                Element::Beryllium => "Be",
                Element::Boron => "B",
                Element::Carbon => "C",
                Element::Nitrogen => "N",
                Element::Oxygen => "O",
                Element::Fluorine => "F",
                Element::Neon => "Ne",
                Element::Sodium => "Na",
                Element::Magnesium => "Mg",
                Element::Aluminium => "Al",
                Element::Silicon => "Si",
                Element::Phosphorus => "P",
                Element::Sulfur => "S",
                Element::Chlorine => "Cl",
                Element::Argon => "Ar",
                Element::Potassium => "K",
                Element::Calcium => "Ca",
                Element::Scandium => "Sc",
                Element::Titanium => "Ti",
                Element::Vanadium => "V",
                Element::Chromium => "Cr",
                Element::Manganese => "Mn",
                Element::Iron => "Fe",
                Element::Cobalt => "Co",
                Element::Nickel => "Ni",
                Element::Copper => "Cu",
                Element::Zinc => "Zn",
                Element::Gallium => "Ga",
                Element::Germanium => "Ge",
                Element::Arsenic => "As",
                Element::Selenium => "Se",
                Element::Bromine => "Br",
                Element::Krypton => "Kr",
                Element::Rubidium => "Rb",
                Element::Strontium => "Sr",
                Element::Yttrium => "Y",
                Element::Zirconium => "Zr",
                Element::Niobium => "Nb",
                Element::Molybdenum => "Mo",
                Element::Technetium => "Tc",
                Element::Ruthenium => "Ru",
                Element::Rhodium => "Rh",
                Element::Palladium => "Pd",
                Element::Silver => "Ag",
                Element::Cadmium => "Cd",
                Element::Indium => "In",
                Element::Tin => "Sn",
                Element::Antimony => "Sb",
                Element::Tellurium => "Te",
                Element::Iodine => "I",
                Element::Xenon => "Xe",
                Element::Cesium => "Cs",
                Element::Barium => "Ba",
                Element::Lanthanum => "La",
                Element::Cerium => "Ce",
                Element::Praseodymium => "Pr",
                Element::Neodymium => "Nd",
                Element::Promethium => "Pm",
                Element::Samarium => "Sm",
                Element::Europium => "Eu",
                Element::Gadolinium => "Gd",
                Element::Terbium => "Tb",
                Element::Dysprosium => "Dy",
                Element::Holmium => "Ho",
                Element::Erbium => "Er",
                Element::Thulium => "Tm",
                Element::Ytterbium => "Yb",
                Element::Lutetium => "Lu",
                Element::Hafnium => "Hf",
                Element::Tantalum => "Ta",
                Element::Tungsten => "W",
                Element::Rhenium => "Re",
                Element::Osmium => "Os",
                Element::Iridium => "Ir",
                Element::Platinum => "Pt",
                Element::Gold => "Au",
                Element::Mercury => "Hg",
                Element::Thallium => "Tl",
                Element::Lead => "Pb",
                Element::Bismuth => "Bi",
                Element::Polonium => "Po",
                Element::Astatine => "At",
                Element::Radon => "Rn",
                Element::Francium => "Fr",
                Element::Radium => "Ra",
                Element::Actinium => "Ac",
                Element::Thorium => "Th",
                Element::Protactinium => "Pa",
                Element::Uranium => "U",
                Element::Neptunium => "Np",
                Element::Plutonium => "Pu",
                Element::Americium => "Am",
                Element::Curium => "Cm",
                Element::Berkelium => "Bk",
                Element::Californium => "Cf",
                Element::Einsteinium => "Es",
                Element::Fermium => "Fm",
                Element::Mendelevium => "Md",
                Element::Nobelium => "No",
                Element::Lawrencium => "Lr",
                Element::Rutherfordium => "Rf",
                Element::Dubnium => "Db",
                Element::Seaborgium => "Sg",
                Element::Bohrium => "Bh",
                Element::Hassium => "Hs",
                Element::Meitnerium => "Mt",
                Element::Darmstadtium => "Ds",
                Element::Roentgenium => "Rg",
                Element::Copernicium => "Cn",
                Element::Nihonium => "Nh",
                Element::Flerovium => "Fl",
                Element::Moscovium => "Mc",
                Element::Livermorium => "Lv",
                Element::Tennessine => "Ts",
                Element::Oganesson => "Og",
            }
        }
    }

    impl Display for Element {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.write_str(self.symbol())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn display() {
        let col = Column {
            elevator: RangedUsize::new_static::<2>(),
            floors: [
                vec![
                    Item::Microchip {
                        element: Element::Cadmium,
                    },
                    Item::Microchip {
                        element: Element::Lead,
                    },
                ],
                vec![
                    Item::Generator {
                        element: Element::Lead,
                    },
                    Item::Microchip {
                        element: Element::Iron,
                    },
                ],
                vec![
                    Item::Microchip {
                        element: Element::Helium,
                    },
                    Item::Generator {
                        element: Element::Helium,
                    },
                ],
                vec![],
            ],
        };

        assert_eq!(
            col.to_string(),
            " F4  .  .    .    .    .    .    .   
 F3  E  HeM  HeG  .    .    .    .   
 F2  .  .    .    FeM  .    .    PbG 
 F1  .  .    .    .    CdM  PbM  .   "
        )
    }

    #[test]
    fn from_str() {
        let input = "The first floor contains a hydrogen-compatible microchip and a lithium-compatible microchip.
The second floor contains a hydrogen generator.
The third floor contains a lithium generator.
The fourth floor contains nothing relevant.";

        let col = input.parse::<Column>().unwrap();
        assert_eq!(
            col,
            Column {
                elevator: RangedUsize::new_static::<0>(),
                floors: [
                    vec![
                        Item::Microchip {
                            element: Element::Hydrogen
                        },
                        Item::Microchip {
                            element: Element::Lithium
                        }
                    ],
                    vec![Item::Generator {
                        element: Element::Hydrogen
                    }],
                    vec![Item::Generator {
                        element: Element::Lithium
                    }],
                    vec![]
                ]
            }
        )
    }
}
