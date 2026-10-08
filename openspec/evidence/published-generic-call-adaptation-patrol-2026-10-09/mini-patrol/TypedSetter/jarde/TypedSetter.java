// jarde: presentation of `TypedSetter` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class TypedSetter<T> extends java.lang.Object {
    // jarde: field Signature `TT;` projected after descriptor erasure and same-class uses at set(LTypedSetter;Ljava/lang/Object;)V@2
    public T value;

    public TypedSetter() {
        // @method <init>()V
        // @declaration a constructor of `TypedSetter`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `set(LTypedSetter;Ljava/lang/Object;)V`: unsupported (ordinary_generic_source_unproved): return source is not a proven parameter value or selected member creation
    public void set(TypedSetter c, java.lang.Object x) {
        // @method set(LTypedSetter;Ljava/lang/Object;)V
        // @declaration an instance method of `TypedSetter`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        c.value = x;
        return;
    }
}
