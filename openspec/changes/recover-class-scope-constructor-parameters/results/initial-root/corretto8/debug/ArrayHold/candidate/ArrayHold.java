// jarde: presentation of `ArrayHold` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ArrayHold<T> extends java.lang.Object {
    // jarde: field Signature `[TT;` projected after descriptor erasure and same-class uses at <init>([Ljava/lang/Object;)V@6
    public T[] v;

    public ArrayHold(T[] v) {
        // jarde: generic Signature `([TT;)V` projected after descriptor erasure and same-run AST/Code/SSA direct field-initializer proof
        // @method <init>([Ljava/lang/Object;)V
        // @declaration a constructor of `ArrayHold`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.v = v;
        return;
    }
}
