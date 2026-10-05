public class WB {
    private int tally;
    private java.util.List<String> bag;
    private int[] grid;
    class Writer {
        void putTally(int v) { tally = v; }
        void putBag(java.util.List<String> v) { bag = v; }
        void putGrid(int[] v) { grid = v; }
    }
    public static void main(String[] a) {
        WB o = new WB(); Writer w = o.new Writer();
        w.putTally(11); w.putBag(java.util.Collections.<String>emptyList()); w.putGrid(new int[]{1,2});
        System.out.println(o.tally + "|" + o.bag.size() + "|" + o.grid.length);
    }
}
