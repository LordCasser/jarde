// jarde: presentation of `L3` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class L3 extends java.lang.Object {
    public L3() {
        // @method <init>()V
        // @declaration a constructor of `L3`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String chainPlain(int arg0) {
        // @method chainPlain(I)Ljava/lang/String;
        // @declaration a static method of `L3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.StringBuilder local1;
        int local2;
        local1 = new java.lang.StringBuilder();
        local1.append('a').append(arg0).append(';');
        for (local2 = 0; local2 < 2; local2 = local2 + 1) {
            local1.append('T').append(local2).append(',');
        }
        return local1.toString();
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `L3`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) chainPlain(7));
        return;
    }
}
