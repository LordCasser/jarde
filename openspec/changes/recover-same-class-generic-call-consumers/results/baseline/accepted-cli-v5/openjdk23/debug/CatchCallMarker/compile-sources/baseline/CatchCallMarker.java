// jarde: presentation of `CatchCallMarker` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class CatchCallMarker<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `valueLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer <init>(Ljava/lang/Object;Z)V@11 has no closed SSA source
    public java.lang.Object value;

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/Object;Z)V`: unsupported (ordinary_generic_source_unproved): member flags, annotation positions, or source name cannot be preserved
    public CatchCallMarker(java.lang.Object x, boolean fail) {
        // @method <init>(Ljava/lang/Object;Z)V
        // @declaration a constructor of `CatchCallMarker`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        try {
            this.value = this.maybe(x, fail);
        } catch (java.lang.RuntimeException ex) {
            this.value = null;
        }
        return;
    }

    // jarde: generic Signature projection refused for `maybe(Ljava/lang/Object;Z)Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    private java.lang.Object maybe(java.lang.Object x, boolean fail) {
        // @method maybe(Ljava/lang/Object;Z)Ljava/lang/Object;
        // @declaration an instance method of `CatchCallMarker`, member flags 0x0002
        // recovered from bytecode; presentation is not claimed to compile
        if (fail) {
            throw new java.lang.RuntimeException();
        } else {
            return x;
        }
    }
}
