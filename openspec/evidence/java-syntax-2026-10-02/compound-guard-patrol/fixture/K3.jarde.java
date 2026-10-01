// jarde: presentation of `K3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class K3 extends java.lang.Object {
    static int x = init();

    public K3() {
        // @method <init>()V
        // @declaration a constructor of `K3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    static int init() {
        // @method init()I
        // @declaration a static method of `K3`, member flags 0x0008
        // recovered from bytecode; presentation is not claimed to compile
        int local0;
        int local1;
        local0 = 1;
        for (local1 = 0; local1 < 4; local1 = local1 + 1) {
            local0 = local0 * 2 + local1 % 2;
        }
        return local0;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `K3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println(K3.x);
        return;
    }
}
