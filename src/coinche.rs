// use strum::IntoEnumIterator;

// #[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd, EnumIter)]
// pub enum Contract {
//     Pass = 0,
//     _80 = 80,
//     _90 = 90,
//     _100 = 100,
//     _110 = 110,
//     _120 = 120,
//     _130 = 130,
//     _140 = 140,
//     _150 = 150,
//     _160 = 160,
//     _170 = 170,
//     _180 = 180,
//     Capot = 250,
//     CapotBelote = 270,
//     Generale = 500, // one player announce he will get all cards
// }

// #[derive(Eq, PartialEq, Clone, Debug)]
// pub enum Type {
//     Color(Color),
//     SansAtout,
//     ToutAtout,
// }

// #[derive(Eq, PartialEq, Clone, Debug)]
// pub enum Interruption {
//     Coinche,
//     Surcoinche,
// }

// impl fmt::Display for Contract {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             Self::Pass => write!(f, "Passe."),
//             Self::Capot => write!(f, "Capot."),
//             Self::CapotBelote => write!(f, "Capot beloté."),
//             Self::Generale => write!(f, "Generale."),
//             _ => write!(f, "{}", *self as usize),
//         }
//     }
// }

// pub contract: Option<Contract>,
// pub interruption: Option<Interruption>,
// pub team: Option<Team>,

// pub fn bidding(&mut self) -> Result<(), Error> {
//     let mut contracts: Vec<Contract> = Contract::iter().collect();

//     for p in self.players.iter_mut() {
//         if self.auto && contracts.len() == 1 {
//             p.contract = Some(Contract::Pass);
//             println!("Auto pass");
//             continue;
//         }

//         p.contract = if self.random {
//             Some(contracts[rand::thread_rng().gen_range(0..contracts.len())])
//         } else {
//             loop {
//                 println!("{} must play : {}", &p, &p.hand);
//                 println!("Choose a contract, possibilities :");
//                 for (i, c) in contracts.iter().enumerate() {
//                     println!("\t{} : press {}", c, i);
//                 }
//                 let contract_index = read_index();
//                 if contract_index < contracts.len() {
//                     break Some(contracts[contract_index]);
//                 } else {
//                     println!("Error, please retry");
//                 }
//             }
//         };

//         contracts = match p.contract {
//             Some(Contract::Pass) => {
//                 println!("Pass");
//                 p.contract = Some(Contract::Pass);
//                 contracts
//             }
//             Some(contract) => {
//                 println!("Chosen contract: {}", contract);
//                 p.contract = Some(contract);
//                 Contract::iter()
//                     .filter(|other_contract| {
//                         other_contract == &Contract::Pass
//                             || *other_contract as usize > contract as usize
//                     })
//                     .collect()
//             }
//             _ => {
//                 println!("A contract must be available for everyone!");
//                 return Err(BeloteErrorKind::InvalidCase.into());
//             }
//         };
//     }
//     Ok(())
// }
