// jarde: presentation of `UnusedHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class UnusedHold<T> extends java.lang.Object {
    // jarde: field Signature `TT;` projected after descriptor erasure and no same-class Fieldref
    public T v;

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/Object;)V`: unsupported (ordinary_generic_source_unproved): member flags, annotation positions, or source name cannot be preserved
    public UnusedHold(java.lang.Object ignored) {
        // @method <init>(Ljava/lang/Object;)V
        // @declaration a constructor of `UnusedHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }
}
