public class RK {
    static class Item { final String label; Item(String l){ label = l; } }
    static final Item Q = new Item("q");
    static String viaElemLocal(Item[] xs){ RK.Item y = xs[0]; return y.label; }   // 元素→局部→字段
    static String viaElemDirect(Item[] xs){ return xs[0].label; }                 // 元素直接字段（对照）
    public static void main(String[] a){ Item[] arr = new Item[]{ Q }; System.out.println(""+viaElemLocal(arr)+"/"+viaElemDirect(arr)); }
}
