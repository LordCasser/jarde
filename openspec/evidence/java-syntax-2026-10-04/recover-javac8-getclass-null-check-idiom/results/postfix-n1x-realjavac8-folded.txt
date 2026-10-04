// jarde: presentation of `N1x` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class N1x extends java.lang.Object {
    private int base;

    public N1x() {
        // @method <init>()V
        // @declaration a constructor of `N1x`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.base = 4;
        return;
    }

    Inner make(int arg1) {
        // @method make(I)LN1x$Inner;
        // @declaration an instance method of `N1x`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return new Inner(arg1);
    }

    static int useStatic() {
        // @method useStatic()I
        // @declaration a static method of `N1x`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new N1x().make(6).total();
    }

    public static int statUse(N1x arg0) {
        // @method statUse(LN1x;)I
        // @declaration a static method of `N1x`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new Stat().use(arg0);
    }

    static class Stat extends java.lang.Object {
        Stat() {
            // @method <init>()V
            // @declaration a constructor of `N1x$Stat`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        int m() {
            // @method m()I
            // @declaration an instance method of `N1x$Stat`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return 1;
        }

        int use(N1x arg1) {
            // @method use(LN1x;)I
            // @declaration an instance method of `N1x$Stat`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return arg1.new Inner(9).total();
        }
    }

    class Inner extends java.lang.Object {
        private int tag;

        Inner(int arg2) {
            // @method <init>(LN1x;I)V
            // @declaration a constructor of `N1x$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            this.tag = arg2;
            return;
        }

        int total() {
            // @method total()I
            // @declaration an instance method of `N1x$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return this.tag + N1x.this.base;
        }
    }
}
