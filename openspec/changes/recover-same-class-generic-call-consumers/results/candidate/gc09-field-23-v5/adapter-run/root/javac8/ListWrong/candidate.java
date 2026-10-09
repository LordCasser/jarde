// jarde: presentation of `ListWrong` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ListWrong<T> extends java.lang.Object {
    // jarde: field Signature `Ljava/util/List<TT;>;` projected after descriptor erasure and same-class uses at put(Ljava/util/List;)V@2
    public java.util.List<T> v;

    public ListWrong() {
        // @method <init>()V
        // @declaration a constructor of `ListWrong`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `put(Ljava/util/List;)V`: same-class generic call dependency did not close over every incoming use
    public void put(java.util.List arg1) {
        // @method put(Ljava/util/List;)V
        // @declaration an instance method of `ListWrong`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.v = arg1;
        return;
    }
}
