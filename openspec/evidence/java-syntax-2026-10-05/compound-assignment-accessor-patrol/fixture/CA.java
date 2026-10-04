public class CA {
    private int seed = 7;
    class Nut {
        Nut(){ seed += 1; }                 // 子类构造器读外类私有字段 -> access$000 读形 + access$100 写形?
        int peek(){ return seed; }          // 普通方法读（对照：应折叠）
    }
    public static void main(String[] a){ CA c = new CA(); Nut n = c.new Nut(); System.out.println(n.peek()); }
}
