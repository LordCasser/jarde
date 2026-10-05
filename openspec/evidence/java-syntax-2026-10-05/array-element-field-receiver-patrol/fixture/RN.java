public class RN {
    static class Item { String tag; Item(String t){ tag = t; } }
    static String writeElem(Item[] xs, String v){ xs[0].tag = v; return xs[0].tag; }     // 元素字段写+读回
    static int writeLoop(Item[] xs, String v){ for(Item x:xs){ x.tag = v; } return xs[0].tag.length(); }
    public static void main(String[] z){ Item[] arr = new Item[]{new Item("a"),new Item("bb")}; System.out.println(""+writeElem(arr,"X")+"/"+writeLoop(arr,"YY")); }
}
