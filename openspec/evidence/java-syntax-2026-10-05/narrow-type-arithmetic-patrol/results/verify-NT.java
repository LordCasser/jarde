public class NT extends java.lang.Object {
    public NT() {
        super();
        return;
    }

    static byte bump(byte arg0) {
        arg0 = (byte) (arg0 + 1);
        return arg0;
    }

    static short pre(short arg0) {
        arg0 = (short) (arg0 + 1);
        return arg0;
    }

    static char next(char arg0) {
        arg0 = (char) (arg0 + 1);
        return arg0;
    }

    static byte shl(byte arg0) {
        arg0 = (byte) (arg0 << 1);
        return arg0;
    }

    static short shr(short arg0) {
        arg0 = (short) (arg0 >> 2);
        return arg0;
    }

    static char addc(char arg0) {
        arg0 = (char) (arg0 + 1);
        return arg0;
    }

    static byte wrap() {
        int local0 = 127;
        local0 = (byte) (local0 + 1);
        return (byte) local0;
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println("" + (int) bump((byte) 41) + "/" + (int) pre((short) 305) + "/" + next('a') + "/" + (int) shl((byte) 3) + "/" + (int) shr((short) -9) + "/" + addc('x') + "/" + (int) wrap());
        return;
    }
}
