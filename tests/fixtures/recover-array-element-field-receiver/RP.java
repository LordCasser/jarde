public class RP {
    static class Item { final String label; Item(String l){ label = l; } }
    static class Sub extends Item { Sub(String l){ super(l); } }
    static final Sub S = new Sub("s");
    static final Item I = new Item("i");
    static String merged(Sub[] subs, Item[] items, boolean b){ Item[] z = b ? subs : items; return z[0].label; }
    static String branchy(Sub[] subs, Item[] items, boolean b){ Item[] z; if (b) { z = subs; } else { z = items; } return z[0].label; }
    public static void main(String[] a){
        Sub[] subs = new Sub[]{ S };
        Item[] items = new Item[]{ I };
        System.out.println(""+merged(subs,items,true)+"/"+branchy(subs,items,false));
    }
}
