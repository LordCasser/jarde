// jarde: presentation of `WCMulti` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class WCMulti extends java.lang.Object {
    static B held;

    public WCMulti() {
        // @method <init>()V
        // @declaration a constructor of `WCMulti`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `WCMulti`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(A.sv() + B.sv());
        java.lang.System.out.println(WCMulti.held.iv());
        return;
    }

    static {
        // @method <clinit>()V
        // @declaration a static initializer of `WCMulti`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        WCMulti.held = new Impl();
    }

    static interface A {
        public static int sv() {
            // @method sv()I
            // @declaration an interface's static method of `WCMulti$A`, member flags 0x0009
            // recovered from bytecode; presentation is not claimed to compile
            return 8;
        }

        public default int dv() {
            // @method dv()I
            // @declaration an interface's default method of `WCMulti$A`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return 3;
        }
    }

    static interface B {
        public static int sv() {
            // @method sv()I
            // @declaration an interface's static method of `WCMulti$B`, member flags 0x0009
            // recovered from bytecode; presentation is not claimed to compile
            return 9;
        }

        // jarde: no body: the member `iv()I` is declared abstract and its declaration carries no Code attribute
        public abstract int iv();
    }

    static class Impl extends java.lang.Object implements B {
        Impl() {
            // @method <init>()V
            // @declaration a constructor of `WCMulti$Impl`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        public int iv() {
            // @method iv()I
            // @declaration an instance method of `WCMulti$Impl`, member flags 0x0001
            // recovered from bytecode; presentation is not claimed to compile
            return 4;
        }
    }
}
