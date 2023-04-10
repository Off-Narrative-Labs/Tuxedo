use runtime::{OuterVerifier, Output};
use sp_core::H256;
use tuxedo_core::types::OutputRef;

pub type OutputInfo = (Output, OutputRef);

type TxHash = H256;
/// The Filter type which is the closure signature used by functions to filter UTXOS
pub type Filter = Box<dyn Fn(&[Output], &TxHash) -> Result<Vec<OutputInfo>, ()>>;

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
        Ok(Box::new(move |outputs, tx_hash| {
            let filtered_outputs = outputs
                .iter()
                .enumerate()
                .map(|(i, output)| {
                    (
                        output.clone(),
                        OutputRef {
                            tx_hash: *tx_hash,
                            index: i as u32,
                        },
                    )
                })
                .filter(|(output, _)| output.verifier == verifier)
                .collect::<Vec<_>>();
            Ok(filtered_outputs)
        }))
    }
}

mod tests {
    use super::*;

    #[cfg(test)]
    use tuxedo_core::{dynamic_typing::DynamicallyTypedData, verifier::*};

    pub struct TestSigCheckFilter;
    impl OutputFilter for TestSigCheckFilter {
        type Filter = Result<Filter, ()>;

        fn build_filter(_verifier: OuterVerifier) -> Self::Filter {
            Ok(Box::new(move |_outputs, _tx_hash| {
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
        let _ = my_filter(&vec![output], &H256::zero());
    }

    #[test]
    fn filter_sig_check_works() {
        let verifier = OuterVerifier::SigCheck(SigCheck {
            owner_pubkey: H256::zero(),
        });

        let outputs_to_filter = vec![
            Output {
                verifier: verifier.clone(),
                payload: DynamicallyTypedData {
                    data: vec![],
                    type_id: *b"1234",
                },
            },
            Output {
                verifier: OuterVerifier::SigCheck(SigCheck {
                    owner_pubkey: H256::from_slice(b"asdfasdfasdfasdfasdfasdfasdfasdf"),
                }),
                payload: DynamicallyTypedData {
                    data: vec![],
                    type_id: *b"1234",
                },
            },
            Output {
                verifier: OuterVerifier::ThresholdMultiSignature(ThresholdMultiSignature {
                    threshold: 1,
                    signatories: vec![H256::zero()],
                }),
                payload: DynamicallyTypedData {
                    data: vec![],
                    type_id: *b"1234",
                },
            },
        ];

        let expected_filtered_output_infos = vec![(
            Output {
                verifier: verifier.clone(),
                payload: DynamicallyTypedData {
                    data: vec![],
                    type_id: *b"1234",
                },
            },
            OutputRef {
                tx_hash: H256::zero(),
                index: 0,
            },
        )];

        let my_filter = SigCheckFilter::build_filter(verifier).expect("Can build sigcheck filter");
        let filtered_output_infos = my_filter(&outputs_to_filter, &H256::zero())
            .expect("Can filter the outputs by verifier correctly");

        assert_eq!(filtered_output_infos, expected_filtered_output_infos);
    }
}
