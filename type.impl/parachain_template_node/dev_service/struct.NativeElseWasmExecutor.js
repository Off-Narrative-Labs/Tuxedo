(function() {var type_impls = {
"parachain_template_node":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-NativeElseWasmExecutor%3CD%3E\" class=\"impl\"><a href=\"#impl-NativeElseWasmExecutor%3CD%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D&gt; <a class=\"struct\" href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html\" title=\"struct parachain_template_node::dev_service::NativeElseWasmExecutor\">NativeElseWasmExecutor</a>&lt;D&gt;<div class=\"where\">where\n    D: NativeExecutionDispatch,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.new\" class=\"method\"><h4 class=\"code-header\">pub fn <a href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html#tymethod.new\" class=\"fn\">new</a>(\n    fallback_method: WasmExecutionMethod,\n    default_heap_pages: <a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u64.html\">u64</a>&gt;,\n    max_runtime_instances: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.usize.html\">usize</a>,\n    runtime_cache_size: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u8.html\">u8</a>\n) -&gt; <a class=\"struct\" href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html\" title=\"struct parachain_template_node::dev_service::NativeElseWasmExecutor\">NativeElseWasmExecutor</a>&lt;D&gt;</h4></section><span class=\"item-info\"><div class=\"stab deprecated\"><span class=\"emoji\">👎</span><span>Deprecated: use <code>Self::new_with_wasm_executor</code> method instead of it</span></div></span></summary><div class=\"docblock\"><p>Create new instance.</p>\n<h5 id=\"parameters\"><a class=\"doc-anchor\" href=\"#parameters\">§</a>Parameters</h5>\n<p><code>fallback_method</code> - Method used to execute fallback Wasm code.</p>\n<p><code>default_heap_pages</code> - Number of 64KB pages to allocate for Wasm execution. Internally this\nwill be mapped as [<code>HeapAllocStrategy::Static</code>] where <code>default_heap_pages</code> represent the\nstatic number of heap pages to allocate. Defaults to <code>DEFAULT_HEAP_ALLOC_STRATEGY</code> if <code>None</code>\nis provided.</p>\n<p><code>max_runtime_instances</code> - The number of runtime instances to keep in memory ready for reuse.</p>\n<p><code>runtime_cache_size</code> - The capacity of runtime cache.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.new_with_wasm_executor\" class=\"method\"><h4 class=\"code-header\">pub fn <a href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html#tymethod.new_with_wasm_executor\" class=\"fn\">new_with_wasm_executor</a>(\n    executor: WasmExecutor&lt;ExtendedHostFunctions&lt;(HostFunctions, HostFunctions, HostFunctions, HostFunctions, HostFunctions, HostFunctions, HostFunctions, HostFunctions, HostFunctions, HostFunctions, HostFunctions, HostFunctions, HostFunctions), &lt;D as NativeExecutionDispatch&gt;::ExtendHostFunctions&gt;&gt;\n) -&gt; <a class=\"struct\" href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html\" title=\"struct parachain_template_node::dev_service::NativeElseWasmExecutor\">NativeElseWasmExecutor</a>&lt;D&gt;</h4></section></summary><div class=\"docblock\"><p>Create a new instance using the given [<code>WasmExecutor</code>].</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.disable_use_native\" class=\"method\"><h4 class=\"code-header\">pub fn <a href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html#tymethod.disable_use_native\" class=\"fn\">disable_use_native</a>(&amp;mut self)</h4></section></summary><div class=\"docblock\"><p>Disable to use native runtime when possible just behave like <code>WasmExecutor</code>.</p>\n<p>Default to enabled.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.allow_missing_host_functions\" class=\"method\"><h4 class=\"code-header\">pub fn <a href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html#tymethod.allow_missing_host_functions\" class=\"fn\">allow_missing_host_functions</a>(\n    &amp;mut self,\n    allow_missing_host_functions: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.bool.html\">bool</a>\n)</h4></section><span class=\"item-info\"><div class=\"stab deprecated\"><span class=\"emoji\">👎</span><span>Deprecated: use <code>Self::new_with_wasm_executor</code> method instead of it</span></div></span></summary><div class=\"docblock\"><p>Ignore missing function imports if set true.</p>\n</div></details></div></details>",0,"parachain_template_node::service::ParachainExecutor"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-RuntimeVersionOf-for-NativeElseWasmExecutor%3CD%3E\" class=\"impl\"><a href=\"#impl-RuntimeVersionOf-for-NativeElseWasmExecutor%3CD%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D&gt; RuntimeVersionOf for <a class=\"struct\" href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html\" title=\"struct parachain_template_node::dev_service::NativeElseWasmExecutor\">NativeElseWasmExecutor</a>&lt;D&gt;<div class=\"where\">where\n    D: NativeExecutionDispatch,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.runtime_version\" class=\"method trait-impl\"><a href=\"#method.runtime_version\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">runtime_version</a>(\n    &amp;self,\n    ext: &amp;mut dyn Externalities,\n    runtime_code: &amp;RuntimeCode&lt;'_&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;RuntimeVersion, Error&gt;</h4></section></summary><div class='docblock'>Extract [<code>RuntimeVersion</code>] of the given <code>runtime_code</code>.</div></details></div></details>","RuntimeVersionOf","parachain_template_node::service::ParachainExecutor"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-CodeExecutor-for-NativeElseWasmExecutor%3CD%3E\" class=\"impl\"><a href=\"#impl-CodeExecutor-for-NativeElseWasmExecutor%3CD%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D&gt; CodeExecutor for <a class=\"struct\" href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html\" title=\"struct parachain_template_node::dev_service::NativeElseWasmExecutor\">NativeElseWasmExecutor</a>&lt;D&gt;<div class=\"where\">where\n    D: NativeExecutionDispatch + 'static,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Error\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Error\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a class=\"associatedtype\">Error</a> = Error</h4></section></summary><div class='docblock'>Externalities error type.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.call\" class=\"method trait-impl\"><a href=\"#method.call\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">call</a>(\n    &amp;self,\n    ext: &amp;mut dyn Externalities,\n    runtime_code: &amp;RuntimeCode&lt;'_&gt;,\n    method: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.str.html\">str</a>,\n    data: &amp;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u8.html\">u8</a>],\n    context: CallContext\n) -&gt; (<a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.77.2/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u8.html\">u8</a>&gt;, Error&gt;, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.bool.html\">bool</a>)</h4></section></summary><div class='docblock'>Call a given method in the runtime. <a>Read more</a></div></details></div></details>","CodeExecutor","parachain_template_node::service::ParachainExecutor"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-ReadRuntimeVersion-for-NativeElseWasmExecutor%3CD%3E\" class=\"impl\"><a href=\"#impl-ReadRuntimeVersion-for-NativeElseWasmExecutor%3CD%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D&gt; ReadRuntimeVersion for <a class=\"struct\" href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html\" title=\"struct parachain_template_node::dev_service::NativeElseWasmExecutor\">NativeElseWasmExecutor</a>&lt;D&gt;<div class=\"where\">where\n    D: NativeExecutionDispatch,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.read_runtime_version\" class=\"method trait-impl\"><a href=\"#method.read_runtime_version\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">read_runtime_version</a>(\n    &amp;self,\n    wasm_code: &amp;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u8.html\">u8</a>],\n    ext: &amp;mut dyn Externalities\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/1.77.2/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/1.77.2/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.u8.html\">u8</a>&gt;, <a class=\"struct\" href=\"https://doc.rust-lang.org/1.77.2/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>&gt;</h4></section></summary><div class='docblock'>Reads the runtime version information from the given wasm code. <a>Read more</a></div></details></div></details>","ReadRuntimeVersion","parachain_template_node::service::ParachainExecutor"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-NativeElseWasmExecutor%3CD%3E\" class=\"impl\"><a href=\"#impl-Clone-for-NativeElseWasmExecutor%3CD%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.77.2/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html\" title=\"struct parachain_template_node::dev_service::NativeElseWasmExecutor\">NativeElseWasmExecutor</a>&lt;D&gt;<div class=\"where\">where\n    D: NativeExecutionDispatch,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.77.2/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; <a class=\"struct\" href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html\" title=\"struct parachain_template_node::dev_service::NativeElseWasmExecutor\">NativeElseWasmExecutor</a>&lt;D&gt;</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/1.77.2/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/1.77.2/src/core/clone.rs.html#169\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/1.77.2/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.77.2/std/primitive.reference.html\">&amp;Self</a>)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/1.77.2/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","parachain_template_node::service::ParachainExecutor"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-GetNativeVersion-for-NativeElseWasmExecutor%3CD%3E\" class=\"impl\"><a href=\"#impl-GetNativeVersion-for-NativeElseWasmExecutor%3CD%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;D&gt; GetNativeVersion for <a class=\"struct\" href=\"parachain_template_node/dev_service/struct.NativeElseWasmExecutor.html\" title=\"struct parachain_template_node::dev_service::NativeElseWasmExecutor\">NativeElseWasmExecutor</a>&lt;D&gt;<div class=\"where\">where\n    D: NativeExecutionDispatch,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.native_version\" class=\"method trait-impl\"><a href=\"#method.native_version\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">native_version</a>(&amp;self) -&gt; &amp;NativeVersion</h4></section></summary><div class='docblock'>Returns the version of the native runtime.</div></details></div></details>","GetNativeVersion","parachain_template_node::service::ParachainExecutor"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()