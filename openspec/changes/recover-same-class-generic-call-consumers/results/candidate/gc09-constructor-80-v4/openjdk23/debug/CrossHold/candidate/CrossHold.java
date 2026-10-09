// jarde: presentation of `CrossHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;U:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class CrossHold<T, U> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer <init>(Ljava/lang/Object;)V@6 is not source-assignable to the projected field type
    public java.lang.Object v;

    public CrossHold(U v) {
        // jarde: generic Signature `(TU;)V` projected after descriptor erasure and same-run AST/Code/SSA constructor body proof
        // @method <init>(Ljava/lang/Object;)V
        // @declaration a constructor of `CrossHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.v = v;
        return;
    }
}
