public class NL {
    static long hexLong = 0xCAFEBABEL;                       // hex long 字面量
    static int binLit = 0b1010_1010;                         // 二进制 + 下划线
    static long under = 1_000_000_000L;                      // 下划线十进制 long
    static double hexFloat = 0x1.91eb851eb851fp+1;           // hex float（π≈3.14）
    static char hexChar = 0x4E2D;                            // char 字面量（十六进制=中文'中'）
    static int charArith(char c){ return c * 2 + 1; }        // char 算术
    static String bits(long v){ return Long.toBinaryString(v); }
    public static void main(String[] a){
        System.out.println(""+hexLong+"/"+binLit+"/"+under+"/"+hexFloat+"/"+hexChar+"/"+charArith('A')+"/"+bits(0xFFL));
    }
}
