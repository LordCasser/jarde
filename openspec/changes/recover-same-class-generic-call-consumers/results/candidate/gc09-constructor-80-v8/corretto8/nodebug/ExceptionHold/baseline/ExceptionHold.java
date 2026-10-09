// jarde: presentation of `ExceptionHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ExceptionHold<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer <init>(Ljava/lang/Object;)V@10 has no closed SSA source
    public java.lang.Object v;

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/Object;)V`: unsupported (ordinary_generic_source_unproved): member flags, annotation positions, or source name cannot be preserved
    public ExceptionHold(java.lang.Object arg1) {
        // @method <init>(Ljava/lang/Object;)V
        // @declaration a constructor of `ExceptionHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        try {
            this.v = this.maybe(arg1);
        } catch (java.lang.RuntimeException local2) {
            this.v = null;
        }
        return;
    }

    private T maybe(T arg1) {
        // jarde: generic Signature `(TT;)TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method maybe(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `ExceptionHold`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return arg1;
    }
}
