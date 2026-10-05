public class RM {
    static class Item { final String label; Item(String l){ label = l; } int len(){ return label.length(); } }
    static final Item AB = new Item("ab");
    static final Item CDE = new Item("cde");
    static final Item XY = new Item("xy");
    static int loopCall(Item[] xs){ int n=0; for(Item x:xs){ n += x.len(); } return n; }   // 元素方法调用
    static int elemCall(Item[] xs){ return xs[0].len(); }                                    // 直接元素调用
    public static void main(String[] a){
        Item[] xs = new Item[]{ AB, CDE };
        Item[] ys = new Item[]{ XY };
        System.out.println(""+loopCall(xs)+"/"+elemCall(ys));
    }
}
