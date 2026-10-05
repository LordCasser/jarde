public class RH {
    static class Item { final String label; Item(String l){ label = l; } }
    static final Item AB = new Item("ab");
    static final Item CDE = new Item("cde");
    static final Item XY = new Item("xy");
    static int total(Item[] xs){                       // 循环参数字段读 + 累积
        int n = 0;
        for(Item x : xs){ n += x.label.length(); }
        return n;
    }
    static String first(Item[] xs){                     // 单字段读返回
        return xs[0].label;
    }
    // 巡查原件把 `new Item[]{new Item(..)}` 直接写在实参位：那是内联数组初始化的分配元素通道
    // （本片范围外的现存拒绝），所以入口把数组取到局部，元素经静态字段读入。
    public static void main(String[] a){
        Item[] xs = new Item[]{ AB, CDE };
        Item[] ys = new Item[]{ XY };
        System.out.println(""+total(xs)+"/"+first(ys));
    }
}
