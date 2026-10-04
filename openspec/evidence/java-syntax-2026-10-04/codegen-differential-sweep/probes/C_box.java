public class C_box {
    static Integer box(int x){ return x; }
    static int unbox(Integer i){ return i; }
    public static void main(String[] a){ Integer i=box(5); System.out.println(unbox(i)+box(300)); }
}
