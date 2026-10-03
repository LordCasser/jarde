// jarde: presentation of `IV2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class IV2 extends java.lang.Object {
    private int count;

    public IV2() {
        // @method <init>()V
        // @declaration a constructor of `IV2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        this.count = 10;
        return;
    }

    int get() {
        // @method get()I
        // @declaration an instance method of `IV2`, member flags 0x0000
        // recovered from bytecode; presentation is not claimed to compile
        return this.count;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `IV2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        IV2 local1 = new IV2();
        local1.new Inner().bump();
        java.lang.System.out.println(local1.get());
        return;
    }

    static int access$002(IV2 arg0, int arg1) {
        // jarde: not recovered: the recovery run for `access$002(LIV2;I)I` produced no statement (explanation only); the artifact's own comment lines are below
        // @method access$002(LIV2;I)I
        // @declaration a static method of `IV2`, member flags 0x1008
        // recovered from bytecode; presentation is not claimed to compile
        // @bytecode 2
        // the instruction at BCI 2 is not part of the provable subset
        // @bytecode 3
        // the value at BCI 3 comes from an Other at BCI 2, which produces no expression this subset writes
        // @bytecode 6
        // the value at BCI 6 comes from an Other at BCI 2, which produces no expression this subset writes
    }

    class Inner extends java.lang.Object {
        Inner() {
            // @method <init>(LIV2;)V
            // @declaration a constructor of `IV2$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            super();
            return;
        }

        void bump() {
            // @method bump()V
            // @declaration an instance method of `IV2$Inner`, member flags 0x0000
            // recovered from bytecode; presentation is not claimed to compile
            IV2.access$002(IV2.this, IV2.this.count + 1);
            return;
        }
    }
}
