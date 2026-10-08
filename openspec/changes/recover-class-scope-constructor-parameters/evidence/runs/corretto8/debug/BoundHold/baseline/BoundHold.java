// jarde: presentation of `BoundHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Number;>Ljava/lang/Object;` projected after physical parent erasure proof
public class BoundHold<T extends java.lang.Number> extends java.lang.Object {
    // jarde: field Signature projection refused for `vLjava/lang/Number;`: unsupported (field_generic_write_source_unproved): writer <init>(Ljava/lang/Number;)V@6 is not source-assignable to the projected field type
    public java.lang.Number v;

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/Number;)V`: unsupported (ordinary_generic_source_unproved): member flags, annotation positions, or source name cannot be preserved
    public BoundHold(java.lang.Number v) {
        // @method <init>(Ljava/lang/Number;)V
        // @declaration a constructor of `BoundHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.v = v;
        return;
    }
}
