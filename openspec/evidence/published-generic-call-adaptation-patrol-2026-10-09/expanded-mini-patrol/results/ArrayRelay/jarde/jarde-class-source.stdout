// jarde: presentation of `ArrayRelay` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ArrayRelay<T> extends java.lang.Object {
    public ArrayRelay() {
        // @method <init>()V
        // @declaration a constructor of `ArrayRelay`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public T[] id(T[] x) {
        // jarde: generic Signature `([TT;)[TT;` projected after descriptor erasure and same-run AST/SSA parameter-return proof; same-class call binding proved
        // @method id([Ljava/lang/Object;)[Ljava/lang/Object;
        // @declaration an instance method of `ArrayRelay`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return x;
    }

    // jarde: generic Signature projection refused for `relay([Ljava/lang/Object;)[Ljava/lang/Object;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
    public java.lang.Object[] relay(java.lang.Object[] x) {
        // @method relay([Ljava/lang/Object;)[Ljava/lang/Object;
        // @declaration an instance method of `ArrayRelay`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.id(x);
    }

    public static void main(java.lang.String[] a) {
        // jarde: not recovered: the recovery run for `main([Ljava/lang/String;)V` produced no statement (explanation only); the artifact's own comment lines are below
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ArrayRelay`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 1
        // the array instruction at BCI 1 produces a value nothing in this body reads, so the instruction the bytecode runs has no place in the text
        // @bytecode 4 1
        // the instruction at BCI 4 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 6
        // the instruction at BCI 6 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 9
        // the instruction at BCI 9 belongs to no shape this run verified: an allocation, a copy or a cast is presented only where a rule proved what it builds
        // @bytecode 13 1 10 0 4 5 6 9 14 15 18 19 22 23 26 29 30 33 35 38 39 40 43 44 47 48 51 52 55 58 61
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 14 1
        // the copy at BCI 4 has no proved local assignment
        // @bytecode 23 26 29 30 33 35 38 39 40 43 44 47 48 51 52 55 58
        // the statement at BCI 58 reads `m`, and no statement of this body declared that local: the write that would have declared it was refused, so its name cannot be read here (P3 2b.2)
        jarde_refused_body();
    }
}
