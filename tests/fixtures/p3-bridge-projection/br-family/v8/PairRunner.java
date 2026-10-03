public class PairRunner {
    public static void main(String[] a) {
        BR$Node n = new BR$Base();
        System.out.println(n.next().getClass().getName());
        BR$Box b = new BR$StrBox();
        System.out.println(b.get());
        BR$StrBox s = new BR$StrBox();
        s.set("x");
    }
}
