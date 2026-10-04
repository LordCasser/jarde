public class NU {
    static int minNeg(){ return Integer.MIN_VALUE / -1; }              // 回绕（= MIN_VALUE 本身）
    static int minAbs(){ return Math.abs(Integer.MIN_VALUE); }          // abs 仍负
    static int uShift(int x){ return x >>> 1; }                         // 无符号右移
    static int uShiftByte(byte b){ return (b & 0xFF) >>> 1; }           // byte 提升链
    static int shiftMask(int x){ return (x << 3) & 0xF; }               // 移位+掩码
    static int negMod(int a, int b){ return a % b; }                    // 负数模
    static long mix(long v){ return (v << 32) | (v >>> 32); }           // long 双向移位组合
    static int rev(int x){ return Integer.reverse(x); }                 // intrinsic 候选
    public static void main(String[] a){ System.out.println(""+minNeg()+"/"+minAbs()+"/"+uShift(-8)+"/"+uShiftByte((byte)-2)+"/"+shiftMask(0xFF)+"/"+negMod(-7,3)+"/"+mix(0x1122334455667788L)+"/"+rev(1)); }
}
