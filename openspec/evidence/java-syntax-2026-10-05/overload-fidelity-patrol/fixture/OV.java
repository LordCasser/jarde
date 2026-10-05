public class OV {
    static String f(int x){ return "int"; }
    static String f(Integer x){ return "Integer"; }
    static String f(long x){ return "long"; }
    static String f(Object x){ return "Object"; }
    static String f(int... xs){ return "varargs"; }
    static String pick1(){ return f(1); }                 // 精确 int
    static String pick2(){ return f((Integer) 1); }       // Integer（cast 强制）
    static String pick3(){ return f((Object) 1); }        // Object（cast 强制）
    static String pick4(){ return f(1, 2); }              // varargs（多实参只有它可行）
    static String pick5(Integer i){ return f(i); }        // Integer 参数（不加宽不装箱的最精确）
    static byte narrow(byte a, byte b){ return (byte) (a + b); }   // byte+byte→int 提升+回 cast
    static char narrowC(char a, char b){ return (char) (a + b); }  // char 同
    static short narrowS(short a, short b){ return (short) (a + b); }
    public static void main(String[] a){ System.out.println(""+pick1()+pick2()+pick3()+pick4()+pick5(2)+"/"+narrow((byte)100,(byte)100)+"/"+narrowC('a','b')+"/"+narrowS((short)1,(short)2)); }
}
