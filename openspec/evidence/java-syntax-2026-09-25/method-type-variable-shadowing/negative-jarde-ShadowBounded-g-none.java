// jarde: presentation of `shadow/ShadowBounded` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package shadow;

// jarde: class Signature `<T:Ljava/lang/Number;>Ljava/lang/Object;` projected after physical parent erasure proof
public class ShadowBounded<T extends java.lang.Number> extends java.lang.Object {
    public ShadowBounded() {
        // @method <init>()V
        // @declaration a constructor of `shadow.ShadowBounded`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    // jarde: generic Signature projection refused for `echo(Ljava/lang/CharSequence;)Ljava/lang/CharSequence;`: invalid input (jvm_signature_erasure_mismatch): erased Signature disagrees with the physical descriptor at parameter 0
    public static java.lang.CharSequence echo(java.lang.CharSequence arg0) {
        // @method echo(Ljava/lang/CharSequence;)Ljava/lang/CharSequence;
        // @declaration a static method of `shadow.ShadowBounded`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return arg0;
    }
}
