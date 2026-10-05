public class RH {
    static class Item { final String label; Item(String l){ label = l; } }
    static int total(Item[] xs){                       // 循环参数字段读 + 累积
        int n = 0;
        for(Item x : xs){ n += x.label.length(); }
        return n;
    }
    static String first(Item[] xs){                     // 单字段读返回
        return xs[0].label;
    }
    public static void main(String[] a){ System.out.println(""+total(new Item[]{new Item("ab"),new Item("cde")})+"/"+first(new Item[]{new Item("xy")})); }
}
