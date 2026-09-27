// jarde: presentation of `cf02/PredicateEffects` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package cf02;

public class PredicateEffects extends java.lang.Object {
    public static int calls;

    public PredicateEffects() {
        // @method <init>()V
        // @declaration a constructor of `cf02.PredicateEffects`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String probe(java.lang.String arg0) {
        // @method probe(Ljava/lang/String;)Ljava/lang/String;
        // @declaration a static method of `cf02.PredicateEffects`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        cf02.PredicateEffects.calls = cf02.PredicateEffects.calls + 1;
        return arg0;
    }

    public static boolean named(java.lang.Object arg0) {
        // @method named(Ljava/lang/Object;)Z
        // @declaration a static method of `cf02.PredicateEffects`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        if (arg0 != null) {
            if (!(arg0 instanceof java.lang.String)) {
            } else {
                return probe((java.lang.String) arg0).length() > 0;
            }
        }
        return false;
    }
}
