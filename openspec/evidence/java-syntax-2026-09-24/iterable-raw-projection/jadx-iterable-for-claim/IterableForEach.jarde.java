// jarde: presentation of `IterableForEach` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class IterableForEach extends java.lang.Object {
    public IterableForEach() {
        // @method <init>()V
        // @declaration a constructor of `IterableForEach`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `test(Ljava/lang/Iterable;)Ljava/lang/String;`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    public java.lang.String test(java.lang.Iterable a) {
        // @method test(Ljava/lang/Iterable;)Ljava/lang/String;
        // @declaration an instance method of `IterableForEach`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder sb;
        java.util.Iterator local3;
        sb = new java.lang.StringBuilder();
        local3 = a.iterator();
        while (local3.hasNext()) {
            java.lang.String s = (java.lang.String) local3.next();
            sb.append(s);
        }
        return sb.toString();
    }
}
