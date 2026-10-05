public class SF {
    static int fall(int x){ int s = 0; switch(x){ case 1: s += 10; case 2: s += 20; break; case 3: s += 30; case 4: s += 40; break; default: s = -1; } return s; }  // 1→贯穿到2；3→贯穿到4
    static int partial(int x){ switch(x){ case 1: case 2: return 12; case 3: return 3; } return 0; }   // 空 case 合并（不同形）
    public static void main(String[] a){ System.out.println(""+fall(1)+"/"+fall(2)+"/"+fall(3)+"/"+fall(4)+"/"+fall(9)+"/"+partial(1)+"/"+partial(2)+"/"+partial(3)+"/"+partial(7)); }
}
