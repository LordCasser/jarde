// jarde: presentation of `StaticRawField` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class StaticRawField extends java.lang.Object {
    // jarde: field Signature `Ljava/util/List<Ljava/lang/String;>;` projected after descriptor erasure and same-class uses at put(Ljava/util/List;)V@1
    public static java.util.List<java.lang.String> v;

    public StaticRawField() {
        // @method <init>()V
        // @declaration a constructor of `StaticRawField`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void put(java.util.List arg0) {
        // @method put(Ljava/util/List;)V
        // @declaration a static method of `StaticRawField`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        StaticRawField.v = arg0;
        return;
    }
}
