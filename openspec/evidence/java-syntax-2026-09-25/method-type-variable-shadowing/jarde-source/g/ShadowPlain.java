// jarde: presentation of `shadow/ShadowPlain` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package shadow;

// jarde: class Signature `<T:Ljava/lang/Object;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ShadowPlain<T> extends java.lang.Object {
    public ShadowPlain() {
        // @method <init>()V
        // @declaration a constructor of `shadow.ShadowPlain`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `echo(Ljava/lang/Object;)Ljava/lang/Object;`: unsupported (jvm_signature_scope_unproved): method type parameter `T` duplicates a name in the class or method scope
    public static java.lang.Object echo(java.lang.Object value) {
        // @method echo(Ljava/lang/Object;)Ljava/lang/Object;
        // @declaration a static method of `shadow.ShadowPlain`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return value;
    }
}
