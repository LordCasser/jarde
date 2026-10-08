// jarde: presentation of `ThisDelegateHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ThisDelegateHold<T> extends java.lang.Object {
    // jarde: field Signature `TT;` projected after descriptor erasure and same-class uses at <init>(Ljava/lang/Object;I)V@6
    public T v;

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/Object;)V`: unsupported (ordinary_generic_source_unproved): member flags, annotation positions, or source name cannot be preserved
    public ThisDelegateHold(java.lang.Object arg1) {
        // @method <init>(Ljava/lang/Object;)V
        // @declaration a constructor of `ThisDelegateHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this(arg1, 0);
        return;
    }

    public ThisDelegateHold(T arg1, int arg2) {
        // jarde: generic Signature `(TT;I)V` projected after descriptor erasure and same-run AST/Code/SSA direct field-initializer proof; same-class call binding proved
        // @method <init>(Ljava/lang/Object;I)V
        // @declaration a constructor of `ThisDelegateHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.v = arg1;
        return;
    }
}
