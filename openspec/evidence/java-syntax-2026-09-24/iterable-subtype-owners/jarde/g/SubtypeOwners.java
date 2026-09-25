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
    public static int sumList(java.util.List values) {
        // @method sumList(Ljava/util/List;)I
        // @declaration a static method of `SubtypeOwners`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        java.util.Iterator local2;
        sum = 0;
        local2 = values.iterator();
        while (local2.hasNext()) {
            java.lang.String value = (java.lang.String) local2.next();
            sum = sum + value.length();
        }
        return sum;
    }

    // jarde: generic Signature projection refused for `sumCollection(Ljava/util/Collection;)I`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    public static int sumCollection(java.util.Collection values) {
        // @method sumCollection(Ljava/util/Collection;)I
        // @declaration a static method of `SubtypeOwners`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        java.util.Iterator local2;
        sum = 0;
        local2 = values.iterator();
        while (local2.hasNext()) {
            java.lang.String value = (java.lang.String) local2.next();
            sum = sum + value.length();
        }
        return sum;
    }

    public static int sumTextIterable(TextIterable values) {
        // @method sumTextIterable(LTextIterable;)I
        // @declaration a static method of `SubtypeOwners`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        int sum;
        java.util.Iterator local2;
        sum = 0;
        local2 = values.iterator();
        while (local2.hasNext()) {
            java.lang.String value = (java.lang.String) local2.next();
            sum = sum + value.length();
        }
        return sum;
    }
}
