use runtime::{Block, OuterVerifier, Output, Transaction};
use sp_core::H256;
use tuxedo_core::{
    dynamic_typing::DynamicallyTypedData,
    types::{Input, OutputRef},
    verifier::*,
};

/// The Filter type which is the closure signature used by functions to filter UTXOS
pub type Filter = Box<dyn Fn(&[Output]) -> Result<Vec<Output>, ()>>;

pub trait OutputFilter {
    /// The Filter type which is the closure signature used by functions to filter UTXOS
    type Filter;
    /// Builds a filter to be passed to wallet sync functions to sync the chosen outputs
    /// to the users DB.
    fn build_filter(verifier: OuterVerifier) -> Self::Filter;
}

pub struct SigCheckFilter;
impl OutputFilter for SigCheckFilter {
    // Todo Add filter error
    type Filter = Result<Filter, ()>;

    fn build_filter(verifier: OuterVerifier) -> Self::Filter {
        Ok(Box::new(move |outputs| {
            let filtered_outputs = outputs
                .iter()
                .cloned()
                .filter(|output| output.verifier == verifier)
                .collect::<Vec<_>>();
            Ok(filtered_outputs)
        }))
    }
}

mod tests {
    use super::*;

    pub struct TestSigCheckFilter;
    impl OutputFilter for TestSigCheckFilter {
        type Filter = Result<Filter, ()>;

        fn build_filter(verifier: OuterVerifier) -> Self::Filter {
            Ok(Box::new(move |outputs| {
                println!("printed something");
                Ok(vec![])
            }))
        }
    }

    #[test]
    fn filter_prints() {
        let verifier = OuterVerifier::SigCheck(SigCheck {
            owner_pubkey: H256::zero(),
        });
        let output = Output {
            verifier: verifier.clone(),
            payload: DynamicallyTypedData {
                data: vec![],
                type_id: *b"1234",
            },
        };

        let my_filter = TestSigCheckFilter::build_filter(verifier).expect("Can build print filter");
        let _ = my_filter(&vec![output]);
    }
}
