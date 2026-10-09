// jarde: presentation of `RawOwnerChild` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>LRawOwnerBase<TT;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): parameterized or nested parent needs a separate inherited-member proof
public class RawOwnerChild extends RawOwnerBase {
    public RawOwnerChild() {
        // @method <init>()V
        // @declaration a constructor of `RawOwnerChild`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void put(RawOwnerChild arg0, java.lang.Object arg1) {
        // @method put(LRawOwnerChild;Ljava/lang/Object;)V
        // @declaration a static method of `RawOwnerChild`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        arg0.value = arg1;
        return;
    }
}
