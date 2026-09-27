// jarde: presentation of `em01/Generic$A` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em01;

// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;Ljava/lang/Comparable<Lem01/Generic$A<TT;>;>;` not projected
// jarde: class Signature projection refused: unsupported (class_generic_source_unproved): class kind, nesting, name, or type-use annotations lack a faithful generic header position
public abstract class Generic$A extends java.lang.Object implements java.lang.Comparable {
    // jarde: field Signature projection refused for `valueLjava/lang/Object;`: unsupported (jvm_signature_scope_unproved): type variable `T` is not declared in the available Signature scope
    java.lang.Object value;

    public Generic$A() {
        // @method <init>()V
        // @declaration a constructor of `em01.Generic$A`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `compareTo(Lem01/Generic$A;)I`: unsupported (jvm_signature_scope_unproved): type variable `T` is not declared in the available Signature scope
    public int compareTo(em01.Generic$A arg1) {
        // @method compareTo(Lem01/Generic$A;)I
        // @declaration an instance method of `em01.Generic$A`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return 0;
    }

    public int compareTo(java.lang.Object arg1) {
        // @method compareTo(Ljava/lang/Object;)I
        // @declaration an instance method of `em01.Generic$A`, member flags 0x1041
        // recovered from bytecode; presentation is not claimed to compile
        return this.compareTo((em01.Generic$A) arg1);
    }
}
