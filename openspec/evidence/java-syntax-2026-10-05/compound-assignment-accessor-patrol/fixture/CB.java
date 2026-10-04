public class CB {
    private int seed = 7;
    class Nut { void bump(){ seed += 1; } int peek(){ return seed; } }
    public static void main(String[] a){ CB c = new CB(); CB.Nut n = c.new Nut(); n.bump(); System.out.println(n.peek()); }
}
