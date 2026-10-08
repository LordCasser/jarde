// jarde: presentation of `WideRawParam` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class WideRawParam<T> extends java.lang.Object {
    // jarde: field Signature projection refused for `valueLjava/lang/Object;`: unsupported (field_generic_write_source_unproved): writer put(JLWideRawParam;DLjava/lang/Object;)V@3 is not source-assignable to the projected field type
    public java.lang.Object value;

    public WideRawParam() {
        // @method <init>()V
        // @declaration a constructor of `WideRawParam`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void put(long arg0, WideRawParam arg2, double arg3, java.lang.Object arg5) {
        // @method put(JLWideRawParam;DLjava/lang/Object;)V
        // @declaration a static method of `WideRawParam`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg2.value = arg5;
        return;
    }
}
