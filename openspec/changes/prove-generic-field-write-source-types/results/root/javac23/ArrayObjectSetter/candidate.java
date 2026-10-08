// jarde: presentation of `ArrayObjectSetter` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ArrayObjectSetter<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `v[Ljava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer put([Ljava/lang/Object;)V@2 is not source-assignable to the projected field type
    public java.lang.Object[] v;

    public ArrayObjectSetter() {
        // @method <init>()V
        // @declaration a constructor of `ArrayObjectSetter`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void put(java.lang.Object[] arg1) {
        // @method put([Ljava/lang/Object;)V
        // @declaration an instance method of `ArrayObjectSetter`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.v = arg1;
        return;
    }
}
