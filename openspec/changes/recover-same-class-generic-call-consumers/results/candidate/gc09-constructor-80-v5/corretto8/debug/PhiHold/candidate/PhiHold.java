// jarde: presentation of `PhiHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class PhiHold<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer <init>(Ljava/lang/Object;Z)V@14 has no closed SSA source
    public java.lang.Object v;

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/Object;Z)V`: unsupported (ordinary_generic_source_unproved): member flags, annotation positions, or source name cannot be preserved
    public PhiHold(java.lang.Object v, boolean choose) {
        // @method <init>(Ljava/lang/Object;Z)V
        // @declaration a constructor of `PhiHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.v = choose ? v : null;
        return;
    }
}
