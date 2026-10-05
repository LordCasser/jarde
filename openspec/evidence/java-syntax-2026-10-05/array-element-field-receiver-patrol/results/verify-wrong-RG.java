public class RG extends java.lang.Object {
    static final Item A;

    static final Item B;

    static final Item C;

    static final java.util.Map BY_LABEL;

    public RG() {
        super();
        return;
    }

    static Item of(java.lang.String arg0) {
        return (Item) RG.BY_LABEL.get((java.lang.Object) arg0);
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println("" + of("beta") + "/" + of("alpha") + "/" + of("?"));
        return;
    }

    static {
        Item[] local1;
        A = new Item("alpha");
        B = new Item("beta");
        C = new Item("gamma");
        BY_LABEL = new java.util.HashMap();
        Item[] local0 = new Item[]{RG.A, RG.B, RG.C};
        local1 = local0;
        for (Item local4 : local1) {
        }
    }
static class Item extends java.lang.Object {
    final java.lang.String label;

    Item(java.lang.String arg1) {
        super();
        this.label = arg1;
        return;
    }
}
}
