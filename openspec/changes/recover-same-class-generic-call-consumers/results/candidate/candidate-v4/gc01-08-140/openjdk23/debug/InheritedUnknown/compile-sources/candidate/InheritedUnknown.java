// jarde: presentation of `InheritedUnknown` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/util/ArrayList<TT;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): parameterized or nested parent needs a separate inherited-member proof
public class InheritedUnknown extends java.util.ArrayList {
    public InheritedUnknown() {
        // @method <init>()V
        // @declaration a constructor of `InheritedUnknown`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public java.lang.Object relay(int index) {
        // @method relay(I)Ljava/lang/Object;
        // @declaration an instance method of `InheritedUnknown`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.get(index);
    }
}
