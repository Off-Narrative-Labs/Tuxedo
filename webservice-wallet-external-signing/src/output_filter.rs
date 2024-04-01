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

pub struct Sr25519SignatureFilter;
impl OutputFilter for Sr25519SignatureFilter {
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

    pub struct TestSr25519SignatureFilter;
    impl OutputFilter for TestSr25519SignatureFilter {
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
        let verifier = OuterVerifier::Sr25519Signature(Sr25519Signature {
            owner_pubkey: H256::zero(),
        });
        let output = Output {
            verifier: verifier.clone(),
            payload: DynamicallyTypedData {
                data: vec![],
                type_id: *b"1234",
            },
        };

        let my_filter =
            TestSr25519SignatureFilter::build_filter(verifier).expect("Can build print filter");
        let _ = my_filter(&[output], &H256::zero());
    }

    #[test]
    fn filter_sr25519_signature_works() {
        let verifier = OuterVerifier::Sr25519Signature(Sr25519Signature {
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
                verifier: OuterVerifier::Sr25519Signature(Sr25519Signature {
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

        let my_filter = Sr25519SignatureFilter::build_filter(verifier)
            .expect("Can build Sr25519Signature filter");
        let filtered_output_infos = my_filter(&outputs_to_filter, &H256::zero())
            .expect("Can filter the outputs by verifier correctly");

        assert_eq!(filtered_output_infos, expected_filtered_output_infos);
    }
}
