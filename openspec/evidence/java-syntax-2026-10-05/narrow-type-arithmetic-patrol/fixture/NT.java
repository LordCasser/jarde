public class NT {
    static byte bump(byte b){ b++; return b; }                       // byte 后缀（iadd+i2b 回转）
    static short pre(short s){ ++s; return s; }                      // short 前缀
    static char next(char c){ c++; return c; }                       // char 后缀（i2c）
    static byte shl(byte b){ b <<= 1; return b; }                    // byte 复合移位收窄
    static short shr(short s){ s >>= 2; return s; }                  // short 复合右移
    static char addc(char c){ c += 1; return c; }                    // char 复合加
    static byte wrap(){ byte b = 127; b++; return b; }               // 溢出回绕（行为敏感）
    public static void main(String[] a){ System.out.println(""+bump((byte) 41)+"/"+pre((short) 305)+"/"+next('a')+"/"+shl((byte) 3)+"/"+shr((short) -9)+"/"+addc('x')+"/"+wrap()); }
}
