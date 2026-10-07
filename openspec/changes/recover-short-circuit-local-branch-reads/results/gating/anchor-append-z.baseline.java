// jarde: presentation of `ScvConcatConsumers` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
public class ScvConcatConsumers extends java.lang.Object {
    public ScvConcatConsumers() {
        // @method <init>()V
        // @declaration a constructor of `ScvConcatConsumers`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        return;
    }

    public static java.lang.String plain(int arg0) {
        // @method plain(I)Ljava/lang/String;
        // @declaration a static method of `ScvConcatConsumers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local1 = (arg0 & 1) != 0 && (arg0 & 2) != 0;
        return "" + local1 + ":";
    }

    public static java.lang.String boxedHead(int arg0) {
        // @method boxedHead(I)Ljava/lang/String;
        // @declaration a static method of `ScvConcatConsumers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local1 = (arg0 & 1) != 0 && (arg0 & 2) != 0;
        return "" + local1 + ":";
    }

    public static java.lang.String doubleChain(int arg0) {
        // @method doubleChain(I)Ljava/lang/String;
        // @declaration a static method of `ScvConcatConsumers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local1 = (arg0 & 1) != 0 && (arg0 & 2) != 0;
        boolean local2 = (arg0 & 4) != 0 || (arg0 & 8) != 0;
        return "" + local1 + ":" + local2;
    }

    public static java.lang.String singleTail(int arg0) {
        // @method singleTail(I)Ljava/lang/String;
        // @declaration a static method of `ScvConcatConsumers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        boolean local1 = (arg0 & 1) != 0 && (arg0 & 2) != 0;
        boolean local2 = (arg0 & 4) != 0;
        return "" + local1 + ":" + local2;
    }

    public static void main(java.lang.String[] arg0) {
        // @method main([Ljava/lang/String;)V
        // @declaration a static method of `ScvConcatConsumers`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        java.lang.System.out.println((java.lang.String) plain(3));
        java.lang.System.out.println((java.lang.String) boxedHead(3));
        java.lang.System.out.println((java.lang.String) doubleChain(5));
        java.lang.System.out.println((java.lang.String) singleTail(5));
        return;
    }
}
