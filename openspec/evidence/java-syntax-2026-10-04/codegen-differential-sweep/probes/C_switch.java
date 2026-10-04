public class C_switch {
    static int pick(int n){ switch(n){ case 1: return 10; case 2: return 20; case 3: return 30; case 4: return 40; case 5: return 50; default: return 0; } }
    public static void main(String[] a){ int s=0; for(int i=0;i<7;i++) s+=pick(i); System.out.println(s); }
}
