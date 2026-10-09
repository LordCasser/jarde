// jarde: presentation of `SameNameOverload` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class SameNameOverload<T> extends java.lang.Object {
    public java.lang.String selected;

    public SameNameOverload() {
        // @method <init>()V
        // @declaration a constructor of `SameNameOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `pick(Ljava/lang/Object;)V`: unsupported (generic_call_binding_unproved): a same-class Methodref names this method or an adjacent overload
    public void pick(java.lang.Object arg1) {
        // @method pick(Ljava/lang/Object;)V
        // @declaration an instance method of `SameNameOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.selected = "generic";
        return;
    }

    public void pick(java.lang.String arg1) {
        // @method pick(Ljava/lang/String;)V
        // @declaration an instance method of `SameNameOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.selected = "string";
        return;
    }

    public void relay(T arg1) {
        // jarde: generic Signature `(TT;)V` projected after descriptor erasure and same-run AST/Code/SSA straight-line void body proof
        // @method relay(Ljava/lang/Object;)V
        // @declaration an instance method of `SameNameOverload`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        this.pick(arg1);
        return;
    }
}
