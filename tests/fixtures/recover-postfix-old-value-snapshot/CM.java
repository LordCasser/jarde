public class CM {
    static int a, b;
    static int compound(){ int x = 10; x += 5; x -= 3; x *= 2; x /= 4; x %= 4; return x; }  // 复合赋值链
    static int compoundInExpr(){ int x = 10; int y = (x += 5); return x + y; }                 // 表达式内复合赋值
    static int compoundField(){ CM.a += 3; CM.b -= 1; return CM.a + CM.b; }                    // 静态字段复合
    static int incDec(){ int i = 5; int j = i++; int k = ++i; int l = i--; int m = --i; return i + j + k + l + m; } // 前/后缀四形
    static int incField(){ CM.a++; ++CM.a; return CM.a; }                                      // 字段前/后缀
    public static void main(String[] args){ System.out.println(""+compound()+"/"+compoundInExpr()+"/"+compoundField()+"/"+incDec()+"/"+incField()); }
}
