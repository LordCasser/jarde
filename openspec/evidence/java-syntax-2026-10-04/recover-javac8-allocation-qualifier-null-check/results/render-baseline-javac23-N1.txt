// jarde: presentation of `N1` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class N1 extends java.lang.Object {
    private int base;

    public N1() {
        // @method <init>()V
        // @declaration a constructor of `N1`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.base = 4;
        return;
    }

    Inner make(int arg1) {
        // @method make(I)LN1$Inner;
        // @declaration an instance method of `N1`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return new Inner(arg1);
    }

    static int useStatic() {
        // @method useStatic()I
        // @declaration a static method of `N1`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return new N1().make(6).total();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `N1`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(useStatic());
        java.lang.System.out.println(new N1().new Inner(3).total());
        java.lang.System.out.println(new Stat().use(new N1()));
        return;
    }

    class Inner extends java.lang.Object {
        private int tag;

        Inner(int arg2) {
            // @method <init>(LN1;I)V
            // @declaration a constructor of `N1$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            this.tag = arg2;
            return;
        }

        int total() {
            // @method total()I
            // @declaration an instance method of `N1$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return this.tag + N1.this.base;
        }
    }

    static class Stat extends java.lang.Object {
        Stat() {
            // @method <init>()V
            // @declaration a constructor of `N1$Stat`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        int m() {
            // @method m()I
            // @declaration an instance method of `N1$Stat`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return 1;
        }

        int use(N1 arg1) {
            // @method use(LN1;)I
            // @declaration an instance method of `N1$Stat`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return arg1.new Inner(9).total();
        }
    }
}
