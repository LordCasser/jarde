// jarde: presentation of `M1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class M1 extends java.lang.Object {
    static Deep deepField;

    Base baseField;

    public M1() {
        // @method <init>()V
        // @declaration a constructor of `M1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    void work() throws Err, java.lang.RuntimeException {
        // @method work()V
        // @declaration an instance method of `M1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return;
    }

    static Deep make() throws Err {
        // @method make()LM1$Deep;
        // @declaration a static method of `M1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new Deep();
    }

    public static void main(java.lang.String[] arg0) throws Err {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `M1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        M1 local1 = new M1();
        local1.baseField = new Deep();
        local1.baseField.hi();
        local1.work();
        java.lang.System.out.println("ok");
        return;
    }

    static class Deep extends Base implements Ctrl {
        Deep() {
            // @method <init>()V
            // @declaration a constructor of `M1$Deep`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }
    }

    static class Base extends java.lang.Object {
        Base() {
            // @method <init>()V
            // @declaration a constructor of `M1$Base`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        void hi() {
            // @method hi()V
            // @declaration an instance method of `M1$Base`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            java.lang.System.out.println("hi");
            return;
        }
    }

    static class Inner extends java.lang.Object {
        Inner() {
            // @method <init>()V
            // @declaration a constructor of `M1$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }
    }

    static class Err extends java.lang.Exception {
        Err() {
            // @method <init>()V
            // @declaration a constructor of `M1$Err`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }
    }

    static interface Ctrl {
    }
}
