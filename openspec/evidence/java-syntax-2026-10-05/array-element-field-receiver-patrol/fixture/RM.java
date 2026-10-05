public class RM {
    static class Item { final String label; Item(String l){ label = l; } int len(){ return label.length(); } }
    static int loopCall(Item[] xs){ int n=0; for(Item x:xs){ n += x.len(); } return n; }   // 元素方法调用
    static int elemCall(Item[] xs){ return xs[0].len(); }                                    // 直接元素调用
    public static void main(String[] a){ System.out.println(""+loopCall(new Item[]{new Item("ab"),new Item("cde")})+"/"+elemCall(new Item[]{new Item("xy")})); }
}
