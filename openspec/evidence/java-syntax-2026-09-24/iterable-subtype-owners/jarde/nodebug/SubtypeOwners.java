// jarde: presentation of `SubtypeOwners` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class SubtypeOwners extends java.lang.Object {
    public SubtypeOwners() {
        // @method <init>()V
        // @declaration a constructor of `SubtypeOwners`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `sumList(Ljava/util/List;)I`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    public static int sumList(java.util.List arg0) {
        // @method sumList(Ljava/util/List;)I
        // @declaration a static method of `SubtypeOwners`, member flags 0x0009
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

    // jarde: generic Signature projection refused for `sumCollection(Ljava/util/Collection;)I`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    public static int sumCollection(java.util.Collection arg0) {
        // @method sumCollection(Ljava/util/Collection;)I
        // @declaration a static method of `SubtypeOwners`, member flags 0x0009
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

    public static int sumTextIterable(TextIterable arg0) {
        // @method sumTextIterable(LTextIterable;)I
        // @declaration a static method of `SubtypeOwners`, member flags 0x0009
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
}
