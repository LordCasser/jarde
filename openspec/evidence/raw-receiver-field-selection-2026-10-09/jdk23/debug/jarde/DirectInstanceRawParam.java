// jarde: presentation of `DirectInstanceRawParam` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class DirectInstanceRawParam<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `valueLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer put(LDirectInstanceRawParam;Ljava/lang/Object;)V@2 is not source-assignable to the projected field type
    public java.lang.Object value;

    public DirectInstanceRawParam() {
        // @method <init>()V
        // @declaration a constructor of `DirectInstanceRawParam`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void put(DirectInstanceRawParam receiver, java.lang.Object value) {
        // @method put(LDirectInstanceRawParam;Ljava/lang/Object;)V
        // @declaration an instance method of `DirectInstanceRawParam`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        receiver.value = value;
        return;
    }
}
