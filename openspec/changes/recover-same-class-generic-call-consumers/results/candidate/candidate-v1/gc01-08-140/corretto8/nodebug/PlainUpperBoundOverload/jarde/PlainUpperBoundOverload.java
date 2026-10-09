// jarde: presentation of `PlainUpperBoundOverload` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Number;>Ljava/lang/Object;` projected after physical parent erasure proof
public class PlainUpperBoundOverload<T extends java.lang.Number> extends java.lang.Object {
    public java.lang.String selected;

    public PlainUpperBoundOverload() {
        // @method <init>()V
        // @declaration a constructor of `PlainUpperBoundOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public void pick(java.lang.Number arg1) {
        // @method pick(Ljava/lang/Number;)V
        // @declaration an instance method of `PlainUpperBoundOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.selected = "number";
        return;
    }

    public void pick(java.lang.Object arg1) {
        // @method pick(Ljava/lang/Object;)V
        // @declaration an instance method of `PlainUpperBoundOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.selected = "object";
        return;
    }

    public void relay(T arg1) {
        // jarde: generic Signature `(TT;)V` projected after descriptor erasure and same-run AST/Code/SSA straight-line void body proof; same-class call binding proved
        // @method relay(Ljava/lang/Number;)V
        // @declaration an instance method of `PlainUpperBoundOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.pick(arg1);
        return;
    }
}
