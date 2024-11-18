(function() {
    var type_impls = Object.fromEntries([["function_grep",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-SingleOrMany%3CA,+M%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/function_grep/filter.rs.html#226\">Source</a><a href=\"#impl-Debug-for-SingleOrMany%3CA,+M%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;A: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"function_grep/filter/trait.Info.html\" title=\"trait function_grep::filter::Info\">Info</a>&lt;Supported = <a class=\"struct\" href=\"function_grep/filter/struct.All.html\" title=\"struct function_grep::filter::All\">All</a>&gt;, M: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"function_grep/filter/trait.Info.html\" title=\"trait function_grep::filter::Info\">Info</a>&lt;Supported = <a class=\"struct\" href=\"function_grep/filter/struct.Language.html\" title=\"struct function_grep::filter::Language\">Language</a>&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"enum\" href=\"function_grep/filter/enum.SingleOrMany.html\" title=\"enum function_grep::filter::SingleOrMany\">SingleOrMany</a>&lt;A, M&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/function_grep/filter.rs.html#226\">Source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","function_grep::filter::FilterType","function_grep::filter::InstantiatedFilterType"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Display-for-SingleOrMany%3CA,+M%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/function_grep/filter.rs.html#252-256\">Source</a><a href=\"#impl-Display-for-SingleOrMany%3CA,+M%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;A: <a class=\"trait\" href=\"function_grep/filter/trait.Info.html\" title=\"trait function_grep::filter::Info\">Info</a>&lt;Supported = <a class=\"struct\" href=\"function_grep/filter/struct.All.html\" title=\"struct function_grep::filter::All\">All</a>&gt;, M: <a class=\"trait\" href=\"function_grep/filter/trait.Info.html\" title=\"trait function_grep::filter::Info\">Info</a>&lt;Supported = <a class=\"struct\" href=\"function_grep/filter/struct.Language.html\" title=\"struct function_grep::filter::Language\">Language</a>&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html\" title=\"trait core::fmt::Display\">Display</a> for <a class=\"enum\" href=\"function_grep/filter/enum.SingleOrMany.html\" title=\"enum function_grep::filter::SingleOrMany\">SingleOrMany</a>&lt;A, M&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/function_grep/filter.rs.html#253-255\">Source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Display.html#tymethod.fmt\">Read more</a></div></details></div></details>","Display","function_grep::filter::FilterType","function_grep::filter::InstantiatedFilterType"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-PartialEq-for-SingleOrMany%3CA,+M%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/function_grep/filter.rs.html#226\">Source</a><a href=\"#impl-PartialEq-for-SingleOrMany%3CA,+M%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;A: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"function_grep/filter/trait.Info.html\" title=\"trait function_grep::filter::Info\">Info</a>&lt;Supported = <a class=\"struct\" href=\"function_grep/filter/struct.All.html\" title=\"struct function_grep::filter::All\">All</a>&gt;, M: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> + <a class=\"trait\" href=\"function_grep/filter/trait.Info.html\" title=\"trait function_grep::filter::Info\">Info</a>&lt;Supported = <a class=\"struct\" href=\"function_grep/filter/struct.Language.html\" title=\"struct function_grep::filter::Language\">Language</a>&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a> for <a class=\"enum\" href=\"function_grep/filter/enum.SingleOrMany.html\" title=\"enum function_grep::filter::SingleOrMany\">SingleOrMany</a>&lt;A, M&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.eq\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/function_grep/filter.rs.html#226\">Source</a><a href=\"#method.eq\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html#tymethod.eq\" class=\"fn\">eq</a>(&amp;self, other: &amp;<a class=\"enum\" href=\"function_grep/filter/enum.SingleOrMany.html\" title=\"enum function_grep::filter::SingleOrMany\">SingleOrMany</a>&lt;A, M&gt;) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests for <code>self</code> and <code>other</code> values to be equal, and is used by <code>==</code>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.ne\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/cmp.rs.html#261\">Source</a></span><a href=\"#method.ne\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html#method.ne\" class=\"fn\">ne</a>(&amp;self, other: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Rhs</a>) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Tests for <code>!=</code>. The default implementation is almost always sufficient,\nand should not be overridden without very good reason.</div></details></div></details>","PartialEq","function_grep::filter::FilterType","function_grep::filter::InstantiatedFilterType"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-SingleOrMany%3CA,+M%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/function_grep/filter.rs.html#232-251\">Source</a><a href=\"#impl-SingleOrMany%3CA,+M%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;A: <a class=\"trait\" href=\"function_grep/filter/trait.Info.html\" title=\"trait function_grep::filter::Info\">Info</a>&lt;Supported = <a class=\"struct\" href=\"function_grep/filter/struct.All.html\" title=\"struct function_grep::filter::All\">All</a>&gt;, M: <a class=\"trait\" href=\"function_grep/filter/trait.Info.html\" title=\"trait function_grep::filter::Info\">Info</a>&lt;Supported = <a class=\"struct\" href=\"function_grep/filter/struct.Language.html\" title=\"struct function_grep::filter::Language\">Language</a>&gt;&gt; <a class=\"enum\" href=\"function_grep/filter/enum.SingleOrMany.html\" title=\"enum function_grep::filter::SingleOrMany\">SingleOrMany</a>&lt;A, M&gt;</h3></section></summary><div class=\"impl-items\"><section id=\"method.filter_name\" class=\"method\"><a class=\"src rightside\" href=\"src/function_grep/filter.rs.html#234-240\">Source</a><h4 class=\"code-header\">pub fn <a href=\"function_grep/filter/enum.SingleOrMany.html#tymethod.filter_name\" class=\"fn\">filter_name</a>(&amp;self) -&gt; <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a></h4></section><section id=\"method.supports\" class=\"method\"><a class=\"src rightside\" href=\"src/function_grep/filter.rs.html#243-250\">Source</a><h4 class=\"code-header\">pub fn <a href=\"function_grep/filter/enum.SingleOrMany.html#tymethod.supports\" class=\"fn\">supports</a>(&amp;self) -&gt; <a class=\"enum\" href=\"function_grep/enum.SupportedLanguages.html\" title=\"enum function_grep::SupportedLanguages\">SupportedLanguages</a></h4></section></div></details>",0,"function_grep::filter::FilterType","function_grep::filter::InstantiatedFilterType"],["<section id=\"impl-Eq-for-SingleOrMany%3CA,+M%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/function_grep/filter.rs.html#226\">Source</a><a href=\"#impl-Eq-for-SingleOrMany%3CA,+M%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;A: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"function_grep/filter/trait.Info.html\" title=\"trait function_grep::filter::Info\">Info</a>&lt;Supported = <a class=\"struct\" href=\"function_grep/filter/struct.All.html\" title=\"struct function_grep::filter::All\">All</a>&gt;, M: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> + <a class=\"trait\" href=\"function_grep/filter/trait.Info.html\" title=\"trait function_grep::filter::Info\">Info</a>&lt;Supported = <a class=\"struct\" href=\"function_grep/filter/struct.Language.html\" title=\"struct function_grep::filter::Language\">Language</a>&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.Eq.html\" title=\"trait core::cmp::Eq\">Eq</a> for <a class=\"enum\" href=\"function_grep/filter/enum.SingleOrMany.html\" title=\"enum function_grep::filter::SingleOrMany\">SingleOrMany</a>&lt;A, M&gt;</h3></section>","Eq","function_grep::filter::FilterType","function_grep::filter::InstantiatedFilterType"],["<section id=\"impl-StructuralPartialEq-for-SingleOrMany%3CA,+M%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/function_grep/filter.rs.html#226\">Source</a><a href=\"#impl-StructuralPartialEq-for-SingleOrMany%3CA,+M%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;A: <a class=\"trait\" href=\"function_grep/filter/trait.Info.html\" title=\"trait function_grep::filter::Info\">Info</a>&lt;Supported = <a class=\"struct\" href=\"function_grep/filter/struct.All.html\" title=\"struct function_grep::filter::All\">All</a>&gt;, M: <a class=\"trait\" href=\"function_grep/filter/trait.Info.html\" title=\"trait function_grep::filter::Info\">Info</a>&lt;Supported = <a class=\"struct\" href=\"function_grep/filter/struct.Language.html\" title=\"struct function_grep::filter::Language\">Language</a>&gt;&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.StructuralPartialEq.html\" title=\"trait core::marker::StructuralPartialEq\">StructuralPartialEq</a> for <a class=\"enum\" href=\"function_grep/filter/enum.SingleOrMany.html\" title=\"enum function_grep::filter::SingleOrMany\">SingleOrMany</a>&lt;A, M&gt;</h3></section>","StructuralPartialEq","function_grep::filter::FilterType","function_grep::filter::InstantiatedFilterType"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[12827]}