// jarde: presentation of `RetainedAliasRawParam` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class RetainedAliasRawParam<T> extends java.lang.Object {
    // jarde: field Signature `TT;` projected after descriptor erasure and same-class uses at put(LRetainedAliasRawParam;Ljava/lang/Object;)Ljava/lang/Object;@4
    public T value;

    public RetainedAliasRawParam() {
        // @method <init>()V
        // @declaration a constructor of `RetainedAliasRawParam`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.Object put(RetainedAliasRawParam arg0, java.lang.Object arg1) {
        // @method put(LRetainedAliasRawParam;Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration a static method of `RetainedAliasRawParam`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        RetainedAliasRawParam local2 = arg0;
        local2.value = arg1;
        return local2;
    }
}
