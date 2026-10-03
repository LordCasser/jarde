// jarde: presentation of `IV4` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class IV4 extends java.lang.Object {
    private int base;

    public IV4() {
        // @method <init>()V
        // @declaration a constructor of `IV4`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.base = 4;
        return;
    }

    static int access$000(IV4 arg0) {
        // @method access$000(LIV4;)I
        // @declaration a static method of `IV4`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        return arg0.base + 100;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `IV4`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(new IV4().new Inner().total());
        return;
    }

    class Inner extends java.lang.Object {
        Inner() {
            // @method <init>(LIV4;)V
            // @declaration a constructor of `IV4$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        int total() {
            // @method total()I
            // @declaration an instance method of `IV4$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            return IV4.access$000(IV4.this);
        }
    }
}
