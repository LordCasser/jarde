// jarde: presentation of `ShadowHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ShadowHold<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer <init>(Ljava/lang/Object;)V@10 has no closed SSA source
    public java.lang.Object v;

    // jarde: generic Signature projection refused for `shadow(Ljava/lang/Object;)Ljava/lang/Object;`: same-class generic call dependency did not close over every incoming use
    private java.lang.Object shadow(java.lang.Object x) {
        // @method shadow(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration an instance method of `ShadowHold`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/Object;)V`: unsupported (generic_constructor_source_unproved): same-run AST/SSA does not prove the constructor body
    public ShadowHold(java.lang.Object v) {
        // @method <init>(Ljava/lang/Object;)V
        // @declaration a constructor of `ShadowHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.v = this.shadow(v);
        return;
    }
}
