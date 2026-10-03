// jarde: presentation of `F1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class F1 extends java.lang.Object {
    public F1() {
        // @method <init>()V
        // @declaration a constructor of `F1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static java.lang.String diamond() {
        // @method diamond()Ljava/lang/String;
        // @declaration a static method of `F1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new Diamond().name();
    }

    static java.lang.String reabstract() {
        // @method reabstract()Ljava/lang/String;
        // @declaration a static method of `F1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new F1$Reabstract$Impl().name();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `F1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) diamond());
        java.lang.System.out.println((java.lang.String) reabstract());
        return;
    }

    static class Diamond extends java.lang.Object implements A, B {
        Diamond() {
            // @method <init>()V
            // @declaration a constructor of `F1$Diamond`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        public java.lang.String name() {
            // @method name()Ljava/lang/String;
            // @declaration an instance method of `F1$Diamond`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return A.super.name() + B.super.name();
        }
    }

    static class Reabstract extends java.lang.Object implements A {
        Reabstract() {
            // @method <init>()V
            // @declaration a constructor of `F1$Reabstract`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }
    }

    static interface B {
        public default java.lang.String name() {
            // @method name()Ljava/lang/String;
            // @declaration an interface's default method of `F1$B`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return "B";
        }
    }

    static interface A {
        public default java.lang.String name() {
            // @method name()Ljava/lang/String;
            // @declaration an interface's default method of `F1$A`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return "A";
        }
    }
}
