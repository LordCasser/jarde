// jarde: presentation of `MV3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class MV3 extends java.lang.Object {
    private int base;

    public MV3() {
        // @method <init>()V
        // @declaration a constructor of `MV3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.base = 4;
        return;
    }

    Inner make(int arg1) {
        // @method make(I)LMV3$Inner;
        // @declaration an instance method of `MV3`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return new Inner(arg1);
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `MV3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(new StatB().b());
        java.lang.System.out.println(new MV3().make(2).total());
        return;
    }

    class Inner extends java.lang.Object {
        int tag;

        Inner(int arg2) {
            // @method <init>(LMV3;I)V
            // @declaration a constructor of `MV3$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            this.tag = arg2;
            return;
        }

        int total() {
            // @method total()I
            // @declaration an instance method of `MV3$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return this.tag + MV3.this.base;
        }
    }

    static class StatB extends StatA {
        StatB() {
            // @method <init>()V
            // @declaration a constructor of `MV3$StatB`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        int b() {
            // @method b()I
            // @declaration an instance method of `MV3$StatB`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return this.a() + 1;
        }
    }

    static class StatA extends java.lang.Object {
        StatA() {
            // @method <init>()V
            // @declaration a constructor of `MV3$StatA`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        int a() {
            // @method a()I
            // @declaration an instance method of `MV3$StatA`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return 5;
        }
    }
}
