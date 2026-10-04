public class D_switchfall {
    static int f(int n){ switch(n){ case 1: case 2: return 10; case 3: return 30; default: return 0; } }  // fall-through 合并
    public static void main(String[] x){ System.out.println(f(1)+f(2)+f(3)+f(9)); }
}
