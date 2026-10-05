public class NJ {
    int tag = 5;
    class Inner { int v; Inner(int v){ this.v = v; } int outerTag(){ return tag * 2; } }
    public static void main(String[] a){ NJ n = new NJ(); NJ.Inner in = n.new Inner(3); System.out.println(""+in.v+"/"+in.outerTag()); }   // n 消费 2 次
}
