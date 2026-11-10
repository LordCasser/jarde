# Body类型命名固定输入

来自s1-body-type-names-v1/pilot-r6的javac23 --release8 -g:none受控Client。两个JAR仅含Client.class，引用分别为Outer.Node$Part、Family$Top.Node$Part。InnerClasses原始行在Client内；原始依赖和隔离Driver及完整编译/行为原流在对应pilot目录。这里只覆盖class literal/cast/instanceof/array，new路径的独立anonymous_typed_nesting_unproved问题保留在tools/pilot/evidence-r5。
