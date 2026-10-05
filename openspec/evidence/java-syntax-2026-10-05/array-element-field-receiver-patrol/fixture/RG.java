import java.util.HashMap;
import java.util.Map;
public class RG {
    static class Item {
        final String label;
        Item(String l){ this.label = l; }
    }
    static final Item A = new Item("alpha");
    static final Item B = new Item("beta");
    static final Item C = new Item("gamma");
    static final Map<String,Item> BY_LABEL = new HashMap<String,Item>();
    static {                                                   // 普通类静态查找表（无 enum 掩蔽）
        Item[] all = new Item[]{ A, B, C };
        for(Item c : all){ BY_LABEL.put(c.label, c); }
    }
    static Item of(String l){ return BY_LABEL.get(l); }
    public static void main(String[] a){ System.out.println(""+of("beta")+"/"+of("alpha")+"/"+of("?")); }
}
