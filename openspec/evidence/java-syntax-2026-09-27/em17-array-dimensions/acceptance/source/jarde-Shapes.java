// jarde: presentation of `em17/Shapes` from the class file's own declaration and one recovery run per member.
// jarde: not a compilable project: no imports and no resources are claimed (the `package` line is the class file's own name, not a claim about a directory); every place this text is not a full recovery carries a marker of this prefix.
package em17;

public class Shapes extends java.lang.Object {
    private char[] payload;

    public Shapes(byte[] arg1) {
        // @method <init>([B)V
        // @declaration a constructor of `em17.Shapes`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        super();
        char[] local2 = toChars(arg1);
        this.payload = new char[local2.length];
        java.lang.System.arraycopy((java.lang.Object) local2, 0, (java.lang.Object) this.payload, 0, arg1.length);
        return;
    }

    private static char[] toChars(byte[] arg0) {
        // @method toChars([B)[C
        // @declaration a static method of `em17.Shapes`, member flags 0x000a
        // recovered from bytecode; presentation is not claimed to compile
        return new char[arg0.length];
    }

    public int payloadLength() {
        // @method payloadLength()I
        // @declaration an instance method of `em17.Shapes`, member flags 0x0001
        // recovered from bytecode; presentation is not claimed to compile
        return this.payload.length;
    }

    public static long[][] longRows(int arg0) {
        // @method longRows(I)[[J
        // @declaration a static method of `em17.Shapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new long[arg0][];
    }

    public static java.lang.String[][] stringRows(int arg0) {
        // @method stringRows(I)[[Ljava/lang/String;
        // @declaration a static method of `em17.Shapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.String[arg0][];
    }

    public static int[][][] deep(int arg0) {
        // @method deep(I)[[[I
        // @declaration a static method of `em17.Shapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new int[arg0][][];
    }

    public static int[][] full(int arg0, int arg1) {
        // @method full(II)[[I
        // @declaration a static method of `em17.Shapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new int[arg0][arg1];
    }

    public static int[][] literal(int arg0, int arg1) {
        // @method literal(II)[[I
        // @declaration a static method of `em17.Shapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new int[][]{new int[]{1}, new int[]{arg0, arg1}, new int[0]};
    }

    public static java.lang.Object[] wrapped(byte[] arg0) {
        // @method wrapped([B)[Ljava/lang/Object;
        // @declaration a static method of `em17.Shapes`, member flags 0x0009
        // recovered from bytecode; presentation is not claimed to compile
        return new java.lang.Object[]{arg0};
    }
}
