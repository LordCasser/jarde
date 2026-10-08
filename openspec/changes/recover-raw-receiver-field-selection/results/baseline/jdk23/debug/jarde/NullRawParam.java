// jarde: presentation of `NullRawParam` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class NullRawParam<T> extends java.lang.Object {
    // jarde: field Signature `TT;` projected after descriptor erasure and same-class uses at put(LNullRawParam;)V@2
    public T value;

    public NullRawParam() {
        // @method <init>()V
        // @declaration a constructor of `NullRawParam`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void put(NullRawParam receiver) {
        // @method put(LNullRawParam;)V
        // @declaration a static method of `NullRawParam`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        receiver.value = null;
        return;
    }
}
