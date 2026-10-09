// jarde: presentation of `BoundOverload` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Number;:Ljava/lang/Comparable<TT;>;>Ljava/lang/Object;` projected after physical parent erasure proof
public class BoundOverload<T extends java.lang.Number & java.lang.Comparable<T>> extends java.lang.Object {
    public BoundOverload() {
        // @method <init>()V
        // @declaration a constructor of `BoundOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void pick(java.lang.Number arg1) {
        // @method pick(Ljava/lang/Number;)V
        // @declaration an instance method of `BoundOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("number");
        return;
    }

    // jarde: generic Signature projection refused for `pick(Ljava/lang/Comparable;)V`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    public void pick(java.lang.Comparable arg1) {
        // @method pick(Ljava/lang/Comparable;)V
        // @declaration an instance method of `BoundOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println("comparable");
        return;
    }

    public void relay(T arg1) {
        // jarde: generic Signature `(TT;)V` projected after descriptor erasure and same-run AST/Code/SSA straight-line void body proof
        // @method relay(Ljava/lang/Number;)V
        // @declaration an instance method of `BoundOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.pick(arg1);
        return;
    }
}
