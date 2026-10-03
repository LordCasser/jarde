// jarde: presentation of `SDDiamond` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class SDDiamond extends java.lang.Object {
    public SDDiamond() {
        // @method <init>()V
        // @declaration a constructor of `SDDiamond`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `SDDiamond`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        Use local1 = new Use();
        java.lang.System.out.println((java.lang.String) local1.name());
        java.lang.System.out.println((java.lang.String) local1.mixed());
        local1.logCall();
        java.lang.System.out.println((java.lang.String) local1.greetCall());
        return;
    }

    static class Use extends java.lang.Object implements A, B {
        Use() {
            // @method <init>()V
            // @declaration a constructor of `SDDiamond$Use`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        public java.lang.String name() {
            // @method name()Ljava/lang/String;
            // @declaration an instance method of `SDDiamond$Use`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return A.super.name() + B.super.name();
        }

        java.lang.String mixed() {
            // @method mixed()Ljava/lang/String;
            // @declaration an instance method of `SDDiamond$Use`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return A.super.name() + "!";
        }

        void logCall() {
            // @method logCall()V
            // @declaration an instance method of `SDDiamond$Use`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            A.super.log("hi");
            return;
        }

        java.lang.String greetCall() {
            // @method greetCall()Ljava/lang/String;
            // @declaration an instance method of `SDDiamond$Use`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return A.super.greet(7);
        }
    }

    static interface B {
        public default java.lang.String name() {
            // @method name()Ljava/lang/String;
            // @declaration an interface's default method of `SDDiamond$B`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return "B";
        }
    }

    static interface A {
        public default java.lang.String name() {
            // @method name()Ljava/lang/String;
            // @declaration an interface's default method of `SDDiamond$A`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return "A";
        }

        public default void log(java.lang.String arg1) {
            // @method log(Ljava/lang/String;)V
            // @declaration an interface's default method of `SDDiamond$A`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            java.lang.System.out.println("log:" + arg1);
            return;
        }

        public default java.lang.String greet(int arg1) {
            // @method greet(I)Ljava/lang/String;
            // @declaration an interface's default method of `SDDiamond$A`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return "g" + arg1;
        }
    }
}
