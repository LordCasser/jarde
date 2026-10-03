// jarde: presentation of `WCDefault` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class WCDefault extends java.lang.Object {
    public WCDefault() {
        // @method <init>()V
        // @declaration a constructor of `WCDefault`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `WCDefault`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(A.sv() + new Use().dv());
        return;
    }

    static interface A {
        public static int sv() {
            // @method sv()I
            // @declaration an interface's static method of `WCDefault$A`, member flags 0x0009
            // recovered from bytecode; presentation is not claimed to compile
            return 8;
        }

        public default int dv() {
            // @method dv()I
            // @declaration an interface's default method of `WCDefault$A`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return 3;
        }
    }

    static class Use extends java.lang.Object implements A {
        Use() {
            // @method <init>()V
            // @declaration a constructor of `WCDefault$Use`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }
    }
}
