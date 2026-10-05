public class CO2 {
    static boolean even(int n){ if(n == 0){ return true; } return odd(n-1); }      // if/else 互递归（对照）
    static boolean odd(int n){ if(n == 0){ return false; } return even(n-1); }
    static boolean litTern(int n){ return n == 0 ? true : Boolean.valueOf(n > 0).booleanValue(); }  // 字面量 vs 调用（无递归）
    public static void main(String[] a){ System.out.println(""+even(10)+"/"+odd(7)+"/"+litTern(3)); }
}
