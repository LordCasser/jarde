// jarde: presentation of `WideHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class WideHold<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer <init>(JDLjava/lang/Object;)V@7 is not source-assignable to the projected field type
    public java.lang.Object v;

    // jarde: generic Signature projection refused for `<init>(JDLjava/lang/Object;)V`: unsupported (ordinary_generic_source_unproved): member flags, annotation positions, or source name cannot be preserved
    public WideHold(long arg1, double arg3, java.lang.Object arg5) {
        // @method <init>(JDLjava/lang/Object;)V
        // @declaration a constructor of `WideHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.v = arg5;
        return;
    }
}
