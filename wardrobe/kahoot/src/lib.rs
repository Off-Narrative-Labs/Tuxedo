//! On-chain clone of Kahoot!, a fun and popular learning game among classroom teachers
//! and students alike. See kahoot.it for more info on the original
//! 
//! The original Kahoot is closed source and hosted on a centralized web2 service. This
//! has many costs and risks that won't eb recounted here. This Tuxedo piece brings a
//! kahoot-like game on-chain. for anyone to use.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;
use tuxedo_core::{
    dynamic_typing::{DynamicallyTypedData, UtxoData},
    ensure, SimpleConstraintChecker,
};
use sp_core::H256;

/// The minimum acceptable deposit / ante amount for a game instance.
/// This should be moved to a config trait.
pub const MIN_DEPOSIT: u128 = 20;

// #[cfg(test)]
// mod tests;

/// An individual instance of a game. Created by a teacher, peeked at several times through
/// its life cycle, and ultimately consumed when the game is settled at the end.
/// 
/// At the moment, I'm putting a lot of info straight into the UTXOs. This may need to be
/// revisited and replaced with IPFS pointers or something.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct GameDetails {
    //TODO I think we need some kind of game id.
    // We need to make sure that all the downstream types like answer ticket and question commitment are
    // associated with the correct game and cannot cross-pollinate.
    // It needs to be unique like based on the transaction hash or something.

    /// Name of the game. Suggested 25 char max.
    pub name: Vec<u8>,
    /// Description of what content will be covered in this game and information that may entice users to play.
    /// Suggested 255 char max.
    pub description: Vec<u8>,
    /// Cryptographic commitments to the questions in the game. This prevents the teacher from changing the questions
    /// after the game or even registration has started.
    pub question_commitments: Vec<H256>,
    /// The minimum number of block that must elapse before the questions can be closed.
    pub min_answer_period: u32,
}

impl UtxoData for GameDetails {
    const TYPE_ID: [u8; 4] = *b"kaht";
}

/// A simple token that represents a registered player's right to answer a question once it is revealed.
/// 
/// When a player registers for a game, they get one of these tokens for each question in the game.
/// Later they consume these tokens one at a time by answering questions as they are revealed.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct AnswerTicket;

impl UtxoData for AnswerTicket {
    const TYPE_ID: [u8; 4] = *b"anst";
}

/// An answer to a specific question in a specific game.
/// 
/// Typically the answer will be an integer indicating the index of the answer
/// in the list of potential answers. It is also possible to answer that the teacher
/// did not properly reveal the question off-chain.
/// Design consideration: an alternative approach would be requiring the teacher to reveal
/// the answer on-chain but that would require posting the entire text of question and answers on chain.
/// Maybe some kind of availability game... To be explored.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum Answer {
    /// The teacher did not reveal the question, thus we cannot reasonably answer it.
    TeacherDidNotReveal,
    /// Normal answer indicating the index of the correct answer in the vec.
    NormalAnswer(u32),
}

impl UtxoData for Answer {
    const TYPE_ID: [u8; 4] = *b"answ";
}


/// A cryptographic commitment (eg a hash) to the text of a question and its answers.
/// 
/// The question preimage will be revealed off chain at some point thus opening the question for answering.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct QuestionCommitment;

impl UtxoData for QuestionCommitment {
    const TYPE_ID: [u8; 4] = *b"qcom";
}

/// Reasons that the constraint checkers associated with the kahoot game may fail.
#[derive(Debug, Eq, PartialEq)]
pub enum KahootError {
    /// An input data has the wrong type.
    BadlyTypedInput,
    /// An output data has the wrong type.
    BadlyTypedOutput,
}

/// Create a game by giving basic details like a name, description, and the number of questions.
/// You must also cryptographically commit the the question details by supplying a hash. The
/// questions themselves will not be posted onchain. They will be revealed offchain when the teacher is
/// ready to open that question.
/// 
/// Shoutout @sukhmeat I'm naming it as a noun this time.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct GameCreation;

impl SimpleConstraintChecker for GameCreation {
    type Error = KahootError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!()
    }
}

/// Register to play in a particular game. Requires payment of an ante equal to the game creator's deposit.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct PlayerRegistration;

impl SimpleConstraintChecker for PlayerRegistration {
    type Error = KahootError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!()
    }
}

/// Begin the game, thus also closing the registration period.
/// 
/// In a fancier more production ready version, there would be some minimum amount of time before
/// the game could be started to make sure everyone has a reasonable chance to get in.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct GameOpening;

impl SimpleConstraintChecker for GameOpening {
    type Error = KahootError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!()
    }
}

/// Submit the answer to a question in a game that you previously registered for.
/// 
/// Peeks at an open question and consumes an answer ticket.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct AnswerSubmission;

impl SimpleConstraintChecker for AnswerSubmission {
    type Error = KahootError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        _peeks: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!()
    }
}

// TODO Design consideration: This settlement step is kind of optional.
// We could just consume all the individual questions and answers in the final
// game closing transaction. Doing it this way is more true to KAhoot style, and
// allows settling questions serially. I'll leave it like this for now.
/// Settle the results of an open question.
/// 
/// Consumes the question commitment that was created when the game was open as
/// well as all the answers for that question. In case some user has no-showed
/// after enough time has passed, this transaction may evict their answer ticket.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct QuestionSettlement;

impl SimpleConstraintChecker for QuestionSettlement {
    type Error = KahootError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        peek_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!()
    }
}

/// Settle the results of an entire game, award the token prizes, and clear all game-related utxos.
/// 
/// Consumes the game that was created way back when the teacher first registered as well as
/// a question settlement for each question in the game. Creates a coin for point-scoring player
/// as well as the host.
/// 
/// Currently the token distribution is that the deposit is refunded to the host
/// and the remainder is allocated to players according to their scores. This could easily
/// be extended.
#[derive(Serialize, Deserialize, Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo)]
pub struct GameSettlement;

impl SimpleConstraintChecker for GameSettlement {
    type Error = KahootError;

    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        peek_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error> {
        todo!()
    }
}