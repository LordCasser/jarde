// jarde: presentation of `L2` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class L2 extends java.lang.Object {
    public L2() {
        // @method <init>()V
        // @declaration a constructor of `L2`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String contWithTail() {
        // @method contWithTail()Ljava/lang/String;
        // @declaration a static method of `L2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local0;
        int local1;
        local0 = new java.lang.StringBuilder();
        int local2;
        loop: for (local1 = 0; local1 < 2; local1 = local1 + 1) {
            local2 = 0;
            while (local2 < 3) {
                if (local2 == 1) {
                    continue loop;
                } else {
                    local0.append(local1).append(local2).append(',');
                    local2 = local2 + 1;
                }
            }
            local0.append('T').append(local1).append(';');
        }
        return local0.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `L2`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) contWithTail());
        return;
    }
}
