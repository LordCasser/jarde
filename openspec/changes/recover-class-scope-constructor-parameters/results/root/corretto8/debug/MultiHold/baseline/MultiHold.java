// jarde: presentation of `MultiHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;U::Ljava/lang/CharSequence;>Ljava/lang/Object;` projected after physical parent erasure proof
public class MultiHold<T, U extends java.lang.CharSequence> extends java.lang.Object {
    // jarde: field Signature projection refused for `firstLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer <init>(Ljava/lang/Object;Ljava/lang/CharSequence;)V@6 is not source-assignable to the projected field type
    public java.lang.Object first;

    // jarde: field Signature projection refused for `secondLjava/lang/CharSequence;`: unsupported (field_generic_write_source_unproved): writer <init>(Ljava/lang/Object;Ljava/lang/CharSequence;)V@11 is not source-assignable to the projected field type
    public java.lang.CharSequence second;

    // jarde: generic Signature projection refused for `<init>(Ljava/lang/Object;Ljava/lang/CharSequence;)V`: unsupported (ordinary_generic_source_unproved): member flags, annotation positions, or source name cannot be preserved
    public MultiHold(java.lang.Object first, java.lang.CharSequence second) {
        // @method <init>(Ljava/lang/Object;Ljava/lang/CharSequence;)V
        // @declaration a constructor of `MultiHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.first = first;
        this.second = second;
        return;
    }
}
