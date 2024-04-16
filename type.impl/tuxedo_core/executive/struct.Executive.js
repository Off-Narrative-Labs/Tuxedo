(function() {var type_impls = {
"tuxedo_parachain_runtime":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Executive%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#41-46\">source</a><a href=\"#impl-Executive%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; <a class=\"struct\" href=\"tuxedo_core/executive/struct.Executive.html\" title=\"struct tuxedo_core::executive::Executive\">Executive</a>&lt;V, C&gt;<div class=\"where\">where\n    V: <a class=\"trait\" href=\"tuxedo_core/verifier/trait.Verifier.html\" title=\"trait tuxedo_core::verifier::Verifier\">Verifier</a>,\n    C: <a class=\"trait\" href=\"tuxedo_core/constraint_checker/trait.ConstraintChecker.html\" title=\"trait tuxedo_core::constraint_checker::ConstraintChecker\">ConstraintChecker</a>,\n    Block&lt;Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;, <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;&gt;: Block&lt;Extrinsic = <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;, Hash = H256&gt;,\n    <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;: Extrinsic,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.validate_tuxedo_transaction\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#52-54\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.validate_tuxedo_transaction\" class=\"fn\">validate_tuxedo_transaction</a>(\n    transaction: &amp;<a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;ValidTransaction, <a class=\"enum\" href=\"tuxedo_core/types/enum.UtxoError.html\" title=\"enum tuxedo_core::types::UtxoError\">UtxoError</a>&lt;&lt;C as <a class=\"trait\" href=\"tuxedo_core/constraint_checker/trait.ConstraintChecker.html\" title=\"trait tuxedo_core::constraint_checker::ConstraintChecker\">ConstraintChecker</a>&gt;::<a class=\"associatedtype\" href=\"tuxedo_core/constraint_checker/trait.ConstraintChecker.html#associatedtype.Error\" title=\"type tuxedo_core::constraint_checker::ConstraintChecker::Error\">Error</a>&gt;&gt;</h4></section></summary><div class=\"docblock\"><p>Does pool-style validation of a tuxedo transaction.\nDoes not commit anything to storage.\nThis returns Ok even if some inputs are still missing because the tagged transaction pool can handle that.\nWe later check that there are no missing inputs in <code>apply_tuxedo_transaction</code></p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.apply_tuxedo_transaction\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#192\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.apply_tuxedo_transaction\" class=\"fn\">apply_tuxedo_transaction</a>(\n    transaction: <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.unit.html\">()</a>, <a class=\"enum\" href=\"tuxedo_core/types/enum.UtxoError.html\" title=\"enum tuxedo_core::types::UtxoError\">UtxoError</a>&lt;&lt;C as <a class=\"trait\" href=\"tuxedo_core/constraint_checker/trait.ConstraintChecker.html\" title=\"trait tuxedo_core::constraint_checker::ConstraintChecker\">ConstraintChecker</a>&gt;::<a class=\"associatedtype\" href=\"tuxedo_core/constraint_checker/trait.ConstraintChecker.html#associatedtype.Error\" title=\"type tuxedo_core::constraint_checker::ConstraintChecker::Error\">Error</a>&gt;&gt;</h4></section></summary><div class=\"docblock\"><p>Does full verification and application of tuxedo transactions.\nMost of the validation happens in the call to <code>validate_tuxedo_transaction</code>.\nOnce those checks are done we make sure there are no missing inputs and then update storage.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.block_height\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#240\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.block_height\" class=\"fn\">block_height</a>() -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a></h4></section></summary><div class=\"docblock\"><p>A helper function that allows tuxedo runtimes to read the current block height</p>\n</div></details><section id=\"method.open_block\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#249\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.open_block\" class=\"fn\">open_block</a>(header: &amp;Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;)</h4></section><section id=\"method.apply_extrinsic\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#264\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.apply_extrinsic\" class=\"fn\">apply_extrinsic</a>(\n    extrinsic: <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.unit.html\">()</a>, DispatchError&gt;, TransactionValidityError&gt;</h4></section><section id=\"method.close_block\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#285\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.close_block\" class=\"fn\">close_block</a>() -&gt; Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;</h4></section><section id=\"method.execute_block\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#312\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.execute_block\" class=\"fn\">execute_block</a>(block: Block&lt;Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;, <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;&gt;)</h4></section><section id=\"method.validate_transaction\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#381-385\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.validate_transaction\" class=\"fn\">validate_transaction</a>(\n    source: TransactionSource,\n    tx: <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;,\n    block_hash: &lt;Block&lt;Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;, <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;&gt; as Block&gt;::Hash\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;ValidTransaction, TransactionValidityError&gt;</h4></section><section id=\"method.inherent_extrinsics\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#420\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.inherent_extrinsics\" class=\"fn\">inherent_extrinsics</a>(data: InherentData) -&gt; <a class=\"struct\" href=\"https://doc.rust-lang.org/1.77.2/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;&gt;</h4></section><section id=\"method.check_inherents\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#457-460\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.check_inherents\" class=\"fn\">check_inherents</a>(\n    block: Block&lt;Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;, <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;&gt;,\n    data: InherentData\n) -&gt; CheckInherentsResult</h4></section></div></details>",0,"tuxedo_parachain_runtime::Executive"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-ParachainExecutiveExtension-for-Executive%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_parachain_core/collation_api.rs.html#17\">source</a><a href=\"#impl-ParachainExecutiveExtension-for-Executive%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; <a class=\"trait\" href=\"tuxedo_parachain_core/collation_api/trait.ParachainExecutiveExtension.html\" title=\"trait tuxedo_parachain_core::collation_api::ParachainExecutiveExtension\">ParachainExecutiveExtension</a> for <a class=\"struct\" href=\"tuxedo_core/executive/struct.Executive.html\" title=\"struct tuxedo_core::executive::Executive\">Executive</a>&lt;V, C&gt;</h3></section></summary><div class=\"impl-items\"><section id=\"method.collect_collation_info\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/tuxedo_parachain_core/collation_api.rs.html#18\">source</a><a href=\"#method.collect_collation_info\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"tuxedo_parachain_core/collation_api/trait.ParachainExecutiveExtension.html#tymethod.collect_collation_info\" class=\"fn\">collect_collation_info</a>(header: &amp;Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;) -&gt; CollationInfo</h4></section></div></details>","ParachainExecutiveExtension","tuxedo_parachain_runtime::Executive"]],
"tuxedo_template_runtime":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Executive%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#41-46\">source</a><a href=\"#impl-Executive%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; <a class=\"struct\" href=\"tuxedo_core/executive/struct.Executive.html\" title=\"struct tuxedo_core::executive::Executive\">Executive</a>&lt;V, C&gt;<div class=\"where\">where\n    V: <a class=\"trait\" href=\"tuxedo_core/verifier/trait.Verifier.html\" title=\"trait tuxedo_core::verifier::Verifier\">Verifier</a>,\n    C: <a class=\"trait\" href=\"tuxedo_core/constraint_checker/trait.ConstraintChecker.html\" title=\"trait tuxedo_core::constraint_checker::ConstraintChecker\">ConstraintChecker</a>,\n    Block&lt;Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;, <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;&gt;: Block&lt;Extrinsic = <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;, Hash = H256&gt;,\n    <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;: Extrinsic,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.validate_tuxedo_transaction\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#52-54\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.validate_tuxedo_transaction\" class=\"fn\">validate_tuxedo_transaction</a>(\n    transaction: &amp;<a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;ValidTransaction, <a class=\"enum\" href=\"tuxedo_core/types/enum.UtxoError.html\" title=\"enum tuxedo_core::types::UtxoError\">UtxoError</a>&lt;&lt;C as <a class=\"trait\" href=\"tuxedo_core/constraint_checker/trait.ConstraintChecker.html\" title=\"trait tuxedo_core::constraint_checker::ConstraintChecker\">ConstraintChecker</a>&gt;::<a class=\"associatedtype\" href=\"tuxedo_core/constraint_checker/trait.ConstraintChecker.html#associatedtype.Error\" title=\"type tuxedo_core::constraint_checker::ConstraintChecker::Error\">Error</a>&gt;&gt;</h4></section></summary><div class=\"docblock\"><p>Does pool-style validation of a tuxedo transaction.\nDoes not commit anything to storage.\nThis returns Ok even if some inputs are still missing because the tagged transaction pool can handle that.\nWe later check that there are no missing inputs in <code>apply_tuxedo_transaction</code></p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.apply_tuxedo_transaction\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#192\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.apply_tuxedo_transaction\" class=\"fn\">apply_tuxedo_transaction</a>(\n    transaction: <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.unit.html\">()</a>, <a class=\"enum\" href=\"tuxedo_core/types/enum.UtxoError.html\" title=\"enum tuxedo_core::types::UtxoError\">UtxoError</a>&lt;&lt;C as <a class=\"trait\" href=\"tuxedo_core/constraint_checker/trait.ConstraintChecker.html\" title=\"trait tuxedo_core::constraint_checker::ConstraintChecker\">ConstraintChecker</a>&gt;::<a class=\"associatedtype\" href=\"tuxedo_core/constraint_checker/trait.ConstraintChecker.html#associatedtype.Error\" title=\"type tuxedo_core::constraint_checker::ConstraintChecker::Error\">Error</a>&gt;&gt;</h4></section></summary><div class=\"docblock\"><p>Does full verification and application of tuxedo transactions.\nMost of the validation happens in the call to <code>validate_tuxedo_transaction</code>.\nOnce those checks are done we make sure there are no missing inputs and then update storage.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.block_height\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#240\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.block_height\" class=\"fn\">block_height</a>() -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a></h4></section></summary><div class=\"docblock\"><p>A helper function that allows tuxedo runtimes to read the current block height</p>\n</div></details><section id=\"method.open_block\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#249\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.open_block\" class=\"fn\">open_block</a>(header: &amp;Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;)</h4></section><section id=\"method.apply_extrinsic\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#264\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.apply_extrinsic\" class=\"fn\">apply_extrinsic</a>(\n    extrinsic: <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.unit.html\">()</a>, DispatchError&gt;, TransactionValidityError&gt;</h4></section><section id=\"method.close_block\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#285\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.close_block\" class=\"fn\">close_block</a>() -&gt; Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;</h4></section><section id=\"method.execute_block\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#312\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.execute_block\" class=\"fn\">execute_block</a>(block: Block&lt;Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;, <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;&gt;)</h4></section><section id=\"method.validate_transaction\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#381-385\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.validate_transaction\" class=\"fn\">validate_transaction</a>(\n    source: TransactionSource,\n    tx: <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;,\n    block_hash: &lt;Block&lt;Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;, <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;&gt; as Block&gt;::Hash\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;ValidTransaction, TransactionValidityError&gt;</h4></section><section id=\"method.inherent_extrinsics\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#420\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.inherent_extrinsics\" class=\"fn\">inherent_extrinsics</a>(data: InherentData) -&gt; <a class=\"struct\" href=\"https://doc.rust-lang.org/1.77.2/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;&gt;</h4></section><section id=\"method.check_inherents\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/executive.rs.html#457-460\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/executive/struct.Executive.html#tymethod.check_inherents\" class=\"fn\">check_inherents</a>(\n    block: Block&lt;Header&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u32.html\">u32</a>, BlakeTwo256&gt;, <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;&gt;,\n    data: InherentData\n) -&gt; CheckInherentsResult</h4></section></div></details>",0,"tuxedo_template_runtime::Executive"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()