(function() {var implementors = {
"tuxedo_core":[["impl&lt;V, V1: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;V&gt;, P: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.Into.html\" title=\"trait core::convert::Into\">Into</a>&lt;<a class=\"struct\" href=\"tuxedo_core/dynamic_typing/struct.DynamicallyTypedData.html\" title=\"struct tuxedo_core::dynamic_typing::DynamicallyTypedData\">DynamicallyTypedData</a>&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.tuple.html\">(P, V1)</a>&gt; for <a class=\"struct\" href=\"tuxedo_core/types/struct.Output.html\" title=\"struct tuxedo_core::types::Output\">Output</a>&lt;V&gt;"],["impl&lt;T: <a class=\"trait\" href=\"tuxedo_core/dynamic_typing/trait.UtxoData.html\" title=\"trait tuxedo_core::dynamic_typing::UtxoData\">UtxoData</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;T&gt; for <a class=\"struct\" href=\"tuxedo_core/dynamic_typing/struct.DynamicallyTypedData.html\" title=\"struct tuxedo_core::dynamic_typing::DynamicallyTypedData\">DynamicallyTypedData</a>"],["impl&lt;V: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"tuxedo_core/dynamic_typing/struct.DynamicallyTypedData.html\" title=\"struct tuxedo_core::dynamic_typing::DynamicallyTypedData\">DynamicallyTypedData</a>&gt; for <a class=\"struct\" href=\"tuxedo_core/types/struct.Output.html\" title=\"struct tuxedo_core::types::Output\">Output</a>&lt;V&gt;"]],
"tuxedo_parachain_core":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"tuxedo_parachain_core/struct.ParachainInherentDataUtxo.html\" title=\"struct tuxedo_parachain_core::ParachainInherentDataUtxo\">ParachainInherentDataUtxo</a>&gt; for ParachainInherentData"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;ParachainInherentData&gt; for <a class=\"struct\" href=\"tuxedo_parachain_core/struct.ParachainInherentDataUtxo.html\" title=\"struct tuxedo_parachain_core::ParachainInherentDataUtxo\">ParachainInherentDataUtxo</a>"]],
"tuxedo_parachain_runtime":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_parachain_runtime/enum.ParachainConstraintChecker.html\" title=\"enum tuxedo_parachain_runtime::ParachainConstraintChecker\">ParachainConstraintChecker</a>&gt; for <a class=\"enum\" href=\"tuxedo_parachain_runtime/enum.InnerConstraintChecker.html\" title=\"enum tuxedo_parachain_runtime::InnerConstraintChecker\">InnerConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_parachain_runtime/enum.ParachainConstraintChecker.html\" title=\"enum tuxedo_parachain_runtime::ParachainConstraintChecker\">ParachainConstraintChecker</a>&gt; for <a class=\"struct\" href=\"tuxedo_core/inherents/struct.InherentAdapter.html\" title=\"struct tuxedo_core::inherents::InherentAdapter\">InherentAdapter</a>&lt;<a class=\"struct\" href=\"parachain_piece/struct.SetParachainInfo.html\" title=\"struct parachain_piece::SetParachainInfo\">SetParachainInfo</a>&lt;<a class=\"struct\" href=\"tuxedo_parachain_runtime/struct.RuntimeParachainConfig.html\" title=\"struct tuxedo_parachain_runtime::RuntimeParachainConfig\">RuntimeParachainConfig</a>&gt;&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"tuxedo_core/inherents/struct.InherentAdapter.html\" title=\"struct tuxedo_core::inherents::InherentAdapter\">InherentAdapter</a>&lt;<a class=\"struct\" href=\"parachain_piece/struct.SetParachainInfo.html\" title=\"struct parachain_piece::SetParachainInfo\">SetParachainInfo</a>&lt;<a class=\"struct\" href=\"tuxedo_parachain_runtime/struct.RuntimeParachainConfig.html\" title=\"struct tuxedo_parachain_runtime::RuntimeParachainConfig\">RuntimeParachainConfig</a>&gt;&gt;&gt; for <a class=\"enum\" href=\"tuxedo_parachain_runtime/enum.ParachainConstraintChecker.html\" title=\"enum tuxedo_parachain_runtime::ParachainConstraintChecker\">ParachainConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_parachain_runtime/enum.InnerConstraintChecker.html\" title=\"enum tuxedo_parachain_runtime::InnerConstraintChecker\">OuterConstraintChecker</a>&gt; for <a class=\"enum\" href=\"tuxedo_parachain_runtime/enum.ParachainConstraintChecker.html\" title=\"enum tuxedo_parachain_runtime::ParachainConstraintChecker\">ParachainConstraintChecker</a>"]],
"tuxedo_template_runtime":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>&gt; for <a class=\"struct\" href=\"amoeba/struct.AmoebaMitosis.html\" title=\"struct amoeba::AmoebaMitosis\">AmoebaMitosis</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>&gt; for <a class=\"struct\" href=\"poe/struct.PoeClaim.html\" title=\"struct poe::PoeClaim\">PoeClaim</a>&lt;<a class=\"struct\" href=\"tuxedo_template_runtime/struct.Runtime.html\" title=\"struct tuxedo_template_runtime::Runtime\">Runtime</a>&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>&gt; for <a class=\"enum\" href=\"money/enum.MoneyConstraintChecker.html\" title=\"enum money::MoneyConstraintChecker\">MoneyConstraintChecker</a>&lt;0&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>&gt; for <a class=\"struct\" href=\"poe/struct.PoeRevoke.html\" title=\"struct poe::PoeRevoke\">PoeRevoke</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>&gt; for <a class=\"struct\" href=\"kitties/struct.FreeKittyConstraintChecker.html\" title=\"struct kitties::FreeKittyConstraintChecker\">FreeKittyConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"amoeba/struct.AmoebaCreation.html\" title=\"struct amoeba::AmoebaCreation\">AmoebaCreation</a>&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>&gt; for <a class=\"struct\" href=\"amoeba/struct.AmoebaDeath.html\" title=\"struct amoeba::AmoebaDeath\">AmoebaDeath</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"tuxedo_core/verifier/struct.UpForGrabs.html\" title=\"struct tuxedo_core::verifier::UpForGrabs\">UpForGrabs</a>&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterVerifier.html\" title=\"enum tuxedo_template_runtime::OuterVerifier\">OuterVerifier</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"kitties/struct.FreeKittyConstraintChecker.html\" title=\"struct kitties::FreeKittyConstraintChecker\">FreeKittyConstraintChecker</a>&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"tuxedo_core/verifier/multi_signature/struct.ThresholdMultiSignature.html\" title=\"struct tuxedo_core::verifier::multi_signature::ThresholdMultiSignature\">ThresholdMultiSignature</a>&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterVerifier.html\" title=\"enum tuxedo_template_runtime::OuterVerifier\">OuterVerifier</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterVerifier.html\" title=\"enum tuxedo_template_runtime::OuterVerifier\">OuterVerifier</a>&gt; for <a class=\"struct\" href=\"tuxedo_core/verifier/simple_signature/struct.Sr25519Signature.html\" title=\"struct tuxedo_core::verifier::simple_signature::Sr25519Signature\">Sr25519Signature</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"tuxedo_core/verifier/simple_signature/struct.Sr25519Signature.html\" title=\"struct tuxedo_core::verifier::simple_signature::Sr25519Signature\">Sr25519Signature</a>&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterVerifier.html\" title=\"enum tuxedo_template_runtime::OuterVerifier\">OuterVerifier</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"amoeba/struct.AmoebaMitosis.html\" title=\"struct amoeba::AmoebaMitosis\">AmoebaMitosis</a>&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterVerifier.html\" title=\"enum tuxedo_template_runtime::OuterVerifier\">OuterVerifier</a>&gt; for <a class=\"struct\" href=\"tuxedo_core/verifier/multi_signature/struct.ThresholdMultiSignature.html\" title=\"struct tuxedo_core::verifier::multi_signature::ThresholdMultiSignature\">ThresholdMultiSignature</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterVerifier.html\" title=\"enum tuxedo_template_runtime::OuterVerifier\">OuterVerifier</a>&gt; for <a class=\"struct\" href=\"tuxedo_core/verifier/struct.UpForGrabs.html\" title=\"struct tuxedo_core::verifier::UpForGrabs\">UpForGrabs</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"tuxedo_core/inherents/struct.InherentAdapter.html\" title=\"struct tuxedo_core::inherents::InherentAdapter\">InherentAdapter</a>&lt;<a class=\"struct\" href=\"timestamp/struct.SetTimestamp.html\" title=\"struct timestamp::SetTimestamp\">SetTimestamp</a>&lt;<a class=\"struct\" href=\"tuxedo_template_runtime/struct.Runtime.html\" title=\"struct tuxedo_template_runtime::Runtime\">Runtime</a>&gt;&gt;&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"poe/struct.PoeDispute.html\" title=\"struct poe::PoeDispute\">PoeDispute</a>&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"poe/struct.PoeClaim.html\" title=\"struct poe::PoeClaim\">PoeClaim</a>&lt;<a class=\"struct\" href=\"tuxedo_template_runtime/struct.Runtime.html\" title=\"struct tuxedo_template_runtime::Runtime\">Runtime</a>&gt;&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"money/enum.MoneyConstraintChecker.html\" title=\"enum money::MoneyConstraintChecker\">MoneyConstraintChecker</a>&lt;0&gt;&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>&gt; for <a class=\"struct\" href=\"poe/struct.PoeDispute.html\" title=\"struct poe::PoeDispute\">PoeDispute</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>&gt; for <a class=\"struct\" href=\"tuxedo_core/inherents/struct.InherentAdapter.html\" title=\"struct tuxedo_core::inherents::InherentAdapter\">InherentAdapter</a>&lt;<a class=\"struct\" href=\"timestamp/struct.SetTimestamp.html\" title=\"struct timestamp::SetTimestamp\">SetTimestamp</a>&lt;<a class=\"struct\" href=\"tuxedo_template_runtime/struct.Runtime.html\" title=\"struct tuxedo_template_runtime::Runtime\">Runtime</a>&gt;&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>&gt; for <a class=\"struct\" href=\"amoeba/struct.AmoebaCreation.html\" title=\"struct amoeba::AmoebaCreation\">AmoebaCreation</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"amoeba/struct.AmoebaDeath.html\" title=\"struct amoeba::AmoebaDeath\">AmoebaDeath</a>&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"poe/struct.PoeRevoke.html\" title=\"struct poe::PoeRevoke\">PoeRevoke</a>&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"struct\" href=\"runtime_upgrade/struct.RuntimeUpgrade.html\" title=\"struct runtime_upgrade::RuntimeUpgrade\">RuntimeUpgrade</a>&gt; for <a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"enum\" href=\"tuxedo_template_runtime/enum.OuterConstraintChecker.html\" title=\"enum tuxedo_template_runtime::OuterConstraintChecker\">OuterConstraintChecker</a>&gt; for <a class=\"struct\" href=\"runtime_upgrade/struct.RuntimeUpgrade.html\" title=\"struct runtime_upgrade::RuntimeUpgrade\">RuntimeUpgrade</a>"]],
"tuxedo_template_wallet":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;OuterConstraintChecker&gt; for <a class=\"enum\" href=\"tuxedo_template_wallet/parachain/enum.ParachainConstraintChecker.html\" title=\"enum tuxedo_template_wallet::parachain::ParachainConstraintChecker\">ParachainConstraintChecker</a>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()