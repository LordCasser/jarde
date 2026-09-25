// jarde: presentation of `StringIterableForeach` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
final class StringIterableForeach extends java.lang.Object {
    StringIterableForeach() {
        // @method <init>()V
        // @declaration a constructor of `StringIterableForeach`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `sumLengths(Ljava/lang/Iterable;)I`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    static int sumLengths(java.lang.Iterable arg0) {
        // @method sumLengths(Ljava/lang/Iterable;)I
        // @declaration a static method of `StringIterableForeach`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        java.util.Iterator local2;
        local1 = 0;
        local2 = arg0.iterator();
        while (local2.hasNext()) {
            java.lang.String local3 = (java.lang.String) local2.next();
            local1 = local1 + local3.length();
        }
        return local1;
    }

    // jarde: generic Signature projection refused for `sumLengthsFrom(Ljava/util/function/Supplier;)I`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    static int sumLengthsFrom(java.util.function.Supplier arg0) {
        // @method sumLengthsFrom(Ljava/util/function/Supplier;)I
        // @declaration a static method of `StringIterableForeach`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local1;
        java.util.Iterator local2;
        local1 = 0;
        local2 = ((java.lang.Iterable) arg0.get()).iterator();
        while (local2.hasNext()) {
            java.lang.String local3 = (java.lang.String) local2.next();
            local1 = local1 + local3.length();
        }
        return local1;
    }
}
