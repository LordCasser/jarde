// jarde: presentation of `RawParam` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class RawParam<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `valueLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer put(LRawParam;Ljava/lang/Object;)V@2 is not source-assignable to the projected field type
    public java.lang.Object value;

    public RawParam() {
        // @method <init>()V
        // @declaration a constructor of `RawParam`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void put(RawParam arg0, java.lang.Object arg1) {
        // @method put(LRawParam;Ljava/lang/Object;)V
        // @declaration a static method of `RawParam`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0.value = arg1;
        return;
    }
}
