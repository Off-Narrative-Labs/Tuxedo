(function() {var type_impls = {
"tuxedo_template_runtime":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#47\">source</a><a href=\"#impl-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;<div class=\"where\">where\n    V: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    C: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.transform\" class=\"method\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#51\">source</a><h4 class=\"code-header\">pub fn <a href=\"tuxedo_core/types/struct.Transaction.html#tymethod.transform\" class=\"fn\">transform</a>&lt;D&gt;(&amp;self) -&gt; <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, D&gt;<div class=\"where\">where\n    D: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;C&gt;,</div></h4></section></summary><div class=\"docblock\"><p>A helper function for transforming a transaction generic over one\nkind of constraint checker into a transaction generic over another type\nof constraint checker. This is useful when moving up and down the aggregation tree.</p>\n</div></details></div></details>",0,"tuxedo_template_runtime::Transaction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#impl-Debug-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;<div class=\"where\">where\n    V: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,\n    C: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","tuxedo_template_runtime::Transaction"],["<section id=\"impl-StructuralEq-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#impl-StructuralEq-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.StructuralEq.html\" title=\"trait core::marker::StructuralEq\">StructuralEq</a> for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;</h3></section>","StructuralEq","tuxedo_template_runtime::Transaction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Default-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#impl-Default-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a> for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;<div class=\"where\">where\n    V: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,\n    C: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html\" title=\"trait core::default::Default\">Default</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.default\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#method.default\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html#tymethod.default\" class=\"fn\">default</a>() -&gt; <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;</h4></section></summary><div class='docblock'>Returns the “default value” for a type. <a href=\"https://doc.rust-lang.org/nightly/core/default/trait.Default.html#tymethod.default\">Read more</a></div></details></div></details>","Default","tuxedo_template_runtime::Transaction"],["<section id=\"impl-Eq-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#impl-Eq-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;<div class=\"where\">where\n    V: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a>,\n    C: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a>,</div></h3></section>","Eq","tuxedo_template_runtime::Transaction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Decode-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#81\">source</a><a href=\"#impl-Decode-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; Decode for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;<div class=\"where\">where\n    V: Decode,\n    C: Decode,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.decode\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#82-84\">source</a><a href=\"#method.decode\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">decode</a>&lt;I&gt;(input: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut I</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;, Error&gt;<div class=\"where\">where\n    I: Input,</div></h4></section></summary><div class='docblock'>Attempt to deserialise the value from input.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.decode_into\" class=\"method trait-impl\"><a href=\"#method.decode_into\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">decode_into</a>&lt;I&gt;(\n    input: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut I</a>,\n    dst: &amp;mut <a class=\"union\" href=\"https://doc.rust-lang.org/nightly/core/mem/maybe_uninit/union.MaybeUninit.html\" title=\"union core::mem::maybe_uninit::MaybeUninit\">MaybeUninit</a>&lt;Self&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;DecodeFinished, Error&gt;<div class=\"where\">where\n    I: Input,</div></h4></section></summary><div class='docblock'>Attempt to deserialize the value from input into a pre-allocated piece of memory. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.skip\" class=\"method trait-impl\"><a href=\"#method.skip\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">skip</a>&lt;I&gt;(input: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut I</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a>, Error&gt;<div class=\"where\">where\n    I: Input,</div></h4></section></summary><div class='docblock'>Attempt to skip the encoded value from input. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.encoded_fixed_size\" class=\"method trait-impl\"><a href=\"#method.encoded_fixed_size\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">encoded_fixed_size</a>() -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>&gt;</h4></section></summary><div class='docblock'>Returns the fixed encoded size of the type. <a>Read more</a></div></details></div></details>","Decode","tuxedo_template_runtime::Transaction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Encode-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#63\">source</a><a href=\"#impl-Encode-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; Encode for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;<div class=\"where\">where\n    V: Encode,\n    C: Encode,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.encode_to\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#64\">source</a><a href=\"#method.encode_to\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">encode_to</a>&lt;T&gt;(&amp;self, dest: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut T</a>)<div class=\"where\">where\n    T: Output + ?<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>,</div></h4></section></summary><div class='docblock'>Convert self to a slice and append it to the destination.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.size_hint\" class=\"method trait-impl\"><a href=\"#method.size_hint\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">size_hint</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a></h4></section></summary><div class='docblock'>If possible give a hint of expected size of the encoding. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.encode\" class=\"method trait-impl\"><a href=\"#method.encode\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">encode</a>(&amp;self) -&gt; <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a>&gt; <a href=\"#\" class=\"tooltip\" data-notable-ty=\"Vec&lt;u8&gt;\">ⓘ</a></h4></section></summary><div class='docblock'>Convert self to an owned vector.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.using_encoded\" class=\"method trait-impl\"><a href=\"#method.using_encoded\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">using_encoded</a>&lt;R, F&gt;(&amp;self, f: F) -&gt; R<div class=\"where\">where\n    F: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/function/trait.FnOnce.html\" title=\"trait core::ops::function::FnOnce\">FnOnce</a>(&amp;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a>]) -&gt; R,</div></h4></section></summary><div class='docblock'>Convert self to a slice and then invoke the given closure with it.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.encoded_size\" class=\"method trait-impl\"><a href=\"#method.encoded_size\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">encoded_size</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a></h4></section></summary><div class='docblock'>Calculates the encoded size. <a>Read more</a></div></details></div></details>","Encode","tuxedo_template_runtime::Transaction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#impl-Clone-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;<div class=\"where\">where\n    V: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    C: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/clone.rs.html#169\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Self</a>)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","tuxedo_template_runtime::Transaction"],["<section id=\"impl-StructuralPartialEq-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#impl-StructuralPartialEq-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.StructuralPartialEq.html\" title=\"trait core::marker::StructuralPartialEq\">StructuralPartialEq</a> for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;</h3></section>","StructuralPartialEq","tuxedo_template_runtime::Transaction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Serialize-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#impl-Serialize-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.193/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a> for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;<div class=\"where\">where\n    V: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.193/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a>,\n    C: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.193/serde/ser/trait.Serialize.html\" title=\"trait serde::ser::Serialize\">Serialize</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.serialize\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#method.serialize\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://docs.rs/serde/1.0.193/serde/ser/trait.Serialize.html#tymethod.serialize\" class=\"fn\">serialize</a>&lt;__S&gt;(\n    &amp;self,\n    __serializer: __S\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;&lt;__S as <a class=\"trait\" href=\"https://docs.rs/serde/1.0.193/serde/ser/trait.Serializer.html\" title=\"trait serde::ser::Serializer\">Serializer</a>&gt;::<a class=\"associatedtype\" href=\"https://docs.rs/serde/1.0.193/serde/ser/trait.Serializer.html#associatedtype.Ok\" title=\"type serde::ser::Serializer::Ok\">Ok</a>, &lt;__S as <a class=\"trait\" href=\"https://docs.rs/serde/1.0.193/serde/ser/trait.Serializer.html\" title=\"trait serde::ser::Serializer\">Serializer</a>&gt;::<a class=\"associatedtype\" href=\"https://docs.rs/serde/1.0.193/serde/ser/trait.Serializer.html#associatedtype.Error\" title=\"type serde::ser::Serializer::Error\">Error</a>&gt;<div class=\"where\">where\n    __S: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.193/serde/ser/trait.Serializer.html\" title=\"trait serde::ser::Serializer\">Serializer</a>,</div></h4></section></summary><div class='docblock'>Serialize this value into the given Serde serializer. <a href=\"https://docs.rs/serde/1.0.193/serde/ser/trait.Serialize.html#tymethod.serialize\">Read more</a></div></details></div></details>","Serialize","tuxedo_template_runtime::Transaction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Deserialize%3C'de%3E-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#impl-Deserialize%3C'de%3E-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'de, V, C&gt; <a class=\"trait\" href=\"https://docs.rs/serde/1.0.193/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt; for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;<div class=\"where\">where\n    V: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.193/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt;,\n    C: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.193/serde/de/trait.Deserialize.html\" title=\"trait serde::de::Deserialize\">Deserialize</a>&lt;'de&gt;,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.deserialize\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#method.deserialize\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://docs.rs/serde/1.0.193/serde/de/trait.Deserialize.html#tymethod.deserialize\" class=\"fn\">deserialize</a>&lt;__D&gt;(\n    __deserializer: __D\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;, &lt;__D as <a class=\"trait\" href=\"https://docs.rs/serde/1.0.193/serde/de/trait.Deserializer.html\" title=\"trait serde::de::Deserializer\">Deserializer</a>&lt;'de&gt;&gt;::<a class=\"associatedtype\" href=\"https://docs.rs/serde/1.0.193/serde/de/trait.Deserializer.html#associatedtype.Error\" title=\"type serde::de::Deserializer::Error\">Error</a>&gt;<div class=\"where\">where\n    __D: <a class=\"trait\" href=\"https://docs.rs/serde/1.0.193/serde/de/trait.Deserializer.html\" title=\"trait serde::de::Deserializer\">Deserializer</a>&lt;'de&gt;,</div></h4></section></summary><div class='docblock'>Deserialize this value from the given Serde deserializer. <a href=\"https://docs.rs/serde/1.0.193/serde/de/trait.Deserialize.html#tymethod.deserialize\">Read more</a></div></details></div></details>","Deserialize<'de>","tuxedo_template_runtime::Transaction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-TypeInfo-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#impl-TypeInfo-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; TypeInfo for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;<div class=\"where\">where\n    <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"struct\" href=\"tuxedo_core/types/struct.Output.html\" title=\"struct tuxedo_core::types::Output\">Output</a>&lt;V&gt;&gt;: TypeInfo + 'static,\n    C: TypeInfo + 'static,\n    V: TypeInfo + 'static,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Identity\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Identity\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a class=\"associatedtype\">Identity</a> = <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;</h4></section></summary><div class='docblock'>The type identifying for which type info is provided. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.type_info\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#method.type_info\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">type_info</a>() -&gt; Type</h4></section></summary><div class='docblock'>Returns the static type identifier for <code>Self</code>.</div></details></div></details>","TypeInfo","tuxedo_template_runtime::Transaction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Extrinsic-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#108-111\">source</a><a href=\"#impl-Extrinsic-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; Extrinsic for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;<div class=\"where\">where\n    C: TypeInfo + <a class=\"trait\" href=\"tuxedo_core/constraint_checker/trait.ConstraintChecker.html\" title=\"trait tuxedo_core::constraint_checker::ConstraintChecker\">ConstraintChecker</a>&lt;V&gt; + 'static,\n    V: TypeInfo + <a class=\"trait\" href=\"tuxedo_core/verifier/trait.Verifier.html\" title=\"trait tuxedo_core::verifier::Verifier\">Verifier</a> + 'static,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Call\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Call\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a class=\"associatedtype\">Call</a> = <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;</h4></section></summary><div class='docblock'>The function call.</div></details><details class=\"toggle\" open><summary><section id=\"associatedtype.SignaturePayload\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.SignaturePayload\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a class=\"associatedtype\">SignaturePayload</a> = <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a></h4></section></summary><div class='docblock'>The payload we carry for signed extrinsics. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.new\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#116\">source</a><a href=\"#method.new\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">new</a>(\n    data: <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;,\n    _: <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&lt;<a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt; as Extrinsic&gt;::SignaturePayload&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;&gt;</h4></section></summary><div class='docblock'>Create new instance of the extrinsic. <a>Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_signed\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#132\">source</a><a href=\"#method.is_signed\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">is_signed</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a>&gt;</h4></section></summary><div class='docblock'>Is this <code>Extrinsic</code> signed?\nIf no information are available about signed/unsigned, <code>None</code> should be returned.</div></details></div></details>","Extrinsic","tuxedo_template_runtime::Transaction"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-PartialEq-for-Transaction%3CV,+C%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#impl-PartialEq-for-Transaction%3CV,+C%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;V, C&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> for <a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;<div class=\"where\">where\n    V: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>,\n    C: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.eq\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/tuxedo_core/types.rs.html#35\">source</a><a href=\"#method.eq\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html#tymethod.eq\" class=\"fn\">eq</a>(&amp;self, other: &amp;<a class=\"struct\" href=\"tuxedo_core/types/struct.Transaction.html\" title=\"struct tuxedo_core::types::Transaction\">Transaction</a>&lt;V, C&gt;) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>This method tests for <code>self</code> and <code>other</code> values to be equal, and is used\nby <code>==</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.ne\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/cmp.rs.html#242\">source</a></span><a href=\"#method.ne\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html#method.ne\" class=\"fn\">ne</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>This method tests for <code>!=</code>. The default implementation is almost always\nsufficient, and should not be overridden without very good reason.</div></details></div></details>","PartialEq","tuxedo_template_runtime::Transaction"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()