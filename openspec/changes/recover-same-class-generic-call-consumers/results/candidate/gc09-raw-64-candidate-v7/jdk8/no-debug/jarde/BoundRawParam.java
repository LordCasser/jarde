// jarde: presentation of `BoundRawParam` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Number;:Ljava/lang/Comparable<TT;>;>Ljava/lang/Object;` projected after physical parent erasure proof
public class BoundRawParam<T extends java.lang.Number & java.lang.Comparable<T>> extends java.lang.Object {
    // jarde: field Signature `TT;` projected after descriptor erasure and same-class uses at put(LBoundRawParam;Ljava/lang/Number;)V@2
    public T value;

    public BoundRawParam() {
        // @method <init>()V
        // @declaration a constructor of `BoundRawParam`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void put(BoundRawParam arg0, java.lang.Number arg1) {
        // @method put(LBoundRawParam;Ljava/lang/Number;)V
        // @declaration a static method of `BoundRawParam`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0.value = arg1;
        return;
    }
}
