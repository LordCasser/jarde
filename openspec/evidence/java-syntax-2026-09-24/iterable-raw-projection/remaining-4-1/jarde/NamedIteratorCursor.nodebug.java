// jarde: presentation of `NamedIteratorCursor` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public final class NamedIteratorCursor extends java.lang.Object {
    private final java.util.Iterator delegate;

    // jarde: generic Signature projection refused for `<init>(Ljava/util/Iterator;)V`: unsupported (generic_source_shape_unproved): the recovered AST/SSA body is not a direct parameter return
    public NamedIteratorCursor(java.util.Iterator arg1) {
        // @method <init>(Ljava/util/Iterator;)V
        // @declaration a constructor of `NamedIteratorCursor`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.delegate = arg1;
        return;
    }

    // jarde: generic Signature projection refused for `iterator()Ljava/util/Iterator;`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    public java.util.Iterator iterator() {
        // @method iterator()Ljava/util/Iterator;
        // @declaration an instance method of `NamedIteratorCursor`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.delegate;
    }

    public static int sum(NamedIteratorCursor arg0) {
        // @method sum(LNamedIteratorCursor;)I
        // @declaration a static method of `NamedIteratorCursor`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.util.Iterator local1;
        int local2;
        local1 = arg0.iterator();
        local2 = 0;
        while (local1.hasNext()) {
            java.lang.String local3 = (java.lang.String) local1.next();
            local2 = local2 + local3.length();
        }
        return local2;
    }
}
