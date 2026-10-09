// jarde: presentation of `RewrittenHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class RewrittenHold<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer <init>(Ljava/lang/Object;)V@11 has no closed SSA source
    public java.lang.Object v;

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/Object;)V`: same-class generic call dependency did not close over every incoming use
    public RewrittenHold(java.lang.Object v) {
        // @method <init>(Ljava/lang/Object;)V
        // @declaration a constructor of `RewrittenHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        v = java.lang.String.valueOf(v);
        this.v = v;
        return;
    }
}
