//! transforms CVM program into analyzed format
//!
//! Cases:
//! 1. Centauri -> transfer -> Cosmos Hub => converted to usual IBC transfer
//! 2. Centauri -> transfer -> Cosmos Hub -> transfer -> Osmosis => PFM enabled transfer
//! 3. Centauri -> transfer -> Cosmos Hub(local CVM op) -> transfer -> Osmosis => unroutable
//! 4. Centauri -> transfer -> Cosmos Hub -> transfer -> Osmosis (swap) => PFM enabled transfer with CVM wasm hook
//! 5. Centauri -> transfer -> Cosmos Hub -> transfer -> Osmosis (swap) -> transfer -> Neutron => PFM enabled transfer with CVM wasm hook and after usual transfer
//! 6. Centauri -> transfer -> Cosmos Hub -> transfer -> Osmosis (swap) -> transfer -> Neutron(swap) => PFM enabled transfer with CVM wasm hook and after one more CVM hop
//!
//! Solutions sorted by price and cheapest one selected.
//!
//! I think because this one will produce dead branches(along with heap garbage), should searchers do that offchain and provide hint for execution?
//! It seems under researched.

use cvm_runtime::{
    shared::{Displayed, XcAddr, XcProgram},
    AbsoluteAmount, Amount, AssetId, Instruction, NetworkId,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("salt should be properly set")]
    SaltShouldBeProperlySet,
    #[error("final transfer amount should be absolute")]
    AmountShouldBeAbsolute,
}

/// ensure that all `Spawns`` has specific `salt` sets
pub fn ensure_salt(program: &XcProgram, value: &[u8]) -> Result<(), Error> {
    for ix in program.instructions.iter() {
        match ix {
            Instruction::Spawn { salt, .. } if salt != value => {
                return Err(Error::SaltShouldBeProperlySet)
            }
            Instruction::Spawn { program, .. } => {
                ensure_salt(&program, value)?;
            }
            _ => {}
        }
    }
    return Ok(());
}

pub struct AbsoluteTransfer {
    pub to: XcAddr,
    pub amount: AbsoluteAmount,
}

pub fn ensure_final_transfers_are_absolute(
    program: &XcProgram,
    value: &[u8],
    result: &mut Vec<AbsoluteTransfer>,
) -> Result<(), Error> {
    for ix in program.instructions.iter() {
        match ix {
            Instruction::Transfer { to, assets } => match (to, assets.0.get(0)) {
                (cvm_runtime::Destination::Account(addr), Some((asset_id, amount))) => {
                    let transfer = AbsoluteTransfer {
                        to: addr.clone(),
                        amount: AbsoluteAmount {
                            amount: amount.intercept,
                            asset_id: *asset_id,
                        },
                    };
                }
                _ => {}
            },
            Instruction::Spawn { program, .. } => {
                ensure_final_transfers_are_absolute(&program, value, result)?
            }
            _ => {}
        }
    }
    Ok(())
}

/// returns assets from entry spawn program
pub fn get_desired_assets(program: &XcProgram) -> Vec<AbsoluteAmount> {
    todo!()
}

#[cfg(test)]
mod tests {

    use std::borrow::BorrowMut;

    use crate::{contract::instantiate, contract::query::query, error::ContractError};

    use cosmwasm_std::{testing::*, Addr};
    use cvm_runtime::{
        executor::ExecuteMsg,
        outpost::{HereItem, InstantiateMsg},
        shared::{XcAddr, XcFunds, XcFundsFilter, XcInstruction, XcProgram},
        Amount, Destination, Instruction,
    };

    #[test]
    fn query_no_data() {
        let msg = cvm_runtime::outpost::QueryMsg::GetAssetById {
            asset_id: 42.into(),
        };
        let resp = query(mock_dependencies().as_ref(), mock_env(), msg);
        assert!(matches!(resp, Err(ContractError::AssetNotFound)));
    }

    #[test]
    fn transfer_from_centauri_to_osmosis() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("sender", &[]);
        let msg = cvm_runtime::outpost::InstantiateMsg(HereItem {
            network_id: 2.into(),
            admin: Addr::unchecked("sender"),
        });
        crate::contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let config = [];
        let msg = cvm_runtime::outpost::ExecuteMsg::Config(
            cvm_runtime::outpost::ConfigSubMsg::Force(config.to_vec()),
        );

        crate::contract::execute::execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let transfer = XcInstruction::transfer_absolute_to_account("bob", 2, 100);
        let spawn = Instruction::Spawn {
            network_id: 2.into(),
            salt: <_>::default(),
            assets: XcFundsFilter::one(1.into(), Amount::new(100, 0)),
            program: XcProgram {
                tag: <_>::default(),
                instructions: vec![transfer],
            },
        };
        let program = XcProgram {
            tag: <_>::default(),
            instructions: vec![spawn],
        };

        // crate::contract::query::query(deps.as_ref(), env.clone(), info.clone(), msg).unwrap();
    }

    #[test]
    fn spawns_neutron_hub_osmosis_hub_centauri() {}

    #[test]
    fn atom_on_centauri_atom_on_neutron_via_hub() {}
}
