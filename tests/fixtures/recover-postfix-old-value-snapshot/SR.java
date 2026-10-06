public class SR {
    static int arrSelf(){ int i = 1; int[] a = new int[3]; a[i] = i++; return a[1] * 100 + i; }   // 存 RHS 旧值（102）
    static int backWrite(){ int i = 2; int[] a = {4,5,6}; a[i--] = a[0] + 100; return a[2] * 1000 + i; }  // 局部后缀作下标（写位）
    public static void main(String[] a){ System.out.println("" + arrSelf() + "/" + backWrite()); }
}
