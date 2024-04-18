//! A constraint checker that allows kitties to be minted. Anyone can mint a kitty out of thin air
//! for a flat fee. Each minted kitty is guaranteed to have unique DNA (assuming Blake2 is collision resistant).

use super::*;
use money::Coin;

/// The Lord said, "Let their be kitties to frolic upon the chain."
/// "Let each kitty be unique with its own unique DNA."
/// The Lord endowed his servant thusly, "Let the plebs create kitties of their own accord,
/// such that each new kitty's DNA be the hash of a sequential nonce."
/// "But let any pleb who creates a kitty with arbitrary DNA be banished from the chain."
/// And the Lord saw that there were kitties and it was good.
/// There was morning, and there was evening, and there was frolicking. The Eighth day.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Encode, Decode, Debug, TypeInfo)]
pub struct UniversalKittyCreator {
    next_nonce: u32,
}

impl UtxoData for UniversalKittyCreator {
    const TYPE_ID: [u8; 4] = *b"ctcr";
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Encode, Decode, Debug, TypeInfo)]
pub enum KittyMintingError {
    BadlyTyped,
    InsufficientFee,
    UniversalCreatorNotSupplied,
    UniversalCreatorNotUpdatedCorrectly,
    TooManyOutputs,
    MintedKittyInvalid,
}

/// The fee to mint a kitty. For serious use this should be in a config trait or in storage.
const MINT_FEE: u128 = 10;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Encode, Decode, Debug, TypeInfo)]
/// A constraint checker that allows minting a kitty for a fixed (hard-coded) fee.
///
/// Inputs:
/// * One or more coins whose value equals or exceeds the fee
///
/// Evicted Inputs:
/// * The universal creator
///
/// Outputs:
/// * The universal creator, in the first position
/// * The new kitty in the second position
/// * TODO Change coins
pub struct MintKitty;

impl SimpleConstraintChecker for MintKitty {
    type Error = KittyMintingError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        evicted_input_data: &[DynamicallyTypedData],
        _peek_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        // Ensure the fee is supplied.
        // Currently no change is given, so the rest can be used for priority.
        let mut total_input_value: u128 = 0;

        for input in input_data {
            let utxo_value = input
                .extract::<Coin<0>>()
                .map_err(|_| KittyMintingError::BadlyTyped)?
                .0;
            total_input_value += utxo_value;
        }
        ensure!(
            total_input_value >= MINT_FEE,
            KittyMintingError::InsufficientFee
        );
        let priority = (total_input_value - MINT_FEE) as u64;

        // Ensure the Creator is evicted, updated, and output properly.
        ensure!(
            evicted_input_data.len() == 1,
            KittyMintingError::UniversalCreatorNotSupplied
        );
        let input_creator = evicted_input_data[0]
            .extract::<UniversalKittyCreator>()
            .map_err(|_| KittyMintingError::BadlyTyped)?;

        ensure!(
            output_data.len() > 0,
            KittyMintingError::UniversalCreatorNotUpdatedCorrectly
        );
        let output_creator = output_data[0]
            .extract::<UniversalKittyCreator>()
            .map_err(|_| KittyMintingError::BadlyTyped)?;

        ensure!(
            output_creator.next_nonce == input_creator.next_nonce + 1,
            KittyMintingError::UniversalCreatorNotUpdatedCorrectly
        );

        // Ensure the new kitty was created properly.
        ensure!(output_data.len() > 1, KittyMintingError::MintedKittyInvalid);
        let minted_kitty = output_data[1]
            .extract::<KittyData>()
            .map_err(|_| KittyMintingError::BadlyTyped)?;

        ensure!(
            minted_kitty.dna == KittyDNA(BlakeTwo256::hash_of(&input_creator.next_nonce)),
            KittyMintingError::MintedKittyInvalid
        );

        // TODO You may want to assert other things about the kitty that was created here.
        // For example its gender and numbers of breedings.

        // Ensure no extra outputs.
        // TODO ideally we would allow change coins at this point.
        ensure!(output_data.len() == 2, KittyMintingError::TooManyOutputs);

        // Ensure the new kitty (and nothing)
        Ok(priority)
    }
}
