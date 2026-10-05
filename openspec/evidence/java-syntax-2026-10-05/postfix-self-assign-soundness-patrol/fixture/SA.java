public class SA {
    static int postSelf(){ int i = 5; i = i++; return i; }              // i=i++ 经典陷阱（旧值回赋，自增丢弃→5）
    static int preSelf(){ int i = 5; i = ++i; return i; }               // ++i 先增后赋→6
    static int postOther(){ int i = 5; int j; j = i++; return i * 10 + j; }  // 跨变量（i=6,j=5→65）
    static int seqInc(){ int i = 5; i++; ++i; return i; }               // 语句位双形→7
    int foo;                                                             // 字段 foo
    int foo(){ return foo + 1; }                                        // 方法 foo()——命名空间合法共存
    int useBoth(){ return foo() + foo; }                                // 双命名空间消费
    static String uni(){ return "a\u0041b"; }                           // unicode 转义（\u0041='A'）
    static char charArith(char c){ c = (char) (c + 1); return c; }      // char 算术回赋
    public static void main(String[] a){ SA s = new SA(); System.out.println(""+postSelf()+"/"+preSelf()+"/"+postOther()+"/"+seqInc()+"/"+s.useBoth()+"/"+uni()+"/"+charArith('x')); }
}
