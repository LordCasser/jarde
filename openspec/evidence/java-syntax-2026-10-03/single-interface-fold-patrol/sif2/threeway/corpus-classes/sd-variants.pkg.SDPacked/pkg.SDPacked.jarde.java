// jarde: presentation of `pkg/SDPacked` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package pkg;

public class SDPacked extends java.lang.Object {
    public SDPacked() {
        // @method <init>()V
        // @declaration a constructor of `pkg.SDPacked`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `pkg.SDPacked`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) new Use().name());
        return;
    }

    public static class Use extends java.lang.Object implements I {
        public Use() {
            // @method <init>()V
            // @declaration a constructor of `pkg.SDPacked$Use`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        public java.lang.String name() {
            // @method name()Ljava/lang/String;
            // @declaration an instance method of `pkg.SDPacked$Use`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return I.super.name();
        }
    }

    public static interface I {
        public default java.lang.String name() {
            // @method name()Ljava/lang/String;
            // @declaration an interface's default method of `pkg.SDPacked$I`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return "PI";
        }
    }
}
